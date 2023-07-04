use env_logger::Env;
use ethers::providers::{Http, Provider};
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use std::env::var;
use std::path::PathBuf;
use std::str::FromStr;
use types::eth::BlockTrace;
use zkevm::{
    circuit::{AGG_DEGREE, DEGREE},
    io,
    prover::Prover,
    utils::{load_or_create_params, load_or_create_seed},
};
use std::fs;

// use zkevm_prover::contract_tx;

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    //step 1. fetch block trace
    let block_num: u64 = var("PROVERD_BLOCK_NUM")
        .expect("PROVERD_BLOCK_NUM env var")
        .parse()
        .expect("Cannot parse PROVERD_BLOCK_NUM env var");
    let rpc_url: String = var("PROVERD_RPC_URL")
        .expect("PROVERD_RPC_URL env var")
        .parse()
        .expect("Cannot parse PROVERD_RPC_URL env var");
    let params_path: String = var("PARAMS_PATH")
        .expect("PARAMS_PATH env var")
        .parse()
        .expect("Cannot parse PARAMS_PATH env var");

    let provider = Provider::<Http>::try_from(rpc_url)
        .expect("failed to initialize ethers Provider");

    let block_traces = get_block_traces_by_number(&provider, block_num, block_num + 1)
        .await
        .unwrap();

    //step 2. create prover
    let mut prover = create_prover(params_path);

    //step 3. start prove
    let block_trace_array = block_traces.as_slice();
    let result = prover.create_agg_circuit_proof_batch(block_trace_array);
    match result {
        Ok(proof) => {
            println!("=========================>prove result is : {:?}", proof);
            let mut proof_path = PathBuf::from("./proof").join("agg.proof");
            fs::create_dir_all(&proof_path).unwrap();
            proof.write_to_dir(&mut proof_path);

            let solidity = prover.create_solidity_verifier(&proof);
            println!("=========================>verify solidity is : {:?}", proof);
            let mut folder = PathBuf::from_str("./").unwrap();
            io::write_verify_circuit_solidity(&mut folder, &Vec::<u8>::from(solidity.as_bytes()))
        }
        Err(e) => {
            println!("=========================>prove err: {:?}", e);
        }
    };
}

fn create_prover(params_path: String) -> Prover {
    let params = load_or_create_params(params_path.as_str(), *DEGREE)
        .expect("failed to load or create kzg params");
    let agg_params = load_or_create_params(params_path.as_str(), *AGG_DEGREE)
        .expect("failed to load or create agg-kzg params");
    let seed = load_or_create_seed("./prove_seed").expect("failed to load or create seed");
    let rng = XorShiftRng::from_seed(seed);

    Prover::from_params_and_rng(params, agg_params, rng)
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
