use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use futures_util::TryStreamExt;

use operator::{
    kube::{
        runtime::watcher::{self, Config as ConfigWatcher, Event},
        Api, Client, ResourceExt,
    },
    BaliusWorker,
};
use pingora::{server::ShutdownWatch, services::background::BackgroundService};
use tokio::pin;
use tracing::{error, info};

use crate::{Consumer, State};

pub struct AuthBackgroundService {
    state: Arc<State>,
}
impl AuthBackgroundService {
    pub fn new(state: Arc<State>) -> Self {
        Self { state }
    }
}

#[async_trait]
impl BackgroundService for AuthBackgroundService {
    async fn start(&self, mut _shutdown: ShutdownWatch) {
        let client = Client::try_default()
            .await
            .expect("failed to create kube client");

        let api = Api::<BaliusWorker>::all(client.clone());
        let stream = watcher::watcher(api.clone(), ConfigWatcher::default());
        pin!(stream);

        loop {
            let result = stream.try_next().await;
            match result {
                // Stream restart, also run on startup.
                Ok(Some(Event::Init)) => {
                    info!("auth: Watcher restarted, reseting consumers");
                    *self.state.consumers.write().await = HashMap::new();
                    self.state.limiter.write().await.clear();
                }
                // Add port from restart.
                Ok(Some(Event::InitApply(crd))) => match crd.status {
                    Some(_) => {
                        info!("auth: Adding consumer: {}", crd.name_any());
                        let consumer = Consumer::from(&crd);
                        self.state.limiter.write().await.remove(&consumer.key);
                        self.state
                            .consumers
                            .write()
                            .await
                            .insert(consumer.key.clone(), consumer);
                    }
                    None => {
                        // New ports are created without status. When the status is added, a new
                        // Applied event is triggered.
                        info!("auth: New port created: {}", crd.name_any());
                    }
                },
                // Stream restart, also run on startup.
                Ok(Some(Event::InitDone)) => {
                    info!("auth: Watcher restart completed");
                }
                // New port created or updated.
                Ok(Some(Event::Apply(crd))) => match crd.status {
                    Some(_) => {
                        info!("auth: Updating consumer: {}", crd.name_any());
                        let consumer = Consumer::from(&crd);
                        self.state.limiter.write().await.remove(&consumer.key);
                        self.state
                            .consumers
                            .write()
                            .await
                            .insert(consumer.key.clone(), consumer);
                    }
                    None => {
                        // New ports are created without status. When the status is added, a new
                        // Applied event is triggered.
                        info!("auth: New port created: {}", crd.name_any());
                    }
                },
                // Port deleted.
                Ok(Some(Event::Delete(crd))) => {
                    info!(
                        "auth: Port deleted, removing from state: {}",
                        crd.name_any()
                    );
                    let consumer = Consumer::from(&crd);
                    self.state.consumers.write().await.remove(&consumer.key);
                    self.state.limiter.write().await.remove(&consumer.key);
                }
                // Empty response from stream. Should never happen.
                Ok(None) => {
                    error!("auth: Empty response from watcher.");
                    continue;
                }
                // Unexpected error when streaming CRDs.
                Err(err) => {
                    error!(error = err.to_string(), "auth: Failed to update crds.");
                    std::process::exit(1);
                }
            }
        }
    }
}
