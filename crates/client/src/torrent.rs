use fast_distribution_core::FileInfo;
use librqbit::{AddTorrent, AddTorrentResponse, ManagedTorrent, Session};
use std::{collections::HashMap, path::{Path, PathBuf}, sync::Arc};
use tracing::warn;

pub struct TorrentRuntime {
    pub file: FileInfo,
    pub handle: Arc<ManagedTorrent>,
    pub checksum_status: Option<bool>,
    pub last_verified_unix_ms: Option<i64>,
}

pub async fn ensure_torrent(
    session: &Arc<Session>,
    torrents: &mut HashMap<String, TorrentRuntime>,
    file: FileInfo,
) {
    if torrents.contains_key(&file.file_id) {
        return;
    }

    let result = session
        .add_torrent(AddTorrent::from_url(file.magnet_link.clone()), None)
        .await;

    let handle = match result {
        Ok(AddTorrentResponse::Added(_, handle)) => handle,
        Ok(AddTorrentResponse::AlreadyManaged(_, handle)) => handle,
        Ok(AddTorrentResponse::ListOnly(_)) => {
            warn!("torrent added in list-only mode, skipping");
            return;
        }
        Err(error) => {
            warn!(?error, "failed to add torrent");
            return;
        }
    };

    torrents.insert(
        file.file_id.clone(),
        TorrentRuntime {
            file,
            handle,
            checksum_status: None,
            last_verified_unix_ms: None,
        },
    );
}

pub fn resolve_download_path(download_dir: &Path, handle: &Arc<ManagedTorrent>) -> Option<PathBuf> {
    let name = handle.name()?;
    Some(download_dir.join(name))
}

