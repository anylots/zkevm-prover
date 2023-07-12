use std::time::Duration;
use std::{sync::Arc, thread};
use tokio::sync::Mutex;
// use std::sync::Mutex;

use axum::extract::Extension;
use axum::{routing::post, Router};
use env_logger::Env;
use ethers::providers::{Http, Provider};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

use tower_http::add_extension::AddExtensionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use zkevm::prover::{AggCircuitProof, Prover};
use zkevm_prover::utils;

// proveRequest
#[derive(Serialize, Deserialize, Debug)]
struct ProveRequest {
    block_num: u64,
    rpc: String,
}

//proveResult
#[derive(Serialize, Deserialize, Debug)]
struct ProveResult {
    prove_name: u64,
    proof: String,
}

const FS_TRACES_PROCESSING: &'static str = "traces/processing";
const FS_TRACES_PROVED: &'static str = "traces/proved";
const FS_PROOF_PARAMS: &'static str = "prove_params";
const FS_PROVE_SEED: &'static str = "prove_seed";
const FS_PROOF: &'static str = "proof";

/**
 * Start Server.
 * this will start an HTTP server, to process zk prove requests.
 */
#[tokio::main]
async fn main() {
    //step1. prepare environment
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    fs::create_dir_all(FS_TRACES_PROCESSING).unwrap();
    fs::create_dir_all(FS_TRACES_PROVED).unwrap();
    fs::create_dir_all(FS_PROOF).unwrap();

    let queue: Arc<Mutex<Vec<ProveRequest>>> = Arc::new(Mutex::new(Vec::new()));

    //step2. start mng
    let task_queue: Arc<Mutex<Vec<ProveRequest>>> = Arc::clone(&queue);
    tokio::spawn(async {
        let service = Router::new()
            .route("/prove_block", post(push_queue))
            .route("/query_proof", post(query_proof))
            .layer(AddExtensionLayer::new(task_queue))
            .layer(CorsLayer::permissive())
            .layer(TraceLayer::new_for_http());

        axum::Server::bind(&"127.0.0.1:3030".parse().unwrap())
            .serve(service.into_make_service())
            .await
            .unwrap();
    });

    //step3. start prover
    let prove_queue: Arc<Mutex<Vec<ProveRequest>>> = Arc::clone(&queue);
    prove_for_queue(prove_queue).await;
}

async fn prove_for_queue(prove_queue: Arc<Mutex<Vec<ProveRequest>>>) {
    //step 1. create prover
    let mut prover = Prover::from_fpath(FS_PROOF_PARAMS, FS_PROVE_SEED);
    loop {
        //step 2. load request from queue
        let queue = prove_queue.lock().await.pop();
        if queue.is_none() {
            continue;
        }
        let prove_request: ProveRequest = queue.unwrap();
        let provider = match Provider::try_from(req.rpc) {
            Ok(provider) => provider,
            Err(e) => {
              log::error!("Failed to init provider: {:#?}", e);
              continue;
            }
          };    

        let block_traces = utils::get_block_traces_by_number(
            &provider,
            prove_request.block_num,
            prove_request.block_num + 1,
        )
        .await
        .unwrap();
        if block_traces.is_empty() {
            continue;
        }

        //step 3. start prove
        log::info!("start prove, block num is: {:#?}", prove_request.block_num);
        let proof = match prover.create_agg_circuit_proof_batch(block_traces.as_slice()) {
            Ok(proof) => {
                log::info!("the prove result is: {:#?}", proof);
                proof
            }
            Err(e) => {
                log::error!("prove err: {:#?}", e);
                continue;
            }
        };
        log::info!("end prove, block num is: {:#?}", prove_request.block_num);

        //step 4. save proof
        let mut proof_path =
            PathBuf::from(FS_PROOF).join(format!("agg-proof#block#{}", prove_request.block_num));
        proof.write_to_dir(&mut proof_path);

        thread::sleep(Duration::from_millis(2000))
    }
}

/**
 * Prove service.
 * Handle Prove Request, Use Layer2's rpc address and block number to fetch trace and generate zk proof.
 */
async fn push_queue(
    Extension(queue): Extension<Arc<Mutex<Vec<ProveRequest>>>>,
    param: String,
) -> String {
    //Parameter non empty verification
    if param.is_empty() {
        return String::from("request is empty");
    }

    // Deserialize parameter
    let prove_request: Result<ProveRequest, serde_json::Error> = serde_json::from_str(&param);
    let prove_request = match prove_request {
        Ok(req) => req,
        Err(_) => return String::from("deserialize proveRequest failed"),
    };
    // Block Num Range verification
    if prove_request.block_num == 0 {
        return String::from("block_num should be greater than 0");
    }

    // Rpc url format verification
    if !prove_request.rpc.starts_with("http://") {
        return String::from("invalid rpc url");
    }
    queue.lock().await.push(prove_request);
    String::from("add task success")
}

async fn query_proof(block_num: String) -> String {
    let fs: Result<fs::ReadDir, std::io::Error> = fs::read_dir(FS_PROOF);
    let mut data = String::new();
    for entry in fs.unwrap() {
        let path = entry.unwrap().path();
        if path
            .to_str()
            .unwrap()
            .contains(format!("block#{}", block_num).as_str())
        {
            let mut file = fs::File::open(path).unwrap();
            file.read_to_string(&mut data).unwrap();
        }
    }
    return data;
}

#[tokio::test]
async fn test() {
    let request = ProveRequest {
        block_num: 4,
        rpc: String::from("127.0.0.1:8569"),
    };
    let info = serde_json::to_string(&request);
    println!("{:?}", info.unwrap());
}
