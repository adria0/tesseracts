use web3::types::{Address, BlockId, BlockNumber, Bytes, Transaction, TransactionId, U256};
use rustc_hex::ToHex;
use serde_derive::Serialize;
use chrono::prelude::*;

lazy_static! {
    static ref GWEI: U256 = U256::from_dec_str("1000000000").unwrap();
    static ref ETHER: U256 = U256::from_dec_str("1000000000000000000").unwrap();
}

pub struct TransactionIdShort(pub TransactionId);
pub struct GWei(pub U256);
pub struct Ether(pub U256);
pub struct Timestamp(pub U256);

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

pub trait HtmlRender {
    fn html(&self) -> TextWithLink;
}

impl HtmlRender for Address {
    fn html(&self) -> TextWithLink {
        TextWithLink::new_link(format!("0x{:x}", self), format!("/0x{:x}", self))
    }
}

impl HtmlRender for Bytes {
    fn html(&self) -> TextWithLink {
        TextWithLink::new_text(
            (self.0)
                .as_slice()
                .chunks(32)
                .map(|c| c.to_hex::<String>())
                .map(|c| format!("{},", c))
                .collect::<String>(),
        )
    }
}

impl HtmlRender for Option<Address> {
    fn html(&self) -> TextWithLink {
        self.map(|v| v.html())
            .unwrap_or_else(|| TextWithLink::new_text("New contract".to_string()))
    }
}

impl HtmlRender for TransactionId {
    fn html(&self) -> TextWithLink {
        match &self {
            TransactionId::Hash(h) => {
                TextWithLink::new_link(format!("0x{:x}", h), format!("/0x{:x}", h))
            }
            _ => unreachable!(),
        }
    }
}

impl HtmlRender for TransactionIdShort {
    fn html(&self) -> TextWithLink {
        match &self.0 {
            TransactionId::Hash(h) => TextWithLink::new_link(
                format!("{:x}", h).chars().take(7).collect::<String>(),
                format!("/0x{:x}", h),
            ),
            _ => unreachable!(),
        }
    }
}

impl HtmlRender for BlockId {
    fn html(&self) -> TextWithLink {
        match &self {
            BlockId::Number(BlockNumber::Number(n)) => {
                TextWithLink::new_link(format!("{}", n), format!("/{}", n))
            }
            _ => unreachable!(),
        }
    }
}

impl HtmlRender for GWei {
    fn html(&self) -> TextWithLink {
        TextWithLink::new_text(format!("{} GWei ({})", self.0 / *GWEI, self.0))
    }
}

impl HtmlRender for Ether {
    fn html(&self) -> TextWithLink {
        if self.0 == U256::zero()  {
            TextWithLink::new_text("0 Ξ".to_string())
        } else {
            let ether  = self.0 / *ETHER;
            let mut remain = self.0 % *ETHER;
            while remain > U256::zero() && remain % 10 == U256::zero() {
                remain /= 10; 
            }
            TextWithLink::new_text(format!("{}.{} Ξ", ether, remain))
        }
    }
}

impl HtmlRender for Timestamp {
    fn html(&self) -> TextWithLink {
        let dt = Utc.timestamp(self.0.low_u64() as i64, 0);
        TextWithLink::new_text(dt.to_rfc2822())
    }
}

pub fn tx_short_json(tx: &Transaction) -> serde_json::Value {
    let shortdata = tx
        .input.0.to_hex::<String>()
        .chars().take(8).collect::<String>();

    let blockid =
        BlockId::Number(BlockNumber::Number(tx.block_number.unwrap().low_u64()));

    json!({
        "blockno"       : blockid.html(),
        "tx"            : TransactionIdShort(TransactionId::Hash(tx.hash)).html(),
        "from"          : tx.from.html(),
        "tonewcontract" : tx.to.is_none(),
        "to"            : if let Some(to) = tx.to { to.html() } else { TextWithLink::blank()},
        "shortdata"     : shortdata,
        "value"         : Ether(tx.value).html()
    })
}
