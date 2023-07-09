use std::process::Command;
use std::time::Duration;
use std::{sync::Arc, thread};
// use tokio::sync::Mutex;
use std::sync::Mutex;

use axum::extract::Extension;
use axum::{routing::post, Router};
use env_logger::Env;
use ethers::providers::{Http, Provider};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;
use tower_http::add_extension::AddExtensionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use zkevm::prover::{AggCircuitProof, Prover};
use zkevm_prover::utils;
use tokio::task;

// instead
#[derive(Serialize, Deserialize, Debug)]
struct ProveRequest {
    block_num: u64,
    rpc: String,
}

/**
 * Start Server.
 * this will start an HTTP server, to process zk prove requests.
 */
#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    task::spawn(prove_for_trace());

    let service = Router::new()
        .route("/prove_block", post(download_trace))
        .layer(AddExtensionLayer::new(Arc::clone(&task_queue)))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // run it with hyper on localhost:3030
    axum::Server::bind(&"127.0.0.1:3030".parse().unwrap())
        .serve(service.into_make_service())
        .await
        .unwrap();

    // let req_param =String::from("{\"block_num\":4,\"rpc\":\"127.0.0.1:8569\"}");
}

/**
 * Prove service.
 * Handle Prove Request, Use Layer2's rpc address and block number to fetch trace and generate zk proof.
 */
async fn download_trace(param: String) -> String {
    //fetch block trace
    let prove_request: ProveRequest = serde_json::from_str(param.as_str()).unwrap();
    let provider = Provider::<Http>::try_from(prove_request.rpc.clone())
        .expect("failed to initialize ethers Provider");
    let block_traces = utils::get_block_traces_by_number
    (&provider, prove_request.block_num, prove_request.block_num + 1,).await;
    if block_traces.is_none(){
        String::from("fetch block trace fail");
    }
    //save proof
    let mut params_file = File::create("trace_path").unwrap();
    params_file.write_all(serde_json::to_string(&block_traces).unwrap().as_bytes());
    log::info!("save traces successfully!");
    String::from("success")
}

async fn prove_for_trace() {
    let mut prover_bin = Command::new("./target/release/prove").spawn().unwrap();
    prover_bin.wait().unwrap();
    println!("prover_bin end");
}


