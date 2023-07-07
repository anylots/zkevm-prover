use std::time::Duration;
use std::{sync::Arc, thread};
// use tokio::sync::Mutex;
use std::sync::Mutex;

use axum::extract::Extension;
use axum::{routing::post, Router};
use env_logger::Env;
use ethers::providers::{Http, Provider};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tower_http::add_extension::AddExtensionLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use zkevm::prover::{AggCircuitProof, Prover};
use zkevm_prover::utils;

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

    let task_queue: Arc<Mutex<Vec<ProveRequest>>> = Arc::new(Mutex::new(Vec::new()));
    // thread::spawn(prove_for_queue(Arc::clone(&task_queue)));
    let prove_queue = Arc::clone(&task_queue);
    thread::spawn(|| async move { prove_for_queue(prove_queue).await });

    let service = Router::new()
        .route("/prove_block", post(add_queue))
        .layer(AddExtensionLayer::new(Arc::clone(&task_queue)))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // run it with hyper on localhost:3030
    axum::Server::bind(&"127.0.0.1:3030".parse().unwrap())
        .serve(service.into_make_service())
        .await
        .unwrap();

    // let req_param =String::from("{\"block_num\":4,\"rpc\":\"127.0.0.1:8569\"}");

    let request = ProveRequest {
        block_num: 4,
        rpc: String::from("127.0.0.1:8569"),
    };
    task_queue.lock().unwrap().push(request);
}

/**
 * Prove service.
 * Handle Prove Request, Use Layer2's rpc address and block number to fetch trace and generate zk proof.
 */
async fn add_queue(
    Extension(queue): Extension<Arc<Mutex<Vec<ProveRequest>>>>,
    param: String,
) -> String {
    //step 1. fetch block trace
    let prove_request: ProveRequest = serde_json::from_str(param.as_str()).unwrap();
    queue.lock().unwrap().push(prove_request);
    String::from("success")
}

async fn prove_for_queue(task_queue: Arc<Mutex<Vec<ProveRequest>>>) {
    loop {
        let queue = task_queue.lock().unwrap().pop();
        if !queue.is_some() {
            continue;
        }
        let prove_request = queue.unwrap();
        let provider = Provider::<Http>::try_from(prove_request.rpc.clone())
            .expect("failed to initialize ethers Provider");

        let block_traces = utils::get_block_traces_by_number(
            &provider,
            prove_request.block_num,
            prove_request.block_num + 1,
        )
        .await
        .unwrap();

        //step 2. create prover
        let mut prover = Prover::from_fpath("./prove_params", "./prove_seed"); //TODO put it in the static field or cache

        //step 3. start prove
        let block_trace_array = block_traces.as_slice();
        let result = prover.create_agg_circuit_proof_batch(block_trace_array);
        let proof = match result {
            Ok(proof) => {
                log::info!("prove result is: {:#?}", proof);
                proof
            }
            Err(e) => {
                log::info!("prove err: {:#?}", e);
                AggCircuitProof::default()
            }
        };

        log::info!("prove result is: {:#?}", proof);
        //save proof
        let mut proof_path = PathBuf::from("./proof").join("test.proof");
        fs::create_dir_all(&proof_path).unwrap();
        proof.write_to_dir(&mut proof_path);

        thread::sleep(Duration::from_millis(1000))
    }
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
