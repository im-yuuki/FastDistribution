use crate::{cli::Args, routes, state::AppState, tls};
use std::sync::{Arc, Mutex};
use tower_http::trace::TraceLayer;
use tracing::{info, warn};

pub async fn run(args: Args) -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let state = Arc::new(Mutex::new(AppState::default()));
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

