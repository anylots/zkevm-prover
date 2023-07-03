use env_logger::Env;
use std::env::var;
use ethers_providers::{Http, Provider};
use reqwest::Url;

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

    
}

async fn get_block_trace_by_number(
    provider: &Provider<Http>,
    block_num: i64,
) -> Option<Vec<BlockTrace>> {
    let url = Url::parse_with_params(
        &setting.rollupscan_api_url,
        &[("index", batch_index.to_string())],
    )?;

}