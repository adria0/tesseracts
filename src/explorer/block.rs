use handlebars::Handlebars;
use super::error::Error;
use super::html::*;

use super::clique::parse_clique_header;
use super::super::bcio::BlockchainReader;

pub fn html(
    reader: &BlockchainReader,
    hb: &Handlebars,
    blockno: u64,
) -> Result<String, Error> {
    if let Some(block) = reader.block_with_txs(blockno)? {
        let mut txs = Vec::new();
        for tx in &block.transactions {
            txs.push(tx_short_json(&tx));
        }
        let author = parse_clique_header(&block);
        Ok(hb.render(
            "block.handlebars",
            &json!({
            "blockno"          : blockno,
            "parent_hash"      : block.parent_hash,
            "uncles_hash"      : block.uncles_hash,
            "author"           : author.html(),
            "state_root"       : block.state_root,
            "receipts_root"    : block.receipts_root,
            "gas_used"         : block.gas_used.low_u64(),
            "gas_limit"        : block.gas_limit.low_u64(),
            "extra_data"       : block.extra_data,
            "timestamp"        : Timestamp(block.timestamp).html().text,
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