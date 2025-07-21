use async_trait::async_trait;
use lazy_static::lazy_static;
use pingora::http::Method;
use pingora::Result;
use pingora::{
    http::ResponseHeader,
    proxy::{ProxyHttp, Session},
    upstreams::peer::HttpPeer,
};
use pingora_limits::rate::Rate;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

use crate::config::Config;
use crate::{Consumer, State, Tier};

static DMTR_API_KEY: &str = "dmtr-api-key";

lazy_static! {
    static ref LEGACY_NETWORKS: HashMap<&'static str, String> = {
        let mut m = HashMap::new();
        m.insert("mainnet", "cardano-mainnet".into());
        m.insert("preprod", "cardano-preprod".into());
        m.insert("preview", "cardano-preview".into());
        m
    };
}

pub fn handle_legacy_networks(network: &str) -> String {
    let default = network.to_string();
    LEGACY_NETWORKS.get(network).unwrap_or(&default).to_string()
}

pub struct BaliusProxy {
    state: Arc<State>,
    config: Arc<Config>,
    host_regex: Regex,
}
impl BaliusProxy {
    pub fn new(state: Arc<State>, config: Arc<Config>) -> Self {
        let host_regex = Regex::new(r"([baliusworker]?[\w\d-]+)?\.?.+").unwrap();

        Self {
            state,
            config,
            host_regex,
        }
    }

    async fn has_limiter(&self, consumer: &Consumer) -> bool {
        let rate_limiter_map = self.state.limiter.read().await;
        rate_limiter_map.get(&consumer.key).is_some()
    }

    async fn add_limiter(&self, consumer: &Consumer, tier: &Tier) {
        let rates = tier
            .rates
            .iter()
            .map(|r| (r.clone(), Rate::new(r.interval)))
            .collect();

        self.state
            .limiter
            .write()
            .await
            .insert(consumer.key.clone(), rates);
    }

    async fn limiter(&self, consumer: &Consumer) -> Result<bool> {
        let tiers = self.state.tiers.read().await.clone();
        let tier = tiers.get(&consumer.tier);
        if tier.is_none() {
            return Ok(true);
        }
        let tier = tier.unwrap();

        if !self.has_limiter(consumer).await {
            self.add_limiter(consumer, tier).await;
        }

        let rate_limiter_map = self.state.limiter.read().await;
        let rates = rate_limiter_map.get(&consumer.key).unwrap();

        if rates
            .iter()
            .any(|(t, r)| r.observe(&consumer.key, 1) > t.limit)
        {
            return Ok(true);
        }

        Ok(false)
    }

    fn extract_key(&self, session: &Session) -> String {
        let host = session
            .get_header("host")
            .map(|v| v.to_str().unwrap())
            .unwrap();

        let captures = self.host_regex.captures(host).unwrap();
        let key = session
            .get_header(DMTR_API_KEY)
            .and_then(|v| v.to_str().ok())
            .or_else(|| captures.get(1).map(|v| v.as_str()))
            .unwrap_or_default();

        key.to_string()
    }

    async fn respond_options(&self, session: &mut Session, ctx: &mut Context) {
        ctx.is_health_request = true;
        session.set_keepalive(None);
        let mut response = ResponseHeader::build(200, None).unwrap();
        response
            .insert_header("Access-Control-Allow-Origin", "*")
            .unwrap();
        response
            .insert_header("Access-Control-Allow-Methods", "POST")
            .unwrap();
        response
            .insert_header("Access-Control-Allow-Headers", "Content-Type,dmtr-api-key")
            .unwrap();
        session
            .write_response_header(Box::new(response), true)
            .await
            .unwrap();
    }

    async fn respond_health(&self, session: &mut Session, ctx: &mut Context) {
        ctx.is_health_request = true;
        session.set_keepalive(None);
        session
            .write_response_body(Some("OK".into()), true)
            .await
            .unwrap();
        let header = Box::new(ResponseHeader::build(200, None).unwrap());
        session.write_response_header(header, true).await.unwrap();
    }
}

#[derive(Debug, Default)]
pub struct Context {
    instance: String,
    consumer: Consumer,
    is_health_request: bool,
}

#[async_trait]
impl ProxyHttp for BaliusProxy {
    type CTX = Context;
    fn new_ctx(&self) -> Self::CTX {
        Context::default()
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool>
    where
        Self::CTX: Send + Sync,
    {
        if session.req_header().method == Method::OPTIONS {
            self.respond_options(session, ctx).await;
            return Ok(true);
        }

        let path = session.req_header().uri.path();
        if path == self.config.health_endpoint {
            self.respond_health(session, ctx).await;
            return Ok(true);
        }

        let key = self.extract_key(session);
        let consumer = self.state.get_consumer(&key).await;

        if consumer.is_none() {
            session.respond_error(401).await?;
            return Ok(true);
        }

        ctx.consumer = consumer.unwrap();
        ctx.instance = format!(
            "balius-{}.{}:{}",
            handle_legacy_networks(&ctx.consumer.network),
            self.config.balius_dns,
            self.config.balius_port
        );

        if self.limiter(&ctx.consumer).await? {
            session.respond_error(429).await?;
            return Ok(true);
        }

        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        let http_peer = HttpPeer::new(&ctx.instance, false, String::default());
        Ok(Box::new(http_peer))
    }

    async fn logging(
        &self,
        session: &mut Session,
        _e: Option<&pingora::Error>,
        ctx: &mut Self::CTX,
    ) {
        if !ctx.is_health_request {
            let response_code = session
                .response_written()
                .map_or(0, |resp| resp.status.as_u16());

            info!(
                "{} response code: {response_code}",
                self.request_summary(session, ctx)
            );

            self.state.metrics.inc_http_total_request(
                &ctx.consumer,
                &self.config.proxy_namespace,
                &ctx.instance,
                &response_code,
            );
        }
    }
}
