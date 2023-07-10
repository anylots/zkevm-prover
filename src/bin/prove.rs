use env_logger::Env;
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use zkevm::{
    circuit::{AGG_DEGREE, DEGREE},
    prover::Prover,
    utils,
};

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    //step 1. create prover
    let mut prover: Prover = create_prover(String::from("prove_params"));

    loop {
        //step 2. load traces
        let traces_dir = fs::read_dir("traces/processing").unwrap();
        let path = traces_dir.last().unwrap().unwrap().path();
        let traces = utils::get_block_trace_from_file(path.clone());
        //step 3. start prove
        let result = prover.create_agg_circuit_proof_batch(vec![traces].as_slice());
        match result {
            Ok(proof) => {
                log::info!("prove result is: {:#?}", proof);
                //step 4. save proof
                let mut proof_path = PathBuf::from("proof").join(format!(
                    "agg-proof#{}",
                    path.clone().file_name().unwrap().to_str().unwrap()
                ));
                proof.write_to_dir(&mut proof_path);
                fs::rename(path.clone(), PathBuf::from("traces/proved").join(
                    path.file_name().unwrap().to_str().unwrap())).unwrap();
            }
            Err(e) => {
                log::info!("prove err: {:#?}", e);
            }
        };
        std::thread::sleep(Duration::from_millis(10000))
    }
}

/**
 * create prover of zkevm
 */
fn create_prover(params_path: String) -> Prover {
    let params = utils::load_or_create_params(params_path.as_str(), *DEGREE)
        .expect("failed to load or create kzg params");
    let agg_params = utils::load_or_create_params(params_path.as_str(), *AGG_DEGREE)
        .expect("failed to load or create agg-kzg params");
    let seed = utils::load_or_create_seed("./prove_seed").expect("failed to load or create seed");
    let rng = XorShiftRng::from_seed(seed);

    Prover::from_params_and_rng(params, agg_params, rng)
}
