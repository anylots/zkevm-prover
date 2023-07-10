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

const PROVER_FOR_TRACES: &'static str = "target/release/prove";
const TRACES_PROCESSING: &'static str = "traces/processing";
const TRACES_PROVED: &'static str = "traces/proved";
const PROOF: &'static str = "proof";

/**
 * Start Server.
 * this will start an HTTP server, to process zk prove requests.
 */
#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    fs::create_dir_all(TRACES_PROCESSING).unwrap();
    fs::create_dir_all(TRACES_PROVED).unwrap();
    fs::create_dir_all(PROOF).unwrap();

    let (tx, rx) = channel();
    prove_for_trace(tx, rx);

    let status =true;
    let prove_status = Arc::new(Mutex::new(status));

    let service = Router::new()
        .route("/prove_block", post(download_trace))
        .route("/download_proof", post(download_trace))
        .route("/status", post(download_trace))
        .layer(AddExtensionLayer::new(Arc::clone(&prove_status)))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // run it with hyper on localhost:3030
    axum::Server::bind(&"127.0.0.1:3030".parse().unwrap())
        .serve(service.into_make_service())
        .await
        .unwrap();
}

async fn prove_process(sender: Sender<String>) {
    //start prover process
    let mut prover_bin = Command::new(PROVER_FOR_TRACES).spawn().unwrap();
    prover_bin.wait().unwrap();
    let mut f = BufReader::new(prover_bin.stdout.unwrap());
    loop {
        let mut buf = String::new();
        match f.read_line(&mut buf) {
            Ok(_) => {
                sender.send(buf).unwrap();
                continue;
            }

            Err(e) => {
                println!("an error!: {:?}", e);
                break;
            }
        }
    }
    println!("prover_bin end");
}

fn prove_for_trace(sender: Sender<String>, rx: Receiver<String>) {
    task::spawn(prove_process(sender));
    task::spawn(async {
        for line in rx {
            println!("prover: {}", line);
        }
    });
    task::spawn(async {
        loop {
            let traces_dir = fs::read_dir("traces/processing").unwrap();
            let file = traces_dir.last().unwrap().unwrap().path().is_file();
            std::thread::sleep(Duration::from_millis(10000))
        }
    });
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
        return String::from("fetch block trace fail");
    }

    //save proof
    let traces_path =
        PathBuf::from("traces").join(format!("block#{}.json", prove_request.block_num));
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
