use crate::{checksum::verify_checksum, torrent::{resolve_download_path, TorrentRuntime}};
use fast_distribution_core::{FileProgress, FileState, TorrentProgress, TorrentStatus};
use librqbit::{TorrentStats, TorrentStatsState};
use std::path::Path;
use tracing::warn;

pub fn build_torrent_progress(stats: &TorrentStats) -> TorrentProgress {
    let (download_rate_bps, upload_rate_bps, peers) = stats
        .live
        .as_ref()
        .map(|live| {
            let download = (live.download_speed.mbps * 1024.0 * 1024.0) as u64;
            let upload = (live.upload_speed.mbps * 1024.0 * 1024.0) as u64;
            let peers = live.snapshot.peer_stats.live as u32;
            (download, upload, peers)
        })
        .unwrap_or((0, 0, 0));

    let status = match stats.state {
        TorrentStatsState::Initializing => TorrentStatus::Idle,
        TorrentStatsState::Paused => TorrentStatus::Paused,
        TorrentStatsState::Error => TorrentStatus::Error,
        TorrentStatsState::Live => {
            if stats.finished {
                TorrentStatus::Seeding
            } else {
                TorrentStatus::Leeching
            }
        }
    };

    let ratio = if stats.progress_bytes == 0 {
        0.0
    } else {
        stats.uploaded_bytes as f32 / stats.progress_bytes as f32
    };

    TorrentProgress {
        status,
        peers,
        downloaded_bytes: stats.progress_bytes,
        uploaded_bytes: stats.uploaded_bytes,
        download_rate_bps,
        upload_rate_bps,
        ratio,
        last_error: stats.error.clone(),
    }
}

pub async fn build_file_progress(
    download_dir: &Path,
    runtime: &mut TorrentRuntime,
    stats: &TorrentStats,
    now: i64,
) -> FileProgress {
    let total_bytes = if stats.total_bytes == 0 {
        runtime.file.total_bytes
    } else {
        stats.total_bytes
    };

    let completed_bytes = stats.progress_bytes;
    let mut state = if completed_bytes == 0 {
        FileState::Missing
    } else {
        FileState::Downloading
    };

    if stats.finished {
        state = FileState::CompleteUnverified;
    }

    if stats.finished {
        if let Some(expected) = runtime.file.checksum_hex.as_deref() {
            if runtime.checksum_status.is_none() {
                if let Some(path) = resolve_download_path(download_dir, &runtime.handle) {
                    match verify_checksum(&path, expected).await {
                        Ok(result) => {
                            runtime.checksum_status = Some(result);
                            runtime.last_verified_unix_ms = Some(now);
                        }
                        Err(error) => {
                            warn!(?error, ?path, "checksum verification failed");
                        }
                    }
                }
            }

            state = match runtime.checksum_status {
                Some(true) => FileState::CompleteVerified,
                Some(false) => FileState::ChecksumFailed,
                None => FileState::CompleteUnverified,
            };
        }
    }

    FileProgress {
        state,
        total_bytes,
        completed_bytes,
        checksum_hex: runtime.file.checksum_hex.clone(),
        last_checked_unix_ms: Some(now),
        last_verified_unix_ms: runtime.last_verified_unix_ms,
    }
}

