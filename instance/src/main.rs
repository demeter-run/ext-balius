use balius_runtime::{ledgers, Runtime, Store};
use kv::PostgresKv;
use logging::PostgresLogger;
use metrics::init_meter_provider;
use miette::{Context, IntoDiagnostic as _};
use prometheus::Registry;
use runtime::FailedWorkers;
use signer::VaultSigner;
use std::{str::FromStr, sync::Arc, time::Duration};
use store::PostgresStore;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn, Level};

mod chainsync;
mod config;
mod kv;
mod logging;
mod metrics;
mod runtime;
mod server;
mod signer;
mod store;
mod utils;

async fn wait_for_exit_signal() {
    let mut sigterm =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).unwrap();
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            warn!("SIGINT detected");
        }
        _ = sigterm.recv() => {
            warn!("SIGTERM detected");
        }
    };
}

pub fn hook_exit_token() -> CancellationToken {
    let cancel = CancellationToken::new();

    let cancel2 = cancel.clone();
    tokio::spawn(async move {
        wait_for_exit_signal().await;
        debug!("notifying exit");
        cancel2.cancel();
    });

    cancel
}

#[tokio::main]
async fn main() -> miette::Result<()> {
    dotenv::dotenv().ok();

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install default provider");
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();
    let registry = Registry::new();
    init_meter_provider(registry.clone())?;

    let config: config::Config = config::load_config(&None)
        .into_diagnostic()
        .context("loading config")?;

    let pg_mgr = bb8_postgres::PostgresConnectionManager::new(
        tokio_postgres::config::Config::from_str(&config.connection)
            .into_diagnostic()
            .context("failed to parse connection")?,
        tokio_postgres::NoTls,
    );

    let pool = bb8::Pool::builder()
        .max_size(config.max_pool_size.unwrap_or(15))
        .build(pg_mgr)
        .await
        .into_diagnostic()
        .context("failed to build pool")?;

    let store = Store::Custom(Arc::new(Mutex::new(PostgresStore::new(
        &pool,
        &config.shard,
    ))));

    let ledger = ledgers::u5c::Ledger::new(&config.ledger)
        .await
        .into_diagnostic()
        .context("setting up ledger")?;

    let failed = FailedWorkers::default();
    let runtime = Runtime::builder(store)
        .with_ledger(ledger.into())
        .with_signer(balius_runtime::sign::Signer::Custom(Arc::new(Mutex::new(
            VaultSigner::try_new(&config.vault_address, &config.vault_token)?,
        ))))
        .with_kv(balius_runtime::kv::Kv::Custom(Arc::new(Mutex::new(
            PostgresKv::from(&pool),
        ))))
        .with_logger(balius_runtime::logging::Logger::Custom(Arc::new(
            Mutex::new(PostgresLogger::from(&pool)),
        )))
        .with_http(balius_runtime::http::Http::Reqwest(
            reqwest::Client::builder()
                .timeout(Duration::from_secs(
                    config.http_client_timeout.unwrap_or(10),
                ))
                .build()
                .expect("failed to build http client"),
        ))
        .build()
        .into_diagnostic()
        .context("setting up runtime")?;

    let cancel = hook_exit_token();

    let jsonrpc_server = async {
        server::serve(
            config.rpc.clone(),
            runtime.clone(),
            failed.clone(),
            cancel.clone(),
        )
        .await
        .into_diagnostic()
        .context("Running JsonRPC server")
    };

    let token_renewer = signer::run(&config, cancel.clone());
    let chainsync_driver = chainsync::run(&config, runtime.clone(), cancel.clone());

    let runtime_update = async {
        tokio::select! {
            _ = runtime::update_runtime(&config, runtime.clone(), failed.clone()) => {

            }
            _ = cancel.cancelled() => {
                warn!("recieved cancellation");
            }
        };
        Ok(())
    };

    let metrics_server = async {
        tokio::select! {
            _ = metrics::run(&config, registry.clone()) => {

            }
            _ = cancel.cancelled() => {
                warn!("received cancellation, shutting down metrics server");
            }
        };
        Ok(())
    };

    tokio::try_join!(
        jsonrpc_server,
        chainsync_driver,
        runtime_update,
        metrics_server,
        token_renewer
    )?;
    Ok(())
}
