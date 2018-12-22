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

use state::*;

pub trait HtmlRender {
    fn html(&self) -> String;
} 

impl HtmlRender for Address {
    fn html(&self) -> String {
        format!("<a_href=/0x{:x}>0x{:x}</a>",self,self)
    }
}


pub struct BytesWithPadding(pub Bytes, pub u8);
impl HtmlRender for BytesWithPadding {
    fn html(&self) -> String {
        let mut spaces = String::from("");
        for _ in 0..(self.1) {
            spaces.push(' ');
        }
        (self.0).0.as_slice()
            .chunks(32).into_iter()
            .map(|c| c.to_hex::<String>() )
            .map(|c| format!("{}{}<br>",spaces,c))
            .collect::<String>()
    }
}

impl HtmlRender for Option<Address> {
    fn html(&self) -> String {
        self.map(|v| v.html()).unwrap_or("New contract".to_string())
    }
}

impl HtmlRender for TransactionId {
    fn html(&self) -> String {
            match &self {
                TransactionId::Hash(h) =>
                    format!("<a_href=/0x{:x}>0x{:x} </a>",h,h),
                _ => String::from("")
            }            
    }
}

pub struct TransactionIdShort(pub TransactionId);
impl HtmlRender for TransactionIdShort {
    fn html(&self) -> String {
            match &self.0 {
                TransactionId::Hash(h) => 
                    format!("<a_href=/0x{:x}>0x{}</a>",
                    h,
                    format!("{:x}",h).chars().take(7).collect::<String>()),
                _ => String::from("")
            }            
    }
}

impl HtmlRender for BlockNumber {
    fn html(&self) -> String {
        match &self {
           BlockNumber::Number(n) => format!("<a_href=/{}>{}</a>",n,n),
            _ => String::from("")
        }
    }
}


pub struct GWei(pub U256);
impl HtmlRender for GWei {
    fn html(&self) -> String {
        let gwei = U256::from_dec_str("1000000000").expect("invalid number");
        format!("{} GWei ({})",self.0 / gwei,self.0 )
    }
}

pub struct Ether(pub U256);
impl HtmlRender for Ether {
    fn html(&self) -> String {
        let finney = U256::from_dec_str("100000000000000").expect("invalid number");
        format!("{} Finney ({})",self.0/finney,self.0 )
    }
}

pub fn page(innerhtml : &str) -> content::Html<String> {
    let mut html = String::from(""); 
    html.push_str("<html><style>body {font-family: Courier;}</style>");
    html.push_str(&innerhtml.replace(" ","&nbsp;").replace("_"," "));
    html.push_str("</html>");
    content::Html(html)
}


pub fn block_info(st : &State, id: BlockId) -> content::Html<String> {
    let block = &st.web3.eth().block_with_txs(id).wait().unwrap().unwrap();
    let mut html = String::from(""); 
    html.push_str(&format!("parent hash      : {:?}<br>",block.parent_hash));
    html.push_str(&format!("uncles hash      : {:?}<br>",block.uncles_hash));
    html.push_str(&format!("author           : {}<br>",block.author.html()));
    html.push_str(&format!("state root       : {:?}<br>",block.state_root));
    html.push_str(&format!("receipts root    : {:?}<br>",block.receipts_root));
    html.push_str(&format!("gas used         : {:?}<br>",block.gas_used));
    html.push_str(&format!("gas limit        : {:?}<br>",block.gas_limit));
    html.push_str(&format!("extra_data       : {:}{:}",block.extra_data.0.to_hex::<String>(),"<br>"));
    html.push_str(&format!("timestamp        : {:?}<br>",block.timestamp));
    html.push_str(&format!("difficulty       : {:?}<br>",block.difficulty));
    html.push_str(&format!("total difficulty : {:?}<br>",block.total_difficulty));
    html.push_str(&format!("seal fields      : {:?}<br>",block.seal_fields));
    html.push_str(&format!("uncles           : {:?}<br><br>",block.uncles));
    html.push_str(&format!("transactions<br>"));
    for tx in &block.transactions {
        let data = tx.input.0.to_hex::<String>().chars().take(8).collect::<String>();
        html.push_str(&format!(" {} {:42}→{:42} {}<br>",
            TransactionIdShort(TransactionId::Hash(tx.hash)).html(),
            tx.from.html(),
            tx.to.html(),
            data));
    }
    page(&html)
}

pub fn tx_info(st : &State, txid: H256) -> content::Html<String> {
    let tx = st.web3.eth().transaction(TransactionId::Hash(txid)).wait().unwrap().unwrap();
    let mut html = String::from("");

    html.push_str(&format!("from              : {}<br>",tx.from.html()));
    html.push_str(&format!("to                : {}<br>",tx.to.html()));
    html.push_str(&format!("value             : {}<br>",Ether(tx.value).html()));
    html.push_str(&format!("block             : {:?}<br>",tx.block_number.unwrap()));
    html.push_str(&format!("gas               : {:?}<br>",tx.gas));
    html.push_str(&format!("gasprice          : {:?}<br>",tx.gas_price));

    let receipt = st.web3.eth().transaction_receipt(txid).wait().unwrap().unwrap();

    html.push_str(&format!("cumulativeGasUsed : {:?}<br>",receipt.cumulative_gas_used));
    html.push_str(&format!("gasUsed           : {:?}<br>",receipt.gas_used));
    html.push_str(&format!("contractAddress   : {}<br>",receipt.contract_address.map(|x| x.html()).unwrap_or("".to_string())));
    html.push_str(&format!("status            : {:?}<br><br>",receipt.status.unwrap()));
    html.push_str(&format!("input<br>{:}",BytesWithPadding(tx.input,4).html()));
    html.push_str(&format!("logs<br>"));

    for (i,log) in receipt.logs.into_iter().enumerate() {
        html.push_str(&format!("  - {}</a><br>",log.address.html()));
        for (t,topic) in log.topics.into_iter().enumerate() {
            html.push_str(&format!("    [{:}] {:?}<br>",t,topic));
        }
        html.push_str(&format!("    data:<br>{}",BytesWithPadding(log.data,9).html()));
    }
    page(&html)
}

pub fn addr_info(st : &State, addr: Address) -> content::Html<String> {
    let mut html = String::from(""); 

    let balance = st.web3.eth().balance(addr,None).wait().unwrap();
    html.push_str(&format!("balance           : {}<br>",Ether(balance).html()));
    let code = st.web3.eth().code(addr,None).wait().unwrap();
    html.push_str(&format!("code              :<br>{}",BytesWithPadding(code,4).html()));
    page(&html)
}

pub fn home(st : &State) -> content::Html<String> {
    let mut last_blockno = st.web3.eth().block_number().wait().unwrap();
    let mut html = String::from(""); 
    for _ in 0..20 {
        let blockno = BlockId::Number(BlockNumber::Number(last_blockno.low_u64()));
        let block = st.web3.eth().block(blockno).wait().unwrap().unwrap();
        html.push_str(&format!("Block {} TxCount {:?}<br>",
            BlockNumber::Number(last_blockno.low_u64()).html(),
            block.transactions.len()).to_string()
        );
        last_blockno = last_blockno - 1;
    }
    page(&html)
}
