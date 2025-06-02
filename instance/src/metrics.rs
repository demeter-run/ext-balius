use std::sync::Arc;

use miette::IntoDiagnostic;
use prometheus::{opts, Encoder, IntCounterVec, Registry};
use tracing::{error, info, instrument};
use warp::{reply::Reply, Filter};

use crate::config::Config;

#[derive(Clone)]
pub struct Metrics {
    pub registry: Registry,
    pub kvget: IntCounterVec,
    pub kvset: IntCounterVec,
    pub kvlist: IntCounterVec,
    pub log: IntCounterVec,
    pub requests: IntCounterVec,
}

impl Metrics {
    pub fn try_new() -> miette::Result<Self> {
        let kvget =
            IntCounterVec::new(opts!("kvget", "Amount of gets to KV",), &["worker"]).unwrap();
        let kvset =
            IntCounterVec::new(opts!("kvset", "Amount of sets to KV",), &["worker"]).unwrap();
        let kvlist =
            IntCounterVec::new(opts!("kvlist", "Amount of lists to KV",), &["worker"]).unwrap();
        let log = IntCounterVec::new(opts!("log", "Amount of log writes",), &["worker", "level"])
            .unwrap();
        let requests = IntCounterVec::new(
            opts!("requests", "Amount of requests to jsonrpc server",),
            &["worker", "method", "code"],
        )
        .unwrap();

        let registry = Registry::default();
        registry
            .register(Box::new(kvget.clone()))
            .into_diagnostic()?;
        registry
            .register(Box::new(kvset.clone()))
            .into_diagnostic()?;
        registry
            .register(Box::new(kvlist.clone()))
            .into_diagnostic()?;
        registry.register(Box::new(log.clone())).into_diagnostic()?;
        registry
            .register(Box::new(requests.clone()))
            .into_diagnostic()?;

        Ok(Metrics {
            registry,
            kvget,
            kvset,
            kvlist,
            log,
            requests,
        })
    }

    pub fn kvget(&self, worker: &str) {
        self.kvget.with_label_values(&[worker]).inc()
    }

    pub fn kvset(&self, worker: &str) {
        self.kvset.with_label_values(&[worker]).inc()
    }

    pub fn kvlist(&self, worker: &str) {
        self.kvlist.with_label_values(&[worker]).inc()
    }

    pub fn log(&self, worker: &str, level: &str) {
        self.log.with_label_values(&[worker, level]).inc()
    }

    pub fn requests(&self, worker: &str, method: &str, code: usize) {
        self.requests
            .with_label_values(&[worker, method, &code.to_string()])
            .inc()
    }
}

async fn metrics_handler(metrics: Arc<Metrics>) -> impl Reply {
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&metrics.registry.gather(), &mut buffer) {
        eprintln!("could not encode custom metrics: {}", e);
    };
    let res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            error!("custom metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    res
}

#[instrument("metrics", skip_all)]
pub async fn run(config: &Config, metrics: Arc<Metrics>) -> miette::Result<()> {
    let route = warp::path!("metrics")
        .map(move || metrics.clone())
        .then(metrics_handler);

    info!(
        addr = config.prometheus_addr.to_string(),
        "Started metrics server"
    );
    warp::serve(route).run(config.prometheus_addr).await;

    Ok(())
}
