use rustc_hex::{FromHex,FromHexError};
use web3::types::{H256,Address,U256};

#[derive(Serialize)]
pub enum Id {
    Addr(Address),
    Tx(H256),
    Block(u64)
}

pub fn hex_to_addr(s : &str) -> Result<Address,FromHexError> {
    s.to_owned()
        .chars()
        .skip(2)
        .collect::<String>()
        .from_hex::<Vec<u8>>()
        .map(|v| Address::from_slice(&v))
}

pub fn hex_to_h256(s : &str) -> Result<H256,FromHexError> {
    s.to_owned()
        .chars()
        .skip(2)
        .collect::<String>()
        .from_hex::<Vec<u8>>()
        .map(|v| H256::from_slice(&v))
}

impl Id {
    pub fn from(id : String) -> Option<Self> {
        if id.len() == 42 /* address */ {
            hex_to_addr(id.as_str()).map(|addr| Id::Addr(addr)).ok()
        } else if id.len() == 66 /* tx */ {
            hex_to_h256(id.as_str()).map(|h| Id::Tx(h)).ok()
        } else if let Ok(blockno_u64) = id.parse::<u64>() {
            Some(Id::Block(blockno_u64))
        } else {
            None
        }
    }
}