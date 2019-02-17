use std::fs::File;
use std::io::prelude::*;

use super::error::{Error,Result};

pub const GETH_CLIQUE : & str = "geth_clique";
pub const GETH_POW    : & str = "geth_pow";
pub const GETH_AUTO   : & str = "geth";

#[derive(Debug, Deserialize)]
pub struct NamedAddress {
    pub address: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub db_path: String,
    pub db_store_itx : bool,
    pub db_store_tx : bool,
    pub db_store_addr : bool,
    pub db_store_neb : bool,
    pub web3_url: String,
    pub web3_client: String,
    pub web3_itx: bool,
    pub scan: bool,
    pub scan_start_block: Option<u64>,
    pub bind: String,
    pub solc_path : Option<String>,
    pub solc_bypass : bool,
    pub named_address : Option<Vec<NamedAddress>>,    
}

impl Config {
    pub fn read(path: &str) -> Result<Self> {
        let mut contents = String::new();
        File::open(path)?.read_to_string(&mut contents)?;
        let cfg : Config = toml::from_str(&contents)?;

        if cfg.web3_client != GETH_CLIQUE
           && cfg.web3_client != GETH_POW
           && cfg.web3_client != GETH_AUTO {
            Err(Error::InvalidOption(format!("only {} allowed in web3_client",GETH_CLIQUE)))
        } else {
            Ok(cfg)
        }
    }
}

