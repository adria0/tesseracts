use handlebars::Handlebars;
use web3::types::{BlockNumber,BlockId};
use reader::BlockchainReader;

use super::error::Error;
use super::html::HtmlRender;

pub fn html(
    reader: &BlockchainReader,
    hb: &Handlebars
) -> Result<String, Error> {
    let mut last_blockno = reader.current_block_number()?;
    let mut blocks = Vec::new();

    for _ in 0..20 {
        let blockno = BlockId::Number(BlockNumber::Number(last_blockno));
        if let Some(block) = reader.block(last_blockno)? {
            blocks.push(json!({
                "block"    : blockno.html(),
                "tx_count" : block.transactions.len()
            }));
        } else {
            return Err(Error::Unexpected);
        }
        last_blockno -= 1;
    }

    Ok(hb.render(
        "home.handlebars",
        &json!({
            "last_indexed_block" : reader.db.get_last_block().unwrap(),
            "blocks": blocks
        }),
    )?)
}