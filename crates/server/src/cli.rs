use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "fast_distribution_server")]
pub struct Args {
    #[arg(long, default_value = "0.0.0.0:12405")]
    pub bind: SocketAddr,
    #[arg(long)]
    pub tls_cert: Option<String>,
    #[arg(long)]
    pub tls_key: Option<String>,
    #[arg(long, default_value = "shares")]
    pub share_dir: PathBuf,
    #[arg(long, default_value_t = 6881)]
    pub bt_port: u16,
}

