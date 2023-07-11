use std::process::Command;
use std::time::Duration;
use std::{sync::Arc, thread};
// use tokio::sync::Mutex;
use std::io::{BufRead, BufReader, Write};
use std::sync::Mutex;

use axum::extract::Extension;
use axum::{routing::post, Router};
use env_logger::Env;
use ethers::providers::{Http, Provider};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use tokio::task;
use tower_http::add_extension::AddExtensionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use zkevm_prover::utils;

// instead
#[derive(Serialize, Deserialize, Debug)]
struct ProveRequest {
    block_num: u64,
    rpc: String,
}

const PROVER_FOR_TRACES: &'static str = "target/release/prover";
const FS_TRACES_PROCESSING: &'static str = "traces/processing";
const FS_TRACES_PROVED: &'static str = "traces/proved";
const FS_PROOF: &'static str = "proof";

/**
 * Start Server.
 * this will start an HTTP server, to process zk prove requests.
 */
#[tokio::main]
async fn main() {
    //prepare environment
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    fs::create_dir_all(FS_TRACES_PROCESSING).unwrap();
    fs::create_dir_all(FS_TRACES_PROVED).unwrap();
    fs::create_dir_all(FS_PROOF).unwrap();

    //start prover
    let status = Arc::new(Mutex::new(true));
    prove_for_trace(Arc::clone(&status));

    //start mng
    let service = Router::new()
        .route("/prove_block", post(download_trace))
        .route("/download_proof", post(download_trace))
        .route("/status", post(prover_status))
        .layer(AddExtensionLayer::new(Arc::clone(&status)))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    axum::Server::bind(&"127.0.0.1:3030".parse().unwrap())
        .serve(service.into_make_service())
        .await
        .unwrap();
}

/**
 * prove
 */
fn prove_for_trace(prove_status: Arc<Mutex<bool>>) {
    task::spawn(prove_process());

    task::spawn(async move {
        loop {
            let traces_dir = fs::read_dir(FS_TRACES_PROCESSING).unwrap();
            if traces_dir.last().is_some() {
                *prove_status.lock().unwrap() = true;
            } else {
                *prove_status.lock().unwrap() = false;
            }
            std::thread::sleep(Duration::from_millis(10000))
        }
    });
}

async fn prove_process() {
    //start prover process
    Command::new(PROVER_FOR_TRACES).spawn().unwrap();
    log::info!("starting prove_process");
}

async fn prover_status(Extension(status): Extension<Arc<Mutex<bool>>>) -> String {
    if *status.lock().unwrap() {
        return String::from("busy");
    } else {
        return String::from("idle");
    }
}

/**
 * Download trace.
 * fetch and save trace from layer2 sequencer.
 */
async fn download_trace(param: String) -> String {
    //fetch block trace
    let prove_request: ProveRequest = serde_json::from_str(param.as_str()).unwrap();
    let provider = Provider::<Http>::try_from(prove_request.rpc.clone())
        .expect("failed to initialize ethers Provider");
    let block_traces: Option<Vec<types::eth::BlockTrace>> = utils::get_block_traces_by_number(
        &provider,
        prove_request.block_num,
        prove_request.block_num + 1,
    )
    .await;
    if block_traces.is_none() {
        log::error!("fetch block trace fail");
        return String::from("fetch block trace fail");
    }

    //save traces
    let traces_path =
        PathBuf::from(FS_TRACES_PROCESSING).join(format!("block#{}.json", prove_request.block_num));
    let mut traces_file = File::create(traces_path).unwrap();
    let save = traces_file.write_all(
        serde_json::to_string(&block_traces.unwrap().last())
            .unwrap()
            .as_bytes(),
    );
    match save {
        Ok(()) => {
            log::info!("save traces successfully!");
            String::from("success")
        }
        Err(e) => {
            log::error!("save trace fail: {e}");
            String::from("save trace fail")
        }
    }
}
