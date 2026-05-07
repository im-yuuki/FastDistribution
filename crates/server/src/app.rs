use crate::{cli::Args, routes, state::AppState, tls};
use std::sync::{Arc, Mutex};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

pub async fn run(args: Args) -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    tokio::fs::create_dir_all(&args.share_dir).await?;

    if args.bt_port == u16::MAX {
        anyhow::bail!("--bt-port must be less than 65535");
    }

    let session = librqbit::Session::new_with_opts(
        args.share_dir.clone(),
        librqbit::SessionOptions {
            listen_port_range: Some(args.bt_port..args.bt_port + 1),
            ..Default::default()
        },
    )
    .await?;
    info!(port = session.tcp_listen_port(), "BitTorrent session started");

    let state = Arc::new(Mutex::new(AppState {
        next_file_id: 0,
        files: Default::default(),
        reports: Default::default(),
        session,
        share_dir: args.share_dir,
    }));
    let app = routes::router(state).layer(TraceLayer::new_for_http());

    if let Some(config) = tls::maybe_load_config(args.tls_cert.as_deref(), args.tls_key.as_deref()).await? {
        info!("listening on https://{}", args.bind);
        axum_server::bind_rustls(args.bind, config)
            .serve(app.into_make_service())
            .await?;
    } else {
        warn!("starting without TLS; provide --tls-cert and --tls-key to enable HTTPS");
        info!("listening on http://{}", args.bind);
        let listener = tokio::net::TcpListener::bind(args.bind).await?;
        axum::serve(listener, app.into_make_service()).await?;
    }

    Ok(())
}
