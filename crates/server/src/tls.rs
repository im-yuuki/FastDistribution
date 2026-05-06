use axum_server::tls_rustls::RustlsConfig;

pub async fn maybe_load_config(
    cert_path: Option<&str>,
    key_path: Option<&str>,
) -> anyhow::Result<Option<RustlsConfig>> {
    match (cert_path, key_path) {
        (Some(cert_path), Some(key_path)) => {
            let config = RustlsConfig::from_pem_file(cert_path, key_path).await?;
            Ok(Some(config))
        }
        _ => Ok(None),
    }
}

