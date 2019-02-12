use web3::types::{Address, H256, Transaction, TransactionReceipt, U256};
use rustc_hex::ToHex;
use serde_derive::Serialize;
use chrono::prelude::*;

use super::super::types::InternalTx;
use super::super::state::GlobalState;

const DATETIME_FORMAT : &str = "%Y-%m-%d %H:%M:%S";

lazy_static! {
    static ref GWEI: U256 = U256::from_dec_str("1000000000").unwrap();
    static ref ETHER: U256 = U256::from_dec_str("1000000000000000000").unwrap();
}

#[derive(Serialize)]
pub struct TextWithLink {
    pub text: String,
    pub link: Option<String>,
}

impl TextWithLink {
    fn new_link(text: String, link: String) -> Self {
        TextWithLink {
            text,
            link: Some(link),
        }
    }
    fn new_text(text: String) -> Self {
        TextWithLink {
            text,
            link: None,
        }
    }
    pub fn blank() -> Self {
        TextWithLink {
            text: "".to_string(),
            link: None,
        }
    }
}

pub struct HtmlRender<'a> {
    ge : &'a GlobalState,
}

impl<'a> HtmlRender<'a> {
    
    pub fn new(ge :&'a GlobalState) -> HtmlRender<'a> {
        HtmlRender { ge }
    }
    
    pub fn addr(&self, addr : &Address) -> TextWithLink {
        TextWithLink::new_link(
            self.ge.named_address.get(addr).unwrap_or(&format!("0x{:x}", addr)).to_string(),
            format!("/0x{:x}", addr)
        )
    }

    pub fn addr_or(&self, optaddr : &Option<Address>, or : &str) -> TextWithLink {
        if let Some(addr) = optaddr {
            self.addr(&addr)
        } else {
             TextWithLink::new_text(or.to_string())
        }
    }

    pub fn bytes(&self, bytes : &[u8], chunk_size: usize) -> Vec<String> {
        bytes.chunks(chunk_size)
            .map(|c| c.to_hex::<String>())
            .collect()
    }

    pub fn txid_short(&self, txid : &H256) -> TextWithLink {
        TextWithLink::new_link(
            format!("{:x}", txid).chars().take(9).collect::<String>(),
            format!("/0x{:x}", txid),
        )
    }

    pub fn blockno(&self, no : u64) -> TextWithLink {
        TextWithLink::new_link(format!("{}", no), format!("/{}", no))
    }

    pub fn gwei(&self, wei : &U256) -> TextWithLink {
        TextWithLink::new_text(format!("{} GWei ({})", wei / *GWEI, wei))
    }

    pub fn ether(&self, wei : &U256) -> TextWithLink {
        if *wei == U256::zero()  {
            TextWithLink::new_text("0 Ξ".to_string())
        } else {
            let ether  = wei / *ETHER;
            let mut remain = wei % *ETHER;
            while remain > U256::zero() && remain % 10 == U256::zero() {
                remain /= 10; 
            }
            TextWithLink::new_text(format!("{}.{} Ξ", ether, remain))
        }
    }

    pub fn timestamp(&self, sec1970 : &U256) -> TextWithLink {
        let dt = Utc.timestamp(sec1970.low_u64() as i64, 0);
        TextWithLink::new_text(format!("{}",dt.format(DATETIME_FORMAT)))
    }

    pub fn tx(&self,tx: &Transaction, rcpt: &Option<TransactionReceipt>) -> serde_json::Value {
        
        let shortdata = tx
            .input.0.to_hex::<String>()
            .chars().take(8).collect::<String>();

        let (to_link,to_label) = if let Some(to) = tx.to {             
            (self.addr(&to),"")
        } else if let Some(rcpt) = rcpt {
            (self.addr_newcontract(&rcpt.contract_address.unwrap()),"")
        } else {
            (TextWithLink::blank(),"New contract")
        };

        json!({
            "type"          : "EXT",
            "blockno"       : self.blockno(tx.block_number.unwrap().low_u64()),
            "tx"            : self.txid_short(&tx.hash),
            "from"          : self.addr(&tx.from),
            "to_link"       : to_link,
            "to_label"      : to_label,
            "shortdata"     : shortdata,
            "value"         : self.ether(&tx.value)
        })
    }

    fn addr_newcontract(&self, addr: &Address) -> TextWithLink {
        let mut twl = self.addr(&addr);
        twl.text = format!("🆕 {}",twl.text);
        twl
    }

    fn addr_to(&self, to: &Option<Address>, contract: &Option<Address>) -> TextWithLink {
        if let Some(to) = to {
            self.addr(&to)
        } else {
            self.addr_newcontract(&contract.unwrap())
        }
    }

    pub fn tx_itx(&self,tx: &Transaction, itx: &InternalTx) -> serde_json::Value {
        
        let shortdata = itx
            .input.to_hex::<String>()
            .chars().take(8).collect::<String>();

        json!({
            "type"          : "int",
            "blockno"       : self.blockno(tx.block_number.unwrap().low_u64()),
            "tx"            : self.txid_short(&tx.hash),
            "from"          : self.addr(&itx.from),
            "to_link"       : self.addr_to(&itx.to,&itx.contract),
            "shortdata"     : shortdata,
            "value"         : self.ether(&itx.value)
        })
    }

    pub fn itx(&self,itx: &InternalTx) -> serde_json::Value {
        let shortdata = itx
            .input.to_hex::<String>()
            .chars().take(8).collect::<String>();

        json!({
            "from"          : self.addr(&itx.from),
            "to"            : self.addr_to(&itx.to,&itx.contract),
            "shortdata"     : shortdata,
            "value"         : self.ether(&itx.value)
        })
    }

}
