use db;

use std::collections::HashMap;
use web3::types::{Address};
use std::sync::atomic::AtomicBool;
use bootstrap::{Config,load_handlebars_templates};
use handlebars::Handlebars;
use eth::types::hex_to_addr;
use web3::futures::Future;

#[derive(Debug)]
pub enum Error {
    Web3(web3::Error),
    FromHex(rustc_hex::FromHexError),
}

impl From<web3::Error> for Error {
    fn from(err: web3::Error) -> Self {
        Error::Web3(err)
    }
}
impl From<rustc_hex::FromHexError> for Error {
    fn from(err: rustc_hex::FromHexError) -> Self {
        Error::FromHex(err)
    }
}

pub type Result<T> = std::result::Result<T,Error>;

pub struct GlobalState {
    pub stop_signal: AtomicBool,
    pub db: db::AppDB,
    pub cfg: Config,
    pub hb: Handlebars,
    pub named_address : HashMap<Address,String>,
}

pub struct Web3Client {
    pub eloop: web3::transports::EventLoopHandle,
    pub web3: web3::Web3<web3::transports::Http>,
}

impl GlobalState {

    pub fn new(cfg: Config) -> Result<Self>  {
        
        let transport = web3::transports::Http::new(cfg.web3_url.as_str())?;
        let web3 = web3::Web3::new(transport.1);
        let network_id = web3.net().version().wait()?;
        info!("Network id is {}",network_id);

        let mut hb = Handlebars::new();
        load_handlebars_templates(&mut hb);

        // create global stop signal
        let stop_signal = AtomicBool::new(false);

        // load database & init if not
        let db = db::AppDB::open_default(
            &format!("{}{}",cfg.db_path,network_id),
            db::Options {
                store_itx : cfg.db_store_itx,
                store_tx : cfg.db_store_tx,
                store_addr : cfg.db_store_addr,
                store_neb : cfg.db_store_neb,
            }
        ).expect("cannot open database");

        // set the last block if not set
        if None == db.get_next_block_to_scan().expect("error reading last block") {
            db.set_next_block_to_scan(cfg.scan_start_block.unwrap_or(1))
                .expect("error setting last block");
        }

        // read named addresses
        let mut named_address = HashMap::new();
        if let Some(nas) = &cfg.named_address {
            for na in nas {
                named_address.insert(hex_to_addr(&na.address)?, na.name.clone());
            }        
        }

        Ok(GlobalState { cfg, db, hb, stop_signal, named_address })
    }
    pub fn new_web3client(&self) -> Web3Client {
        let (eloop, transport) = web3::transports::Http::new(self.cfg.web3_url.as_str())
            .expect("opening http connection");

        Web3Client {
            eloop,
            web3: web3::Web3::new(transport),
        }
    }

}
