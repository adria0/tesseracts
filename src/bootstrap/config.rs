use std::fs::File;
use std::io::prelude::*;

use super::error::{Error,Result};

/// allowed parameters for web3_client 
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
    /// the main page title
    pub ui_title: String,

    /// database location path
    pub db_path: String,

    /// flag to store internal transactions
    pub db_store_itx : bool,

    /// flag to store transaction
    pub db_store_tx : bool,

    /// flag to store addresses in txs
    pub db_store_addr : bool,

    /// flag to store non-empty blocks
    pub db_store_neb : bool,

    /// flag to store non-empty blocks
    pub web3_url: String,

    /// web3 client to use
    pub web3_client: String,

    /// flag is web3 is able to get internal transactions
    pub web3_itx: bool,

    /// flag to scan transactions
    pub scan: bool,

    /// when scan is true, the first block to scan
    pub scan_start_block: Option<u64>,

    /// network ip:port binding
    pub bind: String,

    /// path of solc (optional)
    pub solc_path : Option<String>,

    /// allow abi when adding contracts
    pub solc_bypass : bool,

    /// set of named addresses
    pub named_address : Option<Vec<NamedAddress>>,    
}

impl Config {
    /// read the .toml configutation 
    pub fn read(path: &str) -> Result<Self> {

        // read the contents
        let mut contents = String::new();
        File::open(path)?.read_to_string(&mut contents)?;

        // parse and check parameters
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

