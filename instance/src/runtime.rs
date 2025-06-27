use aws_sdk_s3::Client as S3Client;
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
use url::Url;

use crate::config::Config;

async fn download_s3_object(s3_url: &str) -> miette::Result<Vec<u8>> {
    let url = Url::parse(s3_url)
        .into_diagnostic()
        .context("Failed ot parse url")?;

    let bucket = match url.host_str() {
        Some(url) => url,
        None => miette::bail!("Invalid bucket"),
    };
    let key = url.path().trim_start_matches('/');

    let config = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
    let client = S3Client::new(&config);

    info!(bucket = bucket, key = key, "Donwloading object...");
    let resp = client
        .get_object()
        .bucket(bucket)
        .key(key)
        .send()
        .await
        .into_diagnostic()
        .context("Failed to get from s3")?;

    let body = resp
        .body
        .collect()
        .await
        .into_diagnostic()
        .context("Failed to download from s3")?;
    Ok(body.to_vec())
}

#[instrument("crdwatcher", skip_all)]
pub async fn update_runtime(config: &Config, runtime: Runtime) -> miette::Result<()> {
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
                if crd.spec.network == config.network {
                    info!("Registering worker: {}", &name);
                    match download_s3_object(&crd.spec.url).await {
                        Ok(bytes) => {
                            if let Err(err) = runtime
                                .register_worker(&name, &bytes, Value::Object(crd.spec.config))
                                .await
                            {
                                error!(err =? err, worker = name, "Error registering worker");
                            }
                        }
                        Err(err) => {
                            error!(err = err.to_string(), "Failed to register worker: {}", name);
                        }
                    };
                } else {
                    info!("New CRD doesn't match network: {}", &name);
                }
            }

            Ok(Some(Event::InitDone)) => {
                info!("Workers registered.");
            }

            Ok(Some(Event::Apply(crd))) => {
                let name = crd.name_any();

                if crd.spec.network == config.network {
                    info!("Applying worker: {}", &name);
                    match url::Url::parse(&crd.spec.url) {
                        Ok(module) => {
                            if let Err(err) = runtime
                                .register_worker_from_url(
                                    &name,
                                    &module,
                                    Value::Object(crd.spec.config),
                                )
                                .await
                            {
                                error!(
                                    err =? err,
                                    worker = name,
                                    "Error registering worker"
                                );
                            }
                        }
                        Err(err) => {
                            error!(
                                err = err.to_string(),
                                "Failed to parse URL for worker: {}", name
                            );
                        }
                    };
                } else {
                    info!("Applied CRD doesn't match network: {}", &name);
                }
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
