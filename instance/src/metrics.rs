use miette::IntoDiagnostic;
use opentelemetry::global;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use prometheus::{Encoder, Registry};
use tracing::{info, instrument};
use warp::{reply::Reply, Filter};

use crate::config::Config;

#[instrument("metrics", skip_all)]
pub async fn run(config: &Config, registry: Registry) -> miette::Result<()> {
    info!(
        addr = config.prometheus_addr.to_string(),
        "Started metrics server"
    );
    let route = warp::path!("metrics")
        .map(move || registry.clone())
        .then(metrics_handler);

    warp::serve(route).run(config.prometheus_addr).await;

    Ok(())
}

pub fn init_meter_provider(registry: Registry) -> miette::Result<()> {
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(registry.clone())
        .build()
        .into_diagnostic()?;
    let provider = SdkMeterProvider::builder().with_reader(exporter).build();

    global::set_meter_provider(provider.clone());
    Ok(())
}

async fn metrics_handler(registry: Registry) -> impl Reply {
    let encoder = prometheus::TextEncoder::new();

    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&registry.gather(), &mut buffer) {
        eprintln!("could not encode custom metrics: {}", e);
    };
    let res = match String::from_utf8(buffer.clone()) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("custom metrics could not be from_utf8'd: {}", e);
            String::default()
        }
    };
    buffer.clear();

    res
}
