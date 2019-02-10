use super::error::*;
use super::html::*;

use super::super::bcio::BlockchainReader;
use super::super::state::GlobalState;
use super::clique::parse_clique_header;

pub fn html(
    ge: &GlobalState,
    blockno: u64,
) -> Result<String> {

    let wc = ge.new_web3client();
    let hr = HtmlRender::new(&ge); 
    let reader = BlockchainReader::new(&wc,&ge.db);
    let hb = &ge.hb;

    if let Some(block) = reader.block_with_txs(blockno)? {
        let mut txs = Vec::new();
        for tx in &block.transactions {
            txs.push(hr.tx(&tx));
        }
        let author = parse_clique_header(&block).unwrap();
        Ok(hb.render(
            "block.handlebars",
            &json!({
            "blockno"          : hr.blockno(blockno).text,
            "parent_hash"      : block.parent_hash,
            "uncles_hash"      : block.uncles_hash,
            "author"           : hr.addr(&author),
            "state_root"       : block.state_root,
            "receipts_root"    : block.receipts_root,
            "gas_used"         : block.gas_used.low_u64(),
            "gas_limit"        : block.gas_limit.low_u64(),
            "extra_data"       : block.extra_data,
            "timestamp"        : hr.timestamp(&block.timestamp).text,
            "difficulty"       : block.difficulty,
            "total_difficulty" : block.total_difficulty,
            "seal_fields"      : block.seal_fields,
            "uncles"           : block.uncles,
            "txs"              : txs
            }),
        )?)
    } else {
        Err(Error::NotFound)
    }
}