use std::fs::File;
use std::io::prelude::*;

use super::error::{Error,Result};

const GETH_CLIQUE : &'static str = "geth_clique";

#[derive(Debug, Deserialize)]
pub struct NamedAddress {
    pub address: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Config {
    pub db_path: String,
    pub web3_url: String,
    pub web3_client: String,
    pub scan: bool,
    pub scan_start_block: Option<u64>,
    pub bind: String,
    pub solc_path : String,
    pub named_address : Option<Vec<NamedAddress>>,    
}

impl Config {
    pub fn read(path: &str) -> Result<Self> {
        let mut contents = String::new();
        File::open(path)?.read_to_string(&mut contents)?;
        let cfg : Config = toml::from_str(&contents)?;

        if cfg.web3_client != GETH_CLIQUE {
            return Err(Error::InvalidOption(format!("only {} allowed in web3_client",GETH_CLIQUE)))
        } else {
            Ok(cfg)
        }
    }
}

