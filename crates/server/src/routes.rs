use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Json, Router,
};
use axum::extract::multipart::Multipart;
use fast_distribution_core::{
    AdminAddFileRequest, AdminAddFileResponse, ClientProgressEntry, ClientReport,
    ClientPollResponse, FileInfo, FileProgressEntry, ServerProgressResponse,
    ServerStatusResponse,
};
use serde::Deserialize;
use std::collections::HashMap;

use crate::state::AppState;

type SharedState = std::sync::Arc<std::sync::Mutex<AppState>>;

const MONITOR_TEMPLATE: &str = include_str!("../assets/monitor.html");
const ADD_TEMPLATE: &str = include_str!("../assets/add.html");
const ADDED_TEMPLATE: &str = include_str!("../assets/added.html");

fn render_template(template: &str, replacements: &[(&str, &str)]) -> String {
    let mut output = template.to_string();
    for (key, value) in replacements {
        output = output.replace(key, value);
    }
    output
}

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/api/files", get(list_files).post(add_file))
        .route("/api/files/upload", post(upload_file))
        .route("/api/status", get(progress_status))
        .route("/api/poll", get(client_poll))
        .route("/api/report", post(client_report))
        .route("/ui", get(ui_monitor))
        .route("/ui/add", get(ui_add_form).post(ui_add_submit))
        .with_state(state)
}

async fn list_files(State(state): State<SharedState>) -> Json<ServerStatusResponse> {
    let state = state.lock().expect("state lock");
    Json(ServerStatusResponse {
        files: state.files.values().cloned().collect(),
    })
}

async fn client_poll(State(state): State<SharedState>) -> Json<ClientPollResponse> {
    let state = state.lock().expect("state lock");
    Json(ClientPollResponse {
        files: state.files.values().cloned().collect(),
        next_poll_seconds: 30,
    })
}

#[derive(Debug, Deserialize)]
struct AddFileForm {
    file_name: String,
    magnet_link: String,
    total_bytes: u64,
    checksum_hex: Option<String>,
}

fn add_file_inner(state: &mut AppState, payload: AddFileForm) -> String {
    state.next_file_id += 1;
    let file_id = format!("file-{}", state.next_file_id);
    let info = FileInfo {
        file_id: file_id.clone(),
        file_name: payload.file_name,
        magnet_link: payload.magnet_link,
        total_bytes: payload.total_bytes,
        checksum_hex: payload.checksum_hex,
    };
    state.files.insert(file_id.clone(), info);
    file_id
}

async fn add_file(
    State(state): State<SharedState>,
    Json(payload): Json<AdminAddFileRequest>,
) -> Json<AdminAddFileResponse> {
    let mut state = state.lock().expect("state lock");
    let file_id = add_file_inner(
        &mut state,
        AddFileForm {
            file_name: payload.file_name,
            magnet_link: payload.magnet_link,
            total_bytes: payload.total_bytes,
            checksum_hex: payload.checksum_hex,
        },
    );
    Json(AdminAddFileResponse { file_id })
}

async fn upload_file(
    State(state): State<SharedState>,
    mut multipart: Multipart,
) -> (StatusCode, Json<AdminAddFileResponse>) {
    let mut file_name = String::new();
    let mut checksum_hex: Option<String> = None;
    let mut data: Option<Vec<u8>> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file_name" => file_name = field.text().await.unwrap_or_default(),
            "checksum_hex" => {
                let val = field.text().await.unwrap_or_default();
                if !val.is_empty() {
                    checksum_hex = Some(val);
                }
            }
            "file" => data = Some(field.bytes().await.map(|b| b.to_vec()).unwrap_or_default()),
            _ => {}
        }
    }

    let data = match data {
        Some(d) if !d.is_empty() => d,
        _ => return (StatusCode::BAD_REQUEST, Json(AdminAddFileResponse { file_id: "missing file field".into() })),
    };

    if file_name.is_empty() {
        return (StatusCode::BAD_REQUEST, Json(AdminAddFileResponse { file_id: "missing file_name".into() }));
    }

    let (file_id, share_dir, session) = {
        let mut state = match state.lock() {
            Ok(s) => s,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminAddFileResponse { file_id: e.to_string() })),
        };
        state.next_file_id += 1;
        let file_id = format!("file-{}", state.next_file_id);
        (file_id, state.share_dir.clone(), state.session.clone())
    };

    let total_bytes = data.len() as u64;
    let file_dir = share_dir.join(&file_id);
    if let Err(e) = tokio::fs::create_dir_all(&file_dir).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminAddFileResponse { file_id: e.to_string() }));
    }

    let file_path = file_dir.join(&file_name);
    if let Err(e) = tokio::fs::write(&file_path, &data).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminAddFileResponse { file_id: e.to_string() }));
    }

    let create_result = match librqbit::create_torrent(&file_path, librqbit::CreateTorrentOptions::default()).await {
        Ok(r) => r,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminAddFileResponse { file_id: e.to_string() })),
    };

    let info_hash = create_result.info_hash();
    let magnet = librqbit::Magnet::from_id20(info_hash, Vec::<String>::new(), None);
    let magnet_link = magnet.to_string();

    let torrent_bytes = match create_result.as_bytes() {
        Ok(b) => b,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminAddFileResponse { file_id: e.to_string() })),
    };

    let add_opts = librqbit::AddTorrentOptions {
        output_folder: Some(file_dir.to_string_lossy().into_owned()),
        ..Default::default()
    };

    if let Err(e) = session.add_torrent(librqbit::AddTorrent::from_bytes(torrent_bytes), Some(add_opts)).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminAddFileResponse { file_id: e.to_string() }));
    }

    tracing::info!(?file_id, ?magnet_link, "torrent created and seeding");

    {
        let mut state = match state.lock() {
            Ok(s) => s,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(AdminAddFileResponse { file_id: e.to_string() })),
        };
        let info = FileInfo {
            file_id: file_id.clone(),
            file_name,
            magnet_link,
            total_bytes,
            checksum_hex,
        };
        state.files.insert(file_id.clone(), info);
    }

    (StatusCode::OK, Json(AdminAddFileResponse { file_id }))
}

async fn client_report(
    State(state): State<SharedState>,
    Json(report): Json<ClientReport>,
) -> StatusCode {
    let mut state = state.lock().expect("state lock");
    let per_file = state
        .reports
        .entry(report.file_id.clone())
        .or_insert_with(HashMap::new);
    per_file.insert(report.client_id.clone(), report);
    StatusCode::NO_CONTENT
}

async fn progress_status(State(state): State<SharedState>) -> Json<ServerProgressResponse> {
    let state = state.lock().expect("state lock");
    let mut files = Vec::with_capacity(state.files.len());

    for file in state.files.values() {
        let mut clients = Vec::new();
        if let Some(reports) = state.reports.get(&file.file_id) {
            clients = reports
                .values()
                .map(|report| ClientProgressEntry {
                    client_id: report.client_id.clone(),
                    torrent: report.torrent.clone(),
                    file: report.file.clone(),
                    timestamp_unix_ms: report.timestamp_unix_ms,
                })
                .collect();
        }

        files.push(FileProgressEntry {
            file: file.clone(),
            clients,
        });
    }

    Json(ServerProgressResponse { files })
}

async fn ui_monitor(State(state): State<SharedState>) -> Html<String> {
    let state = state.lock().expect("state lock");
    let mut rows = String::new();
    for file in state.files.values() {
        let client_count = state
            .reports
            .get(&file.file_id)
            .map(|reports| reports.len())
            .unwrap_or(0);
        rows.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            file.file_name, file.file_id, file.total_bytes, client_count
        ));
    }

    Html(render_template(MONITOR_TEMPLATE, &[("{{rows}}", &rows)]))
}

async fn ui_add_form() -> Html<String> {
    Html(ADD_TEMPLATE.to_string())
}

async fn ui_add_submit(
    State(state): State<SharedState>,
    mut multipart: Multipart,
) -> Html<String> {
    let mut file_name = String::new();
    let mut checksum_hex: Option<String> = None;
    let mut data: Option<Vec<u8>> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "file_name" => file_name = field.text().await.unwrap_or_default(),
            "checksum_hex" => {
                let val = field.text().await.unwrap_or_default();
                if !val.is_empty() {
                    checksum_hex = Some(val);
                }
            }
            "file" => data = Some(field.bytes().await.map(|b| b.to_vec()).unwrap_or_default()),
            _ => {}
        }
    }

    let data = match data {
        Some(d) if !d.is_empty() => d,
        _ => return Html("<p>Error: missing or empty file</p><a href='/ui/add'>Back</a>".into()),
    };

    if file_name.is_empty() {
        return Html("<p>Error: missing file_name</p><a href='/ui/add'>Back</a>".into());
    }

    let (file_id, share_dir, session) = {
        let mut state = match state.lock() {
            Ok(s) => s,
            Err(e) => return Html(format!("<p>Error: {}</p><a href='/ui/add'>Back</a>", e)),
        };
        state.next_file_id += 1;
        let file_id = format!("file-{}", state.next_file_id);
        (file_id, state.share_dir.clone(), state.session.clone())
    };

    let total_bytes = data.len() as u64;
    let file_dir = share_dir.join(&file_id);
    if let Err(e) = tokio::fs::create_dir_all(&file_dir).await {
        return Html(format!("<p>Error creating dir: {}</p><a href='/ui/add'>Back</a>", e));
    }

    let file_path = file_dir.join(&file_name);
    if let Err(e) = tokio::fs::write(&file_path, &data).await {
        return Html(format!("<p>Error writing file: {}</p><a href='/ui/add'>Back</a>", e));
    }

    let create_result = match librqbit::create_torrent(&file_path, librqbit::CreateTorrentOptions::default()).await {
        Ok(r) => r,
        Err(e) => return Html(format!("<p>Error creating torrent: {}</p><a href='/ui/add'>Back</a>", e)),
    };

    let info_hash = create_result.info_hash();
    let magnet = librqbit::Magnet::from_id20(info_hash, Vec::<String>::new(), None);
    let magnet_link = magnet.to_string();

    let torrent_bytes = match create_result.as_bytes() {
        Ok(b) => b,
        Err(e) => return Html(format!("<p>Error serializing torrent: {}</p><a href='/ui/add'>Back</a>", e)),
    };

    let add_opts = librqbit::AddTorrentOptions {
        output_folder: Some(file_dir.to_string_lossy().into_owned()),
        ..Default::default()
    };

    if let Err(e) = session.add_torrent(librqbit::AddTorrent::from_bytes(torrent_bytes), Some(add_opts)).await {
        return Html(format!("<p>Error seeding torrent: {}</p><a href='/ui/add'>Back</a>", e));
    }

    {
        let mut state = match state.lock() {
            Ok(s) => s,
            Err(e) => return Html(format!("<p>Error: {}</p><a href='/ui/add'>Back</a>", e)),
        };
        let info = FileInfo {
            file_id: file_id.clone(),
            file_name,
            magnet_link: magnet_link.clone(),
            total_bytes,
            checksum_hex,
        };
        state.files.insert(file_id.clone(), info);
    }

    Html(render_template(
        ADDED_TEMPLATE,
        &[("{{file_id}}", &file_id), ("{{magnet_link}}", &magnet_link)],
    ))
}
