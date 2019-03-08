use super::error::Error;
use super::html::HtmlRender;
use super::utils;

use super::super::eth::BlockchainReader;
use super::super::state::GlobalState;

/// render home page
pub fn render(
    ge: &GlobalState,
    page_no : u64,
) -> Result<String, Error> {

    let hr = HtmlRender::new(&ge); 
    let reader = BlockchainReader::new(&ge);
    let hb = &ge.hb;
    let db = &ge.db;

    let last_blockno = reader.current_block_number()?;
    let mut blocks = Vec::new();

    // get blocks

    let pg = utils::paginate(last_blockno,20,page_no);
    for n in pg.from..pg.to {
        let block_no = last_blockno - n;
        if let Some(block) = reader.block(block_no)? {
            let author = utils::block_author(&ge.cfg,&block);
            blocks.push(json!({
                "block"     : hr.blockno(block_no),
                "tx_count"  : block.transactions.len(),
                "author"    : hr.addr(&author),
                "timestamp" : hr.timestamp(&block.timestamp).text,
                "gas_used"   : block.gas_used.low_u64(), 
                "gas_limit"  : block.gas_limit.low_u64()
            }));
        } else {
            return Err(Error::Unexpected);
        }
    }

    // render

    Ok(hb.render(
        "home.handlebars",
        &json!({
            "last_indexed_block" : db.get_next_block_to_scan().unwrap(),
            "blocks": blocks,
            "has_next_page": pg.next_page.is_some(),
            "next_page": pg.next_page.unwrap_or(0),
            "has_prev_page": pg.prev_page.is_some(),
            "prev_page": pg.prev_page.unwrap_or(0),
        }),
    )?)
}