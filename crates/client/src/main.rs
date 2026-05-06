mod app;
mod checksum;
mod cli;
mod http_client;
mod progress;
mod torrent;

use clap::Parser;
use cli::Args;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    app::run(Args::parse()).await
}





