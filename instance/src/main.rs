use balius_runtime::{drivers, ledgers, Runtime, Store};
use miette::{Context as _, IntoDiagnostic as _};
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn, Level};

mod config;
mod runtime;

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
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    let config: config::Config = config::load_config(&None)
        .into_diagnostic()
        .context("loading config")?;

    let store = Store::open("baliusd.db", None)
        .into_diagnostic()
        .context("opening store")?;

    let ledger = ledgers::u5c::Ledger::new(config.ledger.clone())
        .await
        .into_diagnostic()
        .context("setting up ledger")?;

    let runtime = Runtime::builder(store)
        .with_ledger(ledger.into())
        .with_kv(balius_runtime::kv::Kv::Mock)
        .build()
        .into_diagnostic()
        .context("setting up runtime")?;

    let cancel = hook_exit_token();

    let jsonrpc_server = async {
        balius_runtime::drivers::jsonrpc::serve(config.rpc.clone(), runtime.clone(), cancel.clone())
            .await
            .into_diagnostic()
            .context("Running JsonRPC server")
    };

    let chainsync_driver = async {
        drivers::chainsync::run(config.chainsync.clone(), runtime.clone(), cancel.clone())
            .await
            .into_diagnostic()
            .context("Running chainsync driver")
    };

    let runtime_update = async { runtime::update_runtime(runtime.clone()).await };

    tokio::try_join!(jsonrpc_server, chainsync_driver, runtime_update)?;
    Ok(())
}
