use fast_distribution_core::{ClientPollResponse, ClientReport, CONTROL_PLANE_URL};
use reqwest::Certificate;

pub struct ControlPlaneClient {
    client: reqwest::Client,
}

impl ControlPlaneClient {
    pub fn new(cert_path: &str) -> anyhow::Result<Self> {
        let cert_bytes = std::fs::read(cert_path)?;
        let cert = Certificate::from_pem(&cert_bytes)?;
        let client = reqwest::Client::builder()
            .add_root_certificate(cert)
            .build()?;
        Ok(Self { client })
    }

    pub async fn poll(&self) -> anyhow::Result<ClientPollResponse> {
        let response = self
            .client
            .get(format!("{}/api/poll", CONTROL_PLANE_URL))
            .send()
            .await?
            .error_for_status()?;
        Ok(response.json().await?)
    }

    pub async fn report(&self, report: &ClientReport) -> anyhow::Result<()> {
        self.client
            .post(format!("{}/api/report", CONTROL_PLANE_URL))
            .json(report)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

