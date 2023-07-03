use env_logger::Env;
use ethers::providers::{Http, Provider};
use reqwest::Url;
use std::env::var;
// use zkevm_prover::contract_tx;
use types::eth::BlockTrace;
use zkevm::prover::Prover;
use zkevm_prover::contract_tx;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let block_num: u64 = var("PROVERD_BLOCK_NUM")
        .expect("PROVERD_BLOCK_NUM env var")
        .parse()
        .expect("Cannot parse PROVERD_BLOCK_NUM env var");
    let rpc_url: String = var("PROVERD_RPC_URL")
        .expect("PROVERD_RPC_URL env var")
        .parse()
        .expect("Cannot parse PROVERD_RPC_URL env var");
    let provider = Provider::<Http>::try_from(rpc_url)
        .expect("mock-testnet: failed to initialize ethers Provider");

    let block_traces = get_block_traces_by_number(&provider, block_num, block_num + 1).await;

    let mut prover = Prover::from_fpath("params_path", "seed_path");


    if let Some(block_traces) = block_traces {
        let block_trace_array = block_traces.as_slice();
        let result = prover.create_agg_circuit_proof_batch(block_trace_array);
    }
}

async fn get_block_traces_by_number(
    provider: &Provider<Http>,
    block_start: u64,
    block_end: u64,
) -> Option<Vec<BlockTrace>> {
    let mut block_traces: Vec<BlockTrace> = Vec::new();
    for i in block_start..=block_end {
        log::info!("zkevm-prover: requesting trace of block {i}");
        let trace = provider
            .request("scroll_getBlockTraceByNumberOrHash", [format!("{i:#x}")])
            .await
            .unwrap();
        block_traces.push(trace);
    }
    Some(block_traces)
}
