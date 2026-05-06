use axum::{
    extract::State,
    http::StatusCode,
    response::Html,
    routing::{get, post},
    Form, Json, Router,
};
use fast_distribution_core::{
    AdminAddFileRequest, AdminAddFileResponse, ClientProgressEntry, ClientReport,
    ClientPollResponse, FileInfo, FileProgressEntry, ServerProgressResponse,
    ServerStatusResponse,
};
use serde::Deserialize;
use std::collections::HashMap;

use crate::state::SharedState;

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

fn add_file_inner(state: &mut crate::state::AppState, payload: AddFileForm) -> String {
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
    Form(payload): Form<AddFileForm>,
) -> Html<String> {
    let mut state = state.lock().expect("state lock");
    let file_id = add_file_inner(&mut state, payload);
    Html(render_template(
        ADDED_TEMPLATE,
        &[("{{file_id}}", &file_id)],
    ))
}
