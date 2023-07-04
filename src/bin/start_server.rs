use env_logger::Env;
use ethers::providers::{Http, Provider};
use types::eth::BlockTrace;
use zkevm::prover::Prover;
use serde::{Deserialize, Serialize};
use axum::{routing::post, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[derive(Serialize, Deserialize, Debug)]
struct ProveRequest {
    block_num: u64,
    rpc: String,
}



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

async fn prove_service(param: String) {
    let prove_request: ProveRequest = serde_json::from_str(param.as_str()).unwrap();

    //step 1. fetch block trace
    let provider = Provider::<Http>::try_from(prove_request.rpc)
        .expect("mock-testnet: failed to initialize ethers Provider");

    let block_traces = get_block_traces_by_number(&provider, prove_request.block_num, prove_request.block_num + 1).await.unwrap();

    //step 2. init prover
    let mut prover = Prover::from_fpath("../scroll-prover/src", ".../scroll-prover/src/test_seed");

    //step 3. start prove
    let block_trace_array = block_traces.as_slice();
    let result = prover.create_agg_circuit_proof_batch(block_trace_array);
    match result {
        Ok(proof) => {
            println!("=========================>prove result is : {:?}", proof);
        },
        Err(e) => {
            println!("=========================>prove err: {:?}", e);
       }
    };

}


async fn get_block_traces_by_number(
    provider: &Provider<Http>,
    block_start: u64,
    block_end: u64,
) -> Option<Vec<BlockTrace>> {
    let mut block_traces: Vec<BlockTrace> = Vec::new();
    for i in block_start..block_end {
        log::info!("zkevm-prover: requesting trace of block {i}");
        let trace = provider
            .request("scroll_getBlockTraceByNumberOrHash", [format!("{i:#x}")])
            .await
            .unwrap();
        block_traces.push(trace);
    }
    Some(block_traces)
}
