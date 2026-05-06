use fast_distribution_core::{ClientReport, FileInfo};
use std::{collections::HashMap, sync::{Arc, Mutex}};

#[derive(Debug, Default)]
pub struct AppState {
    pub next_file_id: u64,
    pub files: HashMap<String, FileInfo>,
    pub reports: HashMap<String, HashMap<String, ClientReport>>,
}

pub type SharedState = Arc<Mutex<AppState>>;

