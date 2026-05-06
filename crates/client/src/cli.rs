use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "fast_distribution_client")]
pub struct Args {
    #[arg(long, default_value = "certificate.crt")]
    pub cert_path: String,
    #[arg(long, default_value = "client-1")]
    pub client_id: String,
    #[arg(long, default_value = "downloads")]
    pub download_dir: PathBuf,
}

