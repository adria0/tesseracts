use web3::types::{Address,H256,U256,BlockId,TransactionId,BlockNumber,Bytes};
use web3::futures::Future;

use rustc_hex::{ToHex};
use rocket::response::content;

use serde_derive::Serialize;
use rocksdb::{DB,Direction,DBIterator,IteratorMode};

use state::*;

lazy_static! {
    static ref GWEI   : U256 = U256::from_dec_str("1000000000").unwrap();
    static ref FINNEY : U256 = U256::from_dec_str("100000000000000").unwrap();
}

#[derive(Debug)]
pub enum Error {
    NotFound,
    Web3(web3::Error),
    Render(handlebars::RenderError),
    DbError(rocksdb::Error),
}

impl From<web3::Error> for Error {
    fn from(err: web3::Error) -> Self {
        Error::Web3(err)
    }
}
impl From<handlebars::RenderError> for Error {
    fn from(err: handlebars::RenderError) -> Self {
        Error::Render(err)
    }
}
impl From<rocksdb::Error> for Error {
    fn from(err: rocksdb::Error) -> Self {
        Error::DbError(err)
    }
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
                        format!("{:x}",h).chars().take(7).collect::<String>(),
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

pub fn open_default(gs : &GlobalState, path : &str) -> Result<u8, Error> {
    DB::open_default(path).map(|x| 1u8 )?;
    let ls = gs.create_local();
    let block = ls.web3.eth().block_number().wait()?;
    Ok(1)
}

pub fn page(innerhtml : &str) -> content::Html<String> {
    let mut html = String::from(""); 
    html.push_str("<html><style>body {font-family: Courier;}</style>");
    html.push_str(&innerhtml.replace(" ","&nbsp;").replace("_"," "));
    html.push_str("</html>");
    content::Html(html)
}

pub fn block_info(gs : &GlobalState, id: BlockId) -> Result<content::Html<String>,Error> {
    let ls = gs.create_local();
    let block = ls.web3.eth().block_with_txs(id).wait()?;
    
    if let Some(block) = block {
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
        Ok(content::Html(gs.tmpl.render("block.handlebars", &json!({
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
            }))?))    
    } else {
        Err(Error::NotFound)
    }
}

pub fn tx_info(gs : &GlobalState, txid: H256) -> Result<content::Html<String>,Error> {
    let ls = gs.create_local();
    let tx = ls.web3.eth().transaction(TransactionId::Hash(txid)).wait()?;

    if let Some(tx) = tx {

        let mut logs = Vec::new();
        let mut cumulative_gas_used = String::from("");
        let mut gas_used = String::from("");
        let mut contract_address = TextWithLink::blank();
        let mut status = String::from("");

        let receipt = ls.web3.eth().transaction_receipt(txid).wait()?;

        if let Some(receipt) = receipt {
            for (_,log) in receipt.logs.into_iter().enumerate() {
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

            cumulative_gas_used = format!("{}",receipt.cumulative_gas_used.low_u64());
            gas_used = format!("{}",receipt.gas_used.low_u64());
            contract_address = receipt.contract_address
                .map_or_else(|| TextWithLink::blank(), |c| c.html());
            status =  receipt.status.map_or_else(|| String::from(""),|s| format!("{}",s));
        }

        let block = tx.block_number.map_or_else(
                || TextWithLink::blank(),
                |b| BlockId::Number(BlockNumber::Number(b.low_u64())).html());

        let inputhtml = tx.input.html();
        let input : Vec<&str> = inputhtml.text.split(',').collect();

        Ok(content::Html(gs.tmpl.render("tx.handlebars", &json!({
            "from"                : tx.from.html(),
            "to"                  : tx.to.html(),
            "value"               : Ether(tx.value).html().text,
            "block"               : block,
            "gas"                 : tx.gas.low_u64(),
            "gas_price"           : GWei(tx.gas_price).html().text,
            "cumulative_gas_used" : cumulative_gas_used,
            "gas_used"            : gas_used,
            "contract_address"    : contract_address,
            "status"              : status,
            "input"               : input,
            "logs"                : logs,
            }))?))

    } else {
        Err(Error::NotFound)
    }
}
 
pub fn addr_info(gs : &GlobalState, addr: Address) -> Result<content::Html<String>,Error> {

    let ls = gs.create_local();
    
    let balance = ls.web3.eth().balance(addr,None).wait()?;
    let code = ls.web3.eth().code(addr,None).wait()?;
    let mut txs = Vec::new();

    for txhash in gs.db.iter_addr_txs(&addr) {
        if let Ok(Some(tx)) = gs.db.get_tx(&txhash) {
            let shortdata = tx.input.0.to_hex::<String>().chars().take(8).collect::<String>();
            txs.push(json!({
                "tx"        : TransactionIdShort(TransactionId::Hash(txhash)).html(),
                "from"      : tx.from.html(),
                "to"        : tx.to.html(),
                "shortdata" : shortdata,
            }));
        } else {
            txs.push(json!({
                "tx"        : TransactionIdShort(TransactionId::Hash(txhash)).html(),
                "from"      : "not indexed",
                "to"        : "not indexed",
                "shortdata" : "not indexed",
            }));
        }
    }

    Ok(content::Html(gs.tmpl
        .render("address.handlebars", &json!({
            "balance" : Ether(balance).html().text,
            "code"    : code.html().text.split(',').into_iter().collect::<Vec<&str>>(),
            "txs"     : txs
        }))?))    
}

pub fn home(gs : &GlobalState) -> Result<content::Html<String>,Error> {
    let ls = gs.create_local();

    let mut last_blockno = ls.web3.eth().block_number().wait()?.low_u64();
    let mut blocks = Vec::new();

    for _ in 0..20 {
        let blockno = BlockId::Number(BlockNumber::Number(last_blockno));
        let block = ls.web3.eth().block(blockno.clone()).wait()?;
        if let Some(block) = block {
            blocks.push(json!({
                "block"    : blockno.html(),
                "tx_count" : block.transactions.len()
            }));
        }
        last_blockno = last_blockno - 1;
    }

    Ok(content::Html(gs.tmpl
        .render("home.handlebars", &json!({
            "blocks": blocks
        }))?))
}