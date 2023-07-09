use env_logger::Env;
use ethers::providers::{Http, Provider};
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use std::env::var;
use std::path::PathBuf;
use std::str::FromStr;
use std::fs;
use zkevm::{
    circuit::{AGG_DEGREE, DEGREE},
    io,
    prover::Prover,
    utils::{load_or_create_params, load_or_create_seed},
};
use zkevm_prover::utils;


#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    //load traces
    let proof_vec = read_from_file(&path);
    let proof = serde_json::from_slice::<TargetCircuitProof>(proof_vec.as_slice()).unwrap();

    //step 2. create prover
    let mut prover = create_prover(params_path);

    //step 3. start prove
    let block_trace_array = block_traces.as_slice();
    let result = prover.create_agg_circuit_proof_batch(block_trace_array);
    match result {
        Ok(proof) => {
            log::info!("prove result is: {:#?}", proof);
            //save proof
            let mut proof_path = PathBuf::from("./proof").join("agg.proof");
            fs::create_dir_all(&proof_path).unwrap();
            proof.write_to_dir(&mut proof_path);
            //save verify contract
            let solidity = prover.create_solidity_verifier(&proof);
            log::info!("verify solidity is: {:#?}", solidity);

            let mut folder = PathBuf::from_str("./verifier").unwrap();
            io::write_verify_circuit_solidity(&mut folder, &Vec::<u8>::from(solidity.as_bytes()))
        }
        Err(e) => {
            log::info!("prove err: {:#?}", e);
        }
    };
}

/**
 * create prover of zkevm
 */
fn create_prover(params_path: String) -> Prover {

    let params = load_or_create_params(params_path.as_str(), *DEGREE)
        .expect("failed to load or create kzg params");
    let agg_params = load_or_create_params(params_path.as_str(), *AGG_DEGREE)
        .expect("failed to load or create agg-kzg params");
    let seed = load_or_create_seed("./prove_seed").expect("failed to load or create seed");
    let rng = XorShiftRng::from_seed(seed);

    Prover::from_params_and_rng(params, agg_params, rng)
}
