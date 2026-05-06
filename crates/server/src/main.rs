mod app;
mod cli;
mod routes;
mod state;
mod tls;

use clap::Parser;
use cli::Args;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    app::run(Args::parse()).await
}




