use env_logger::Env;
use ethers::providers::{Http, Provider};
use zkevm::prover::{Prover, AggCircuitProof};
use serde::{Deserialize, Serialize};
use axum::{routing::post, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use zkevm_prover::utils;


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

    let service = Router::new()
        .route("/prove_block", post(prove_service))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // run it with hyper on localhost:3030
    axum::Server::bind(&"127.0.0.1:3030".parse().unwrap())
        .serve(service.into_make_service())
        .await
        .unwrap();
}

/**
 * Prove service.
 * Handle Prove Request, Use Layer2's rpc address and block number to fetch trace and generate zk proof.
 */
async fn prove_service(param: String)-> String  {
    
    //step 1. fetch block trace
    let prove_request: ProveRequest = serde_json::from_str(param.as_str()).unwrap();
    let provider = Provider::<Http>::try_from(prove_request.rpc)
        .expect("mock-testnet: failed to initialize ethers Provider");

    let block_traces = utils::get_block_traces_by_number(&provider, prove_request.block_num, prove_request.block_num + 1).await.unwrap();

    //step 2. create prover
    let mut prover = Prover::from_fpath("./prove_params", "./prove_seed"); //TODO put it in the static field or cache

    //step 3. start prove
    let block_trace_array = block_traces.as_slice();
    let result = prover.create_agg_circuit_proof_batch(block_trace_array);
    let proof = match result {
        Ok(proof) => {
            log::info!("prove result is: {:#?}", proof);
            proof
        },
        Err(e) => {
            log::info!("prove err: {:#?}", e);
            AggCircuitProof::default()
       }
    };

    //step4. return prove result
    let data = serde_json::to_string(&proof).unwrap();
    data
}

