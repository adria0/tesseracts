use handlebars::Handlebars;
use web3::types::{BlockNumber,BlockId};
use reader::BlockchainReader;

use super::error::Error;
use super::html::{Timestamp,HtmlRender};

pub struct Pagination {
    pub from : u64,
    pub to   : u64,
    pub prev_page : Option<u64>,
    pub next_page : Option<u64>,
}

pub fn paginate( max : u64, page_size : u64, page_no : u64 ) -> Pagination {
    let from = page_no*page_size;
    let to = if (page_no+1)*page_size > max {
        max
    } else {
        (page_no+1)*page_size
    };
    let prev_page = if page_no > 0 {
        Some(page_no-1)
    } else {
        None
    };
    let next_page = if to < max {
        Some(page_no+1)
    } else {
        None
    };
    Pagination{from,to,prev_page,next_page}
}

pub fn html(
    reader: &BlockchainReader,
    hb: &Handlebars,
    page_no : u64,
) -> Result<String, Error> {

    let last_blockno = reader.current_block_number()?;
    let mut blocks = Vec::new();

    let pg = paginate(last_blockno,20,page_no);
    for n in pg.from..pg.to {
        let block_no = last_blockno - n;
        let block_id = BlockId::Number(BlockNumber::Number(block_no));
        if let Some(block) = reader.block(block_no)? {
            blocks.push(json!({
                "block"     : block_id.html(),
                "tx_count"  : block.transactions.len(),
                "timestamp" : Timestamp(block.timestamp).html().text,
                "gas_used"   : block.gas_used.low_u64(), 
                "gas_limit"  : block.gas_limit.low_u64()
            }));
        } else {
            return Err(Error::Unexpected);
        }
    }
    Ok(hb.render(
        "home.handlebars",
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