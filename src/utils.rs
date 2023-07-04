
use ethers::providers::{Http, Provider};
use types::eth::BlockTrace;

pub async fn get_block_traces_by_number(
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
