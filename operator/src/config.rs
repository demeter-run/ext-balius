use lazy_static::lazy_static;
use std::{env, time::Duration};

lazy_static! {
    static ref CONTROLLER_CONFIG: Config = Config::from_env();
}

pub fn get_config() -> &'static Config {
    &CONTROLLER_CONFIG
}

#[derive(Debug, Clone)]
pub struct Config {
    pub extension_domain: String,
    pub metrics_delay: Duration,
    pub prometheus_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            extension_domain: env::var("EXTENSION_DOMAIN")
                .unwrap_or("balius-m1.demeter.run".into()),
            metrics_delay: Duration::from_secs(
                std::env::var("METRICS_DELAY")
                    .expect("METRICS_DELAY must be set")
                    .parse::<u64>()
                    .expect("METRICS_DELAY must be a number"),
            ),
            prometheus_url: env::var("PROMETHEUS_URL").expect("PROMETHEUS_URL must be set"),
        }
    }
}
