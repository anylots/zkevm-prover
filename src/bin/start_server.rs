use std::{sync::Arc, thread};
use tokio::sync::Mutex;

use axum::{routing::post, Router};
use env_logger::Env;
use ethers::providers::{Http, Provider};
use serde::{Deserialize, Serialize};
use tower_http::add_extension::AddExtensionLayer;
use axum::extract::Extension;
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

unsafe impl Send for ProveRequest {}
unsafe impl Sync for ProveRequest {}

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
        .layer(AddExtensionLayer::new(task_queue))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // run it with hyper on localhost:3030
    axum::Server::bind(&"127.0.0.1:3030".parse().unwrap())
        .serve(service.into_make_service())
        .await
        .unwrap();
}

async fn prove_for_queue(task_queue: Arc<Mutex<Vec<ProveRequest>>>) {
    loop {
        let queue = task_queue.lock().await;
        let prove_request = queue.get(0).unwrap();
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

        //step4. return prove result
        let data = serde_json::to_string(&proof).unwrap();
        // data
    }
}

/**
 * Prove service.
 * Handle Prove Request, Use Layer2's rpc address and block number to fetch trace and generate zk proof.
 */
async fn add_queue(
    Extension(info): Extension<Arc<Mutex<Vec<ProveRequest>>>>,
    param: String,
) -> String {
    format!(
        "Sigined User: {}",
        info.lock().await.get(0).unwrap().block_num
    );

    //step 1. fetch block trace
    let prove_request: ProveRequest = serde_json::from_str(param.as_str()).unwrap();
    info.lock().await.push(prove_request);
    String::from("success")
}
