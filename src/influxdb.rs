use reqwest::Client;
use thiserror::Error;

use crate::config::Config;
use crate::scale::ScaleAdvertisement;

#[derive(Debug, Error)]
pub enum InfluxError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("InfluxDB returned error status {0}: {1}")]
    BadStatus(u16, String),
}

pub struct InfluxClient {
    client: Client,
    write_url: String,
    auth_header: String,
}

impl InfluxClient {
    pub fn new(config: &Config) -> Self {
        let write_url = format!(
            "{}/api/v2/write?org={}&bucket={}&precision=s",
            config.influxdb_url, config.influxdb_org, config.influxdb_bucket
        );
        let auth_header = format!("Token {}", config.influxdb_token);

        InfluxClient {
            client: Client::new(),
            write_url,
            auth_header,
        }
    }

    pub async fn write_weight(&self, adv: &ScaleAdvertisement) -> Result<(), InfluxError> {
        let ts = time::OffsetDateTime::now_utc().unix_timestamp();
        let body = format!("weight,device=mi_scale_2 value={:.3} {}", adv.weight_kg, ts);

        let response = self
            .client
            .post(&self.write_url)
            .header("Authorization", &self.auth_header)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(InfluxError::BadStatus(status.as_u16(), body));
        }

        Ok(())
    }
}
