use balius_runtime::{drivers, Runtime};
use kube_leader_election::{LeaseLock, LeaseLockParams};
use miette::{Context, IntoDiagnostic};
use operator::kube::Client;
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::Duration,
};
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use super::config::Config;

#[instrument("chainsync", skip_all)]
pub async fn run(
    config: &Config,
    runtime: Runtime,
    cancel: CancellationToken,
) -> miette::Result<()> {
    let is_leader = Arc::new(AtomicBool::new(false));

    // Run leader election as background process
    let lease = async {
        let is_leader = is_leader.clone();
        let client = Client::try_default()
            .await
            .into_diagnostic()
            .context("failed to create kube client")?;

        let holder_id = format!("shard-{}-pod-{}", config.shard, config.pod);
        let leadership = LeaseLock::new(
            client,
            &config.namespace,
            LeaseLockParams {
                holder_id,
                lease_name: config.shard.clone(),
                lease_ttl: Duration::from_secs(config.lease_ttl_seconds.unwrap_or(10)),
            },
        );

        loop {
            tokio::select! {
                result = leadership.try_acquire_or_renew() => {
                    match result {
                        Ok(ll) => is_leader.store(ll.acquired_lease, std::sync::atomic::Ordering::Relaxed),
                        Err(err) => tracing::error!("{:?}", err),
                    };
                    tokio::time::sleep(Duration::from_secs(config.lease_renew_seconds.unwrap_or(5))).await;


                }
                _ = cancel.cancelled() => {
                    tracing::warn!("received cancellation, dropping chainsync lease if hold.");
                    leadership.step_down().await.into_diagnostic().context("dropping lease")?;
                    return Ok(())
                }
            };
        }
    };

    let chainsync_driver = async {
        loop {
            if is_leader.load(std::sync::atomic::Ordering::Relaxed) {
                return tokio::select! {
                    result = drivers::chainsync::run(config.chainsync.clone(), runtime.clone(), cancel.clone()) => {
                        result.into_diagnostic()
                            .context("Running chainsync driver")

                    }
                    _ = cancel.cancelled() => {
                        tracing::warn!("received cancellation");
                        Ok(())
                    }
                };
            } else {
                tokio::time::sleep(Duration::from_secs(config.lease_renew_seconds.unwrap_or(5)))
                    .await;
            }
        }
    };

    tokio::try_join!(lease, chainsync_driver)?;
    Ok(())
}
