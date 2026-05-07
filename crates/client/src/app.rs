use crate::{
    cli::Args,
    http_client::ControlPlaneClient,
    progress::{build_file_progress, build_torrent_progress},
    torrent::{ensure_torrent, TorrentRuntime},
};
use fast_distribution_core::{now_unix_ms, ClientReport};
use librqbit::Session;
use std::{collections::HashMap, time::Duration};
use tracing::{info, warn};

pub async fn run(args: Args) -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let client = ControlPlaneClient::new(args.control_plane_url.clone(), &args.cert_path)?;
    let session = Session::new(args.download_dir.clone()).await?;
    let mut torrents: HashMap<String, TorrentRuntime> = HashMap::new();

    loop {
        let poll = match client.poll().await {
            Ok(poll) => poll,
            Err(error) => {
                warn!(?error, "poll failed");
                tokio::time::sleep(Duration::from_secs(30)).await;
                continue;
            }
        };

        info!("received {} file entries", poll.files.len());

        for file in poll.files {
            ensure_torrent(&session, &mut torrents, file).await;
        }

        for runtime in torrents.values_mut() {
            let stats = runtime.handle.stats();
            let now = now_unix_ms();
            let file_progress =
                build_file_progress(&args.download_dir, runtime, &stats, now).await;
            let torrent_progress = build_torrent_progress(&stats);

            let report = ClientReport {
                client_id: args.client_id.clone(),
                file_id: runtime.file.file_id.clone(),
                torrent: torrent_progress,
                file: file_progress,
                timestamp_unix_ms: now,
            };

            if let Err(error) = client.report(&report).await {
                warn!(?error, "report failed");
            }
        }

        tokio::time::sleep(Duration::from_secs(poll.next_poll_seconds)).await;
    }
}
