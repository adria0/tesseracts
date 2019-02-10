use web3::types::{Address, H256, Transaction, U256};
use rustc_hex::ToHex;
use serde_derive::Serialize;
use chrono::prelude::*;

use super::super::types::InternalTx;
use super::super::state::GlobalState;

const DATETIME_FORMAT : &'static str = "%Y-%m-%d %H:%M:%S";

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
        return HtmlRender { ge }
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

    pub fn bytes(&self, bytes : &[u8]) -> String {
        bytes.chunks(32)
            .map(|c| c.to_hex::<String>())
            .map(|c| format!("{},", c))
            .collect::<String>()
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

    pub fn tx(&self,tx: &Transaction) -> serde_json::Value {
        let shortdata = tx
            .input.0.to_hex::<String>()
            .chars().take(8).collect::<String>();

        json!({
            "blockno"       : self.blockno(tx.block_number.unwrap().low_u64()),
            "tx"            : self.txid_short(&tx.hash),
            "from"          : self.addr(&tx.from),
            "tonewcontract" : tx.to.is_none(),
            "to"            : self.addr_or(&tx.to,""),
            "shortdata"     : shortdata,
            "value"         : self.ether(&tx.value)
        })
    }

    pub fn itx(&self,itx: &InternalTx) -> serde_json::Value {
        let shortdata = itx
            .input.to_hex::<String>()
            .chars().take(8).collect::<String>();

        json!({
            "from"          : self.addr(&itx.from),
            "tonewcontract" : itx.to.is_none(),
            "to"            : self.addr_or(&itx.to,""),
            "shortdata"     : shortdata,
            "value"         : self.ether(&itx.value)
        })
    }

}
