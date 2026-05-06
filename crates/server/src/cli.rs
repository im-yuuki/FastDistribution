use clap::Parser;
use std::net::SocketAddr;

#[derive(Parser, Debug)]
#[command(name = "fast_distribution_server")]
pub struct Args {
    #[arg(long, default_value = "0.0.0.0:12405")]
    pub bind: SocketAddr,
    #[arg(long)]
    pub tls_cert: Option<String>,
    #[arg(long)]
    pub tls_key: Option<String>,
}

