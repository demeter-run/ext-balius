use balius_runtime::Runtime;
use futures_util::TryStreamExt;
use miette::{Context, IntoDiagnostic};
use operator::{
    kube::{
        runtime::watcher::{self, Config as ConfigWatcher, Event},
        Api, Client, ResourceExt,
    },
    BaliusWorker,
};
use serde_json::Value;
use tokio::pin;
use tracing::{error, info, instrument};

#[instrument("crdwatcher", skip_all)]
pub async fn update_runtime(runtime: Runtime) -> miette::Result<()> {
    let client = Client::try_default()
        .await
        .expect("failed to create kube client");

    let api = Api::<BaliusWorker>::all(client.clone());
    let stream = watcher::watcher(api.clone(), ConfigWatcher::default());
    pin!(stream);

    loop {
        let result = stream.try_next().await;
        match result {
            Ok(Some(Event::Init)) => {
                info!("Watcher restarted, registering workers");
            }

            Ok(Some(Event::InitApply(crd))) => {
                let name = crd.name_any();
                info!("Registering worker: {}", &name);

                match url::Url::parse(&crd.spec.url) {
                    Ok(module) => {
                        runtime
                            .register_worker_from_url(
                                &name,
                                &module,
                                Value::Object(crd.spec.config),
                            )
                            .await
                            .into_diagnostic()
                            .context("registering worker")?;
                    }
                    Err(err) => {
                        error!(
                            err = err.to_string(),
                            "Failed to parse URL for worker: {}", name
                        );
                    }
                };
            }

            Ok(Some(Event::InitDone)) => {
                info!("Workers registered.");
            }

            Ok(Some(Event::Apply(crd))) => {
                let name = crd.name_any();
                info!("Updateted worker: {}", &name);

                match url::Url::parse(&crd.spec.url) {
                    Ok(module) => {
                        runtime
                            .register_worker_from_url(
                                &name,
                                &module,
                                Value::Object(crd.spec.config),
                            )
                            .await
                            .into_diagnostic()
                            .context("registering worker")?;
                    }
                    Err(err) => {
                        error!(
                            err = err.to_string(),
                            "Failed to parse URL for worker: {}", name
                        );
                    }
                };
            }

            Ok(Some(Event::Delete(crd))) => {
                info!("Removing worker: {}", crd.name_any());
                runtime
                    .remove_worker(&crd.name_any())
                    .await
                    .into_diagnostic()
                    .context("removing worker from runtime")?;
            }

            Ok(None) => {
                error!("Empty response from crdwatcher.");
                continue;
            }
            // Unexpected error when streaming CRDs.
            Err(err) => {
                error!(error = err.to_string(), "Error consuming CRDs. Exiting");
                std::process::exit(1);
            }
        }
    }
}
