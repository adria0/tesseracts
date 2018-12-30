use web3::types::Address;
use web3::types::H256;
use web3::types::U256;
use web3::types::BlockId;
use web3::types::TransactionId;
use web3::types::BlockNumber;
use web3::types::Bytes;
use web3::futures::Future;

use rustc_hex::{ToHex};
use rocket::response::content;

use serde_derive::Serialize;

use state::*;

lazy_static! {
    static ref GWEI   : U256 = U256::from_dec_str("1000000000").unwrap();
    static ref FINNEY : U256 = U256::from_dec_str("100000000000000").unwrap();
}

pub struct TransactionIdShort(pub TransactionId);
pub struct GWei(pub U256);
pub struct Ether(pub U256);

#[derive(Serialize)]
pub struct TextWithLink {
    pub text : String,
    pub link : Option<String>,
}
impl TextWithLink {
    fn new_link(text : String, link: String) -> Self {
        TextWithLink{text:text, link:Some(link)}
    }
    fn new_text(text : String) -> Self {
        TextWithLink{text:text, link:None}
    }
    fn blank() -> Self {
        TextWithLink{text:"".to_string(), link:None}
    }
}

pub trait HtmlRender {
    fn html(&self) -> TextWithLink;
} 

impl HtmlRender for Address {
    fn html(&self) -> TextWithLink {
        TextWithLink::new_link(
            format!("0x{:x}",self),
            format!("/0x{:x}",self)
        )
    }
}

impl HtmlRender for Bytes {
    fn html(&self) -> TextWithLink {
        TextWithLink::new_text(
            (self.0).as_slice()
                .chunks(32).into_iter()
                .map(|c| c.to_hex::<String>() )
                .map(|c| format!("{},",c))
                .collect::<String>()
        )
    }
}

impl HtmlRender for Option<Address> {
    fn html(&self) -> TextWithLink {
        self.map(|v| v.html()).unwrap_or(
            TextWithLink::new_text("New contract".to_string())
        )
    }
}

impl HtmlRender for TransactionId {
    fn html(&self) -> TextWithLink {
            match &self {
                TransactionId::Hash(h) =>
                    TextWithLink::new_link(
                        format!("0x{:x}",h),
                        format!("/0x{:x}",h),
                    ),
                _ => unreachable!()
            }            
    }
}

impl HtmlRender for TransactionIdShort {
    fn html(&self) -> TextWithLink {
            match &self.0 {
                TransactionId::Hash(h) => 
                    TextWithLink::new_link(
                        format!("0x{:x}",h).chars().take(7).collect::<String>(),
                        format!("/0x{:x}",h),
                    ),
                _ => unreachable!()
            }            
    }
}

impl HtmlRender for BlockId {
    fn html(&self) -> TextWithLink {
        match &self {
           BlockId::Number(BlockNumber::Number(n)) => 
                TextWithLink::new_link(
                    format!("{}",n),
                    format!("/{}",n),
                ),
                _ => unreachable!()
        }
    }
}

impl HtmlRender for GWei {
    fn html(&self) -> TextWithLink {
        TextWithLink::new_text(
            format!("{} GWei ({})",self.0 / *GWEI,self.0 )
        )
    }
}

impl HtmlRender for Ether {
    fn html(&self) -> TextWithLink {
        TextWithLink::new_text(
            format!("{} Finney ({})",self.0 / *FINNEY,self.0 )
        )
    }
}

pub fn page(innerhtml : &str) -> content::Html<String> {
    let mut html = String::from(""); 
    html.push_str("<html><style>body {font-family: Courier;}</style>");
    html.push_str(&innerhtml.replace(" ","&nbsp;").replace("_"," "));
    html.push_str("</html>");
    content::Html(html)
}

pub fn block_info(gs : &GlobalState, id: BlockId) -> content::Html<String> {
    let ls = gs.create_local();
    let block = ls.web3.eth().block_with_txs(id).wait().unwrap().unwrap();
    
    let mut txs = Vec::new();
    for tx in &block.transactions {
        let shortdata = tx.input.0.to_hex::<String>().chars().take(8).collect::<String>();
        txs.push(json!({
            "shorttx"   : TransactionIdShort(TransactionId::Hash(tx.hash)).html(),
            "from"      : tx.from.html(),
            "to"        : tx.to.html(),
            "shortdata" : shortdata,
        }));
    }

    content::Html(gs.tmpl
        .render("block.handlebars", &json!({
            "parent_hash"      : block.parent_hash,
            "uncles_hash"      : block.uncles_hash,
            "author"           : block.author.html(),
            "state_root"       : block.state_root,
            "receipts_root"    : block.receipts_root,
            "gas_used"         : block.gas_used.low_u64(),
            "gas_limit"        : block.gas_limit.low_u64(),
            "extra_data"       : block.extra_data,
            "timestamp"        : block.timestamp,
            "difficulty"       : block.difficulty,
            "total_difficulty" : block.total_difficulty,
            "seal_fields"      : block.seal_fields,
            "uncles"           : block.uncles,
            "txs"              : txs
        })).expect("error rendering"))    
}

pub fn tx_info(gs : &GlobalState, txid: H256) -> content::Html<String> {
    let ls = gs.create_local();
    let tx = ls.web3.eth().transaction(TransactionId::Hash(txid)).wait().unwrap().unwrap();
    let receipt = ls.web3.eth().transaction_receipt(txid).wait().unwrap().unwrap();

    let mut logs = Vec::new();
    for (i,log) in receipt.logs.into_iter().enumerate() {
        let mut topics = Vec::new();
        for (t,topic) in log.topics.into_iter().enumerate() {
            topics.push(json!({"n":t, "hash": topic}));
        }
        logs.push(json!({
            "address" : log.address.html(),
            "data"    : log.data.html().text.split(',').into_iter().collect::<Vec<&str>>(),
            "topics"  : topics,
        }));
    }

    content::Html(gs.tmpl
        .render("tx.handlebars", &json!({
            "from"                : tx.from.html(),
            "to"                  : tx.to.html(),
            "value"               : Ether(tx.value).html().text,
            "block"               : BlockId::Number(BlockNumber::Number(tx.block_number.unwrap().low_u64())).html(),
            "gas"                 : tx.gas.low_u64(),
            "gas_price"           : GWei(tx.gas_price).html().text,
            "cumulative_gas_used" : receipt.cumulative_gas_used.low_u64(),
            "gas_used"            : receipt.gas_used.low_u64(),
            "contract_address"    : receipt.contract_address.map(|x| x.html()).unwrap_or(TextWithLink::blank()),
            "status"              : receipt.status.unwrap(),
            "input"               : tx.input.html().text.split(',').into_iter().collect::<Vec<&str>>(),
            "logs"                : logs,
        })).expect("error rendering"))    
}
 
pub fn addr_info(gs : &GlobalState, addr: Address) -> content::Html<String> {
    let ls = gs.create_local();
    let balance = ls.web3.eth().balance(addr,None).wait().unwrap();
    let code = ls.web3.eth().code(addr,None).wait().unwrap();
    content::Html(gs.tmpl
        .render("address.handlebars", &json!({
            "balance" : Ether(balance).html().text,
            "code"    : code.html().text.split(',').into_iter().collect::<Vec<&str>>(),
        })).expect("error rendering"))    
}

pub fn home(gs : &GlobalState) -> content::Html<String> {
    let ls = gs.create_local();
    let mut last_blockno = ls.web3.eth().block_number().wait().unwrap();
    let mut blocks = Vec::new();
    for _ in 0..20 {
        let blockno = BlockId::Number(BlockNumber::Number(last_blockno.low_u64()));
        let block = ls.web3.eth().block(blockno.clone()).wait().unwrap().unwrap();
        blocks.push(json!({
            "block"    : blockno.html(),
            "tx_count" : block.transactions.len()
        }));
        last_blockno = last_blockno - 1;
    }

    content::Html(gs.tmpl
        .render("home.handlebars", &json!({
            "blocks": blocks
        })).expect("error rendering"))
}