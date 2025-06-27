use std::net::SocketAddr;

use balius_runtime::{drivers, ledgers};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Config {
    pub network: String,
    pub connection: String,
    pub max_pool_size: Option<u32>,
    pub namespace: String,
    pub pod: String,
    pub shard: String,
    pub lease_ttl_seconds: Option<u64>,
    pub lease_renew_seconds: Option<u64>,
    pub rpc: drivers::jsonrpc::Config,
    pub ledger: ledgers::u5c::Config,
    pub chainsync: drivers::chainsync::Config,
    pub prometheus_addr: SocketAddr,
    pub vault_address: String,
    pub vault_token: String,
    pub vault_token_renew_seconds: u64,
    pub vault_token_renew_increment: Option<String>,
}

pub fn load_config<T>(explicit_file: &Option<std::path::PathBuf>) -> Result<T, config::ConfigError>
where
    T: DeserializeOwned,
{
    let mut s = config::Config::builder();

    // our base config will always be in /etc/dolos
    if let Ok(cfg) = std::env::var("BALIUSD_CONFIG") {
        s = s.add_source(config::File::with_name(&cfg).required(false));
    }

    // but we can override it by having a file in the working dir
    s = s.add_source(config::File::with_name("baliusd.toml").required(false));

    // if an explicit file was passed, then we load it as mandatory
    if let Some(explicit) = explicit_file.as_ref().and_then(|x| x.to_str()) {
        s = s.add_source(config::File::with_name(explicit).required(true));
    }

    // finally, we use env vars to make some last-step overrides
    s = s.add_source(config::Environment::with_prefix("BALIUSD").separator("_"));

    s.build()?.try_deserialize()
}
