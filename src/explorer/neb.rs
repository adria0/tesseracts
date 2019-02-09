use handlebars::Handlebars;
use web3::types::{BlockNumber,BlockId};

use super::super::bcio::BlockchainReader;
use super::clique::parse_clique_header;
use super::error::Error;
use super::html::{Timestamp,HtmlRender};
use super::paginate;
use super::super::db;

pub fn html(
    db : &db::AppDB,
    reader: &BlockchainReader,
    hb: &Handlebars,
    page_no : u64,

) -> Result<String, Error> {

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
            let block_id = BlockId::Number(BlockNumber::Number(n));
            if let Some(block) = reader.block(n)? {
                let author = parse_clique_header(&block).unwrap();
                blocks.push(json!({
                    "block"     : block_id.html(),
                    "author"     : author.html(),
                    "tx_count"  : block.transactions.len(),
                    "timestamp" : Timestamp(block.timestamp).html().text,
                    "gas_used"   : block.gas_used.low_u64(), 
                    "gas_limit"  : block.gas_limit.low_u64()
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
