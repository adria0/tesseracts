use std::fs::File;
use std::io::prelude::*;
use std::sync::atomic::AtomicBool;

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

#[derive(Debug)]
pub enum Error {
    InvalidOption(String),
    Io(std::io::Error),
    Toml(toml::de::Error),
}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Toml(err)
    }
}

const GETH_CLIQUE : &'static str = "geth_clique";

impl Config {
    pub fn read(path: &str) -> Result<Self, Error> {
        let mut contents = String::new();
        File::open(path)?.read_to_string(&mut contents)?;
        let cfg : Config = toml::from_str(&contents)?;

        if cfg.web3_client != GETH_CLIQUE {
            Err(Error::InvalidOption(format!("only {} allowed in web3_client",GETH_CLIQUE)))
        } else {
            Ok(cfg)
        }
    }
}

