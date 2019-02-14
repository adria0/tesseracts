use super::error::*;
use super::html::*;
use super::utils;

use super::super::bcio::BlockchainReader;
use super::super::state::GlobalState;

pub fn html(
    ge: &GlobalState,
    blockno: u64,
) -> Result<String> {

    let hr = HtmlRender::new(&ge); 
    let reader = BlockchainReader::new(&ge);
    let hb = &ge.hb;

    if let Some(block) = reader.block_with_txs(blockno)? {
        let mut txs = Vec::new();
        for tx in &block.transactions {
            if tx.to.is_some() {
                txs.push(hr.tx(&tx,&None));
            } else {
                let (tx,rcpt) = reader.tx(tx.hash)?.unwrap();
                txs.push(hr.tx(&tx,&rcpt));
            }
        }
        let author = utils::author(&ge.cfg,&block);
        let rawextra = hr.bytes(&block.extra_data.0,32);

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
                "extra_data"       : rawextra,
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