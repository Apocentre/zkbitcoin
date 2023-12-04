//! Code related to the Bitcoind JSON RPC interface.
//! It heavily relies on the jsonrpc and bitcoincore_rpc crates (and its dependencies).
//! It does not directly make use of these crates due to some issues (loss of information when getting 500 errors from bitcoind).

use anyhow::{Context, Result};
use base64::{engine::general_purpose, Engine};
use bitcoin::{Transaction, Txid};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use std::time::Duration;

use crate::constants::BITCOIN_JSON_RPC_VERSION;

/// Timeout (in seconds) for json rpc requests.
const JSON_RPC_TIMEOUT: u64 = 2;

//
// Context
//

#[derive(Default)]
pub struct RpcCtx {
    pub version: Option<&'static str>,
    pub wallet: Option<String>,
    pub address: Option<String>,
    pub auth: Option<String>,
}

impl RpcCtx {
    pub fn new(
        version: Option<&'static str>,
        wallet: Option<String>,
        address: Option<String>,
        auth: Option<String>,
    ) -> Self {
        let ctx = Self {
            version,
            wallet,
            address,
            auth,
        };

        println!("- using RPC node at address {}", ctx.address());

        if ctx.auth().is_some() {
            println!("- using given RPC credentials");
        } else {
            println!("- using no RPC credentials");
        }

        if let Some(wallet) = ctx.wallet() {
            println!("- using wallet {wallet}");
        } else {
            println!("- using default wallet");
        }

        ctx
    }

    pub fn wallet(&self) -> Option<&str> {
        self.wallet.as_deref()
    }

    pub fn address(&self) -> &str {
        self.address.as_deref().unwrap_or("http://127.0.0.1:18331")
    }

    pub fn auth(&self) -> Option<&str> {
        self.auth.as_deref()
        /*.map(|s| {
            s.split('.')
                .map(str::to_string)
                .collect_tuple()
                .expect("auth was incorrectly passed (expected `user:pw`)")
        })*/
    }

    pub fn for_testing() -> Self {
        Self {
            version: Some(BITCOIN_JSON_RPC_VERSION),
            wallet: Some("mywallet".to_string()),
            address: Some(JSON_RPC_ENDPOINT.to_string()),
            auth: Some(JSON_RPC_AUTH.to_string()),
        }
    }
}

//
// Main JSON RPC request function
//

/// The endpoint for our bitcoind full node.
pub const JSON_RPC_ENDPOINT: &str = "http://146.190.33.39:18331";

/// The RPC authentication our bitcoind node uses (user + password).
// TODO: obviously we're using poor's man authentication :))
const JSON_RPC_AUTH: &str = "root:hellohello";

/// Implements a JSON RPC request to the bitcoind node.
/// Following the [JSON RPC 1.0 spec](https://www.jsonrpc.org/specification_v1).
pub async fn json_rpc_request<'a>(
    ctx: &RpcCtx,
    method: &'static str,
    params: &'a [Box<serde_json::value::RawValue>],
) -> Result<String, reqwest::Error> {
    // create the request
    let request = bitcoincore_rpc::jsonrpc::Request::<'a> {
        // bitcoind doesn't seem to support anything else but json rpc 1.0
        jsonrpc: ctx.version,
        // I don't think that field is useful (https://www.jsonrpc.org/specification_v1)
        id: serde_json::Value::String("whatevs".to_string()),
        method,
        params,
    };

    let mut headers = HeaderMap::new();
    if let Some(auth) = ctx.auth() {
        let user_n_pw = general_purpose::STANDARD.encode(auth);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {}", user_n_pw)).unwrap(),
        );
    }

    let body = serde_json::to_string(&request).unwrap();

    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(JSON_RPC_TIMEOUT))
        .build()?;

    let endpoint = ctx.address();
    let url = match &ctx.wallet {
        Some(wallet) => format!("{}/wallet/{}", endpoint, wallet),
        None => endpoint.to_string(),
    };

    {
        let body = serde_json::to_string_pretty(&request).unwrap();
        println!("- sending request to {url} with body: {body}");
    }

    let response = client
        .post(url)
        .header(CONTENT_TYPE, "application/json")
        .body(body)
        .send()
        .await?;
    println!("- status_code: {:?}", &response.status().as_u16());
    response.text().await
}

//
// Helpers around useful Bitcoin RPC functions
//

pub enum TransactionOrHex<'a> {
    Hex(String),
    Transaction(&'a Transaction),
}

pub async fn fund_raw_transaction<'a>(
    ctx: &RpcCtx,
    tx: TransactionOrHex<'a>,
) -> Result<(String, Transaction)> {
    let tx_hex = match tx {
        TransactionOrHex::Hex(hex) => hex,
        TransactionOrHex::Transaction(tx) => bitcoin::consensus::encode::serialize_hex(tx),
    };

    let response = json_rpc_request(
        ctx,
        "fundrawtransaction",
        &[serde_json::value::to_raw_value(&serde_json::Value::String(tx_hex)).unwrap()],
    )
    .await
    .context("fundrawtransaction error")?;

    // TODO: get rid of unwrap in here
    let response: bitcoincore_rpc::jsonrpc::Response = serde_json::from_str(&response).unwrap();
    let parsed: bitcoincore_rpc::json::FundRawTransactionResult = response.result().unwrap();
    let tx: Transaction = bitcoin::consensus::encode::deserialize(&parsed.hex).unwrap();
    let actual_hex = hex::encode(&parsed.hex);
    //println!("- funded tx: {tx:?}");
    println!("- funded tx (in hex): {actual_hex}");

    Ok((actual_hex, tx))
}

pub async fn sign_transaction<'a>(
    ctx: &RpcCtx,
    tx: TransactionOrHex<'a>,
) -> Result<(String, Transaction)> {
    let tx_hex = match tx {
        TransactionOrHex::Hex(hex) => hex,
        TransactionOrHex::Transaction(tx) => bitcoin::consensus::encode::serialize_hex(tx),
    };

    let response = json_rpc_request(
        ctx,
        "signrawtransactionwithwallet",
        &[serde_json::value::to_raw_value(&serde_json::Value::String(tx_hex)).unwrap()],
    )
    .await
    .context("signrawtransactionwithwallet error")?;

    // TODO: get rid of unwrap in here
    let response: bitcoincore_rpc::jsonrpc::Response = serde_json::from_str(&response).unwrap();
    let parsed: bitcoincore_rpc::json::SignRawTransactionResult = response.result().unwrap();
    let tx: Transaction = bitcoin::consensus::encode::deserialize(&parsed.hex).unwrap();
    let actual_hex = hex::encode(&parsed.hex);
    //println!("- signed tx: {tx:?}");
    println!("- signed tx (in hex): {actual_hex}");

    Ok((actual_hex, tx))
}

pub async fn send_raw_transaction<'a>(ctx: &RpcCtx, tx: TransactionOrHex<'a>) -> Result<Txid> {
    let tx_hex = match tx {
        TransactionOrHex::Hex(hex) => hex,
        TransactionOrHex::Transaction(tx) => bitcoin::consensus::encode::serialize_hex(tx),
    };

    let response = json_rpc_request(
        ctx,
        "sendrawtransaction",
        &[serde_json::value::to_raw_value(&serde_json::Value::String(tx_hex)).unwrap()],
    )
    .await
    .context("sendrawtransaction error")?;

    // TODO: get rid of unwrap in here
    let response: bitcoincore_rpc::jsonrpc::Response = serde_json::from_str(&response).unwrap();
    let txid: bitcoin::Txid = response.result().unwrap();
    println!("- txid broadcast to the network: {txid}");
    println!("- on an explorer: https://blockstream.info/testnet/tx/{txid}");

    Ok(txid)
}
