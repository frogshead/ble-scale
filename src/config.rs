use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingVar(String),
}

pub struct Config {
    pub scale_address: String,
    pub influxdb_url: String,
    pub influxdb_token: String,
    pub influxdb_org: String,
    pub influxdb_bucket: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        fn require(name: &str) -> Result<String, ConfigError> {
            dotenv::var(name).map_err(|_| ConfigError::MissingVar(name.to_string()))
        }

        Ok(Config {
            scale_address: require("ADDRESS")?,
            influxdb_url: require("INFLUXDB_URL")?,
            influxdb_token: require("INFLUXDB_TOKEN")?,
            influxdb_org: require("INFLUXDB_ORG")?,
            influxdb_bucket: require("INFLUXDB_BUCKET")?,
        })
    }
}
