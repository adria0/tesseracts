use rustc_hex::{FromHex};

use web3::types::Address;
use web3::types::H256;
use web3::types::BlockId;
use web3::types::BlockNumber;

use serde::ser::{Serialize, Serializer};

#[derive(Serialize)]
pub enum Id {
    Addr(Address),
    Tx(H256),
    Block(u64)
}

pub enum LinkType {
    IsTxFrom,
    IsTxTo,
}

impl LinkType {
    pub fn id(&self) -> u8 {
        match self {
            LinkType::IsTxFrom => 0,
            LinkType::IsTxTo => 1,
        }
    }
}

impl Id {
    pub fn from(id : String) -> Option<Self> {
        if id.len() == 42 /* address */ {
            let hex : String = id.chars().skip(2).collect();
            let addr : Address = hex.as_str()
                .from_hex::<Vec<u8>>()
                .map(|v| Address::from_slice(&v))
                .expect("unable to parse address");
            Some(Id::Addr(addr))
        } else if id.len() == 66 /* tx */ {
            let hex : String = id.chars().skip(2).collect();
            let txid : H256 = hex.as_str()
                .from_hex::<Vec<u8>>()
                .map(|v| H256::from_slice(&v))
                .expect("unable to parse tx");
            Some(Id::Tx(txid))
        } else if let Ok(blockno_u64) = id.parse::<u64>() {
//            let blockno = BlockNumber::Number(blockno_u64);
            Some(Id::Block(blockno_u64))
        } else {
            None
        }
    }
}
