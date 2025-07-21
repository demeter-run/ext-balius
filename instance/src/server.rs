use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};
use warp::Filter as _;

use balius_runtime::{wit, Error, Runtime};

use crate::runtime::FailedWorkers;

#[derive(Deserialize)]
struct Request {
    pub id: Option<String>,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn parse_request(body: serde_json::Value) -> Result<Request, ErrorResponse> {
    match serde_json::from_value(body) {
        Ok(x) => Ok(x),
        Err(x) => Err(ErrorResponse {
            error: x.to_string(),
        }),
    }
}

pub async fn handle_request(
    state: (Runtime, FailedWorkers),
    worker: String,
    body: serde_json::Value,
) -> warp::reply::Json {
    let (runtime, failed) = state;

    if let Some(reason) = failed.read(&worker).await {
        return warp::reply::json(&ErrorResponse {
            error: format!("failed to load into runtime: {reason}"),
        });
    }

    let request = match parse_request(body) {
        Ok(x) => x,
        Err(err) => return warp::reply::json(&err),
    };

    debug!(
        worker,
        id = request.id,
        method = request.method,
        "handling request"
    );

    let params = serde_json::to_vec(&request.params).unwrap();

    let reply = runtime
        .handle_request(&worker, &request.method, params)
        .await;

    match reply {
        Ok(x) => {
            debug!(worker, id = request.id, "request successful");

            let x = match x {
                wit::Response::Acknowledge => json!({}),
                wit::Response::Json(x) => serde_json::from_slice(&x).unwrap(),
                wit::Response::Cbor(x) => json!({ "cbor": hex::encode(x) }),
                wit::Response::PartialTx(x) => json!({ "tx": hex::encode(x) }),
            };

            warp::reply::json(&x)
        }
        Err(err) => {
            error!(worker, id = request.id, "request failed");
            warp::reply::json(&ErrorResponse {
                error: err.to_string(),
            })
        }
    }
}

pub async fn serve(
    config: balius_runtime::drivers::jsonrpc::Config,
    runtime: Runtime,
    failed: FailedWorkers,
    cancel: CancellationToken,
) -> Result<(), Error> {
    let filter = warp::any()
        .map(move || (runtime.clone(), failed.clone()))
        .and(warp::path::param())
        .and(warp::post())
        .and(warp::body::json())
        .then(handle_request)
        .with(
            warp::cors()
                .allow_any_origin()
                .allow_method("POST")
                .allow_method("OPTIONS")
                .allow_headers(vec!["content-type", "dmtr-api-key"])
                .build(),
        )
        .with(warp::log("cors test"));

    let address: SocketAddr = config
        .listen_address
        .parse()
        .map_err(|x: std::net::AddrParseError| Error::Config(x.to_string()))?;

    let (addr, server) =
        warp::serve(filter).bind_with_graceful_shutdown(address, cancel.cancelled_owned());

    tracing::info!(%addr, "Json-RPC server listening");

    server.await;

    Ok(())
}
