use balius_runtime::sign::SignerProvider;
use balius_runtime::wit::balius::app::sign as wit;
use base64::{engine::general_purpose::STANDARD, Engine};
use miette::{Context, IntoDiagnostic};
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use vaultrs::api::transit::requests::{CreateKeyRequest, ExportKeyType, ExportVersion};
use vaultrs::api::transit::KeyType;
use vaultrs::client::{VaultClient, VaultClientSettingsBuilder};
use vaultrs::error::ClientError;
use vaultrs::token;
use vaultrs::transit::{data, key};

use super::config::Config;

pub struct VaultSigner {
    client: VaultClient,
}
impl VaultSigner {
    pub fn try_new(address: &str, token: &str) -> miette::Result<Self> {
        // Create a client
        let client = VaultClient::new(
            VaultClientSettingsBuilder::default()
                .address(address)
                .token(token)
                .verify(false)
                .build()
                .into_diagnostic()
                .context("creating vault client settings")?,
        )
        .into_diagnostic()
        .context("creating vault client")?;
        Ok(Self { client })
    }

    pub fn key_for_worker(worker_id: &str, key_name: &str) -> String {
        format!("{worker_id}-{key_name}")
    }
}

#[async_trait::async_trait]
impl SignerProvider for VaultSigner {
    async fn add_key(&mut self, worker_id: &str, key_name: String, algorithm: String) -> Vec<u8> {
        if algorithm != "ed25519" {
            panic!("Only ed25519 supported.")
        }
        // Create an encryption key using the /transit backend
        let vault_key = Self::key_for_worker(worker_id, &key_name);
        if let Err(err) = key::create(
            &self.client,
            "transit",
            &vault_key,
            Some(CreateKeyRequest::builder().key_type(KeyType::Ed25519)),
        )
        .await
        {
            tracing::error!(err =? err, "failed to create signing key");
            panic!("failed to create signing key");
        }

        let response = match key::export(
            &self.client,
            "transit",
            &vault_key,
            ExportKeyType::PublicKey,
            ExportVersion::Latest,
        )
        .await
        {
            Ok(response) => response,
            Err(err) => {
                tracing::error!(err =? err, "failed to export public key");
                panic!("failed to export public key");
            }
        };
        match response.keys.get("1") {
            Some(key) => match STANDARD.decode(key) {
                Ok(decoded) => decoded,
                Err(err) => {
                    tracing::error!(err =? err, "failed to decode public key");
                    panic!("failed to decode public key");
                }
            },
            None => {
                tracing::error!(response =? response, "failed to get public key");
                panic!("failed to get public key");
            }
        }
    }

    async fn sign_payload(
        &mut self,
        worker_id: &str,
        key_name: String,
        payload: wit::Payload,
    ) -> Result<wit::Signature, wit::SignError> {
        let vault_key = Self::key_for_worker(worker_id, &key_name);
        let response = data::sign(
            &self.client,
            "transit",
            &vault_key,
            &STANDARD.encode(payload),
            None,
        )
        .await
        .map_err(|err| match &err {
            ClientError::APIError { code: _, errors } => {
                if errors.iter().any(|x| x.contains("not found")) {
                    wit::SignError::KeyNotFound(err.to_string())
                } else {
                    wit::SignError::Internal(err.to_string())
                }
            }
            _ => wit::SignError::Internal(err.to_string()),
        })?;
        STANDARD
            .decode(response.signature.replace("vault:v1:", ""))
            .map_err(|err| wit::SignError::Internal(err.to_string()))
    }
}

#[instrument("vault-token-renewer", skip_all)]
pub async fn run(config: &Config, cancel: CancellationToken) -> miette::Result<()> {
    // Create a client
    let client = VaultClient::new(
        VaultClientSettingsBuilder::default()
            .address(&config.vault_address)
            .token(&config.vault_token)
            .verify(false)
            .build()
            .into_diagnostic()
            .context("creating vault client settings")?,
    )
    .into_diagnostic()
    .context("creating vault client")?;

    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(config.vault_token_renew_seconds)) => {
                let response = token::renew_self(
                    &client,
                    Some(config.vault_token_renew_increment.as_ref().map_or("1h", |v| v))
                ).await.into_diagnostic().context("renewing vault token")?;
                tracing::debug!(response =? response,  "vault token renewed");
            }
            _ = cancel.cancelled() => {
                tracing::warn!("received cancellation");
                return Ok(())
            }
        }
    }
}
