use fast_distribution_core::{ClientReport, FileInfo};
use std::{collections::HashMap, fmt, path::PathBuf, sync::Arc};

pub struct AppState {
    pub next_file_id: u64,
    pub files: HashMap<String, FileInfo>,
    pub reports: HashMap<String, HashMap<String, ClientReport>>,
    pub session: Arc<librqbit::Session>,
    pub share_dir: PathBuf,
}

impl fmt::Debug for AppState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppState")
            .field("next_file_id", &self.next_file_id)
            .field("files", &self.files)
            .field("reports", &self.reports)
            .field("session", &"<librqbit::Session>")
            .field("share_dir", &self.share_dir)
            .finish()
    }
}

