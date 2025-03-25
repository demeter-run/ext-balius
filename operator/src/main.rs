use dotenv::dotenv;
use std::{io, sync::Arc};
use tracing::Level;

use operator::{controller, metrics as metrics_collector, State};

#[tokio::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install default provider");
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let state = Arc::new(State::default());

    metrics_collector::run_metrics_collector(state.clone());
    metrics_collector::run_metrics_server(state.clone());

    controller::run(state.clone()).await;

    Ok(())
}
