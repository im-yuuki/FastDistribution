use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub file_id: String,
    pub file_name: String,
    pub magnet_link: String,
    pub total_bytes: u64,
    pub checksum_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatusResponse {
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerProgressResponse {
    pub files: Vec<FileProgressEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileProgressEntry {
    pub file: FileInfo,
    pub clients: Vec<ClientProgressEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientProgressEntry {
    pub client_id: String,
    pub torrent: TorrentProgress,
    pub file: FileProgress,
    pub timestamp_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPollResponse {
    pub files: Vec<FileInfo>,
    pub next_poll_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAddFileRequest {
    pub file_name: String,
    pub magnet_link: String,
    pub total_bytes: u64,
    pub checksum_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminAddFileResponse {
    pub file_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TorrentStatus {
    Idle,
    Leeching,
    Seeding,
    Paused,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TorrentProgress {
    pub status: TorrentStatus,
    pub peers: u32,
    pub downloaded_bytes: u64,
    pub uploaded_bytes: u64,
    pub download_rate_bps: u64,
    pub upload_rate_bps: u64,
    pub ratio: f32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileState {
    Missing,
    Downloading,
    CompleteUnverified,
    CompleteVerified,
    ChecksumFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileProgress {
    pub state: FileState,
    pub total_bytes: u64,
    pub completed_bytes: u64,
    pub checksum_hex: Option<String>,
    pub last_checked_unix_ms: Option<i64>,
    pub last_verified_unix_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientReport {
    pub client_id: String,
    pub file_id: String,
    pub torrent: TorrentProgress,
    pub file: FileProgress,
    pub timestamp_unix_ms: i64,
}


