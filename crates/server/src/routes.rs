use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use fast_distribution_core::{
    AdminAddFileRequest, AdminAddFileResponse, ClientProgressEntry, ClientReport,
    ClientPollResponse, FileInfo, FileProgressEntry, ServerProgressResponse,
    ServerStatusResponse,
};
use std::collections::HashMap;

use crate::state::SharedState;

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/api/files", get(list_files).post(add_file))
        .route("/api/status", get(progress_status))
        .route("/api/poll", get(client_poll))
        .route("/api/report", post(client_report))
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

async fn add_file(
    State(state): State<SharedState>,
    Json(payload): Json<AdminAddFileRequest>,
) -> Json<AdminAddFileResponse> {
    let mut state = state.lock().expect("state lock");
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

