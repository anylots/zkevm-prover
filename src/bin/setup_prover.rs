use clap::Parser;
use zkevm::{
    circuit::{DEGREE, AGG_DEGREE},
    utils::{load_or_create_params, load_or_create_seed},
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// generate params and write into file
    #[clap(short, long = "params")]
    params_path: Option<String>,
    /// generate seed and write into file
    #[clap(short, long = "seed")]
    seed_path: Option<String>,
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let args = Args::parse();
    let path = match args.params_path {
        Some(path) => path,
        None => String::from("prove_params"),
    };
    //create super circut param
    load_or_create_params(path.as_str(), *DEGREE).expect("failed to load or create params");
    //create aggregator circut param
    load_or_create_params(path.as_str(), *AGG_DEGREE).expect("failed to load or create agg-kzg params");
    //create seed
    load_or_create_seed(path.as_str()).expect("failed to load or create seed");
}
