use web3::types::{BlockNumber,BlockId};

use super::error::*;
use super::clique::parse_clique_header;
use super::error::Error;
use super::html::HtmlRender;
use super::paginate;

use super::super::state::GlobalState;
use super::super::bcio::BlockchainReader;

pub fn html(
    ge: &GlobalState,
    page_no : u64,
) -> Result<String> {

    let wc = ge.new_web3client();
    let hr = HtmlRender::new(&ge); 
    let reader = BlockchainReader::new(&wc,&ge.db);
    let db = &ge.db;
    let hb = &ge.hb;

    let mut blocks = Vec::new();

    let count_non_empty_blocks = db.count_non_empty_blocks()?;
    let  limit = if count_non_empty_blocks > 200 {
        200
    } else {
        count_non_empty_blocks
    };
    let pg = paginate::paginate(limit,20,page_no);

    if pg.from <= pg.to {
        let it = reader.db.iter_non_empty_blocks()?.skip(pg.from as usize);
        for n in it.take((pg.to-pg.from) as usize) {
            if let Some(block) = reader.block(n)? {
                let author = parse_clique_header(&block).unwrap();
                blocks.push(json!({
                    "block"     : hr.blockno(n),
                    "author"    : hr.addr(&author),
                    "tx_count"  : block.transactions.len(),
                    "timestamp" : hr.timestamp(&block.timestamp).text,
                    "gas_used"  : block.gas_used.low_u64(), 
                    "gas_limit" : block.gas_limit.low_u64()
                }));
            } else {
                return Err(Error::Unexpected);
            }
        }
    }

    Ok(hb.render(
        "neb.handlebars",
        &json!({
            "last_indexed_block" : reader.db.get_last_block().unwrap(),
            "blocks": blocks,
            "has_next_page": pg.next_page.is_some(),
            "next_page": pg.next_page.unwrap_or(0),
            "has_prev_page": pg.prev_page.is_some(),
            "prev_page": pg.prev_page.unwrap_or(0),
        }),
    )?)
}
