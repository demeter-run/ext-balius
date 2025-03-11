use prometheus::Registry;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Kube Error: {0}")]
    KubeError(#[source] kube::Error),

    #[error("Deserialize Error: {0}")]
    DeserializeError(#[source] serde_json::Error),

    #[error("Http Request error: {0}")]
    HttpError(String),

    #[error("Config Error: {0}")]
    ConfigError(String),
}

impl Error {
    pub fn metric_label(&self) -> String {
        format!("{self:?}").to_lowercase()
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::DeserializeError(value)
    }
}

impl From<kube::Error> for Error {
    fn from(value: kube::Error) -> Self {
        Error::KubeError(value)
    }
}

#[derive(Clone)]
pub struct State {
    registry: Registry,
    pub metrics: Metrics,
}
impl State {
    pub fn new() -> Self {
        let registry = Registry::default();
        let metrics = Metrics::default().register(&registry).unwrap();
        Self { registry, metrics }
    }

    pub fn metrics_collected(&self) -> Vec<prometheus::proto::MetricFamily> {
        self.registry.gather()
    }
}
impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

pub use k8s_openapi;
pub use kube;

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub mod controller;
pub use crate::controller::*;

pub mod metrics;
pub use metrics::*;

mod config;
pub use config::*;

mod utils;
pub use utils::*;
