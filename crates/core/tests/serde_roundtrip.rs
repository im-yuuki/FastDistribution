use fast_distribution_core::{ClientReport, FileProgress, FileState, TorrentProgress, TorrentStatus};

#[test]
fn serde_roundtrip_report() {
    let report = ClientReport {
        client_id: "client-1".to_string(),
        file_id: "file-1".to_string(),
        torrent: TorrentProgress {
            status: TorrentStatus::Leeching,
            peers: 3,
            downloaded_bytes: 10,
            uploaded_bytes: 2,
            download_rate_bps: 100,
            upload_rate_bps: 10,
            ratio: 0.2,
            last_error: None,
        },
        file: FileProgress {
            state: FileState::Downloading,
            total_bytes: 100,
            completed_bytes: 10,
            checksum_hex: None,
            last_checked_unix_ms: None,
            last_verified_unix_ms: None,
        },
        timestamp_unix_ms: 1,
    };

    let json = serde_json::to_string(&report).expect("serialize report");
    let decoded: ClientReport = serde_json::from_str(&json).expect("deserialize report");

    assert_eq!(decoded.client_id, report.client_id);
    assert_eq!(decoded.file_id, report.file_id);
}


