use db::AppDB;

use std::collections::HashMap;
use web3::types::{Address};
use std::sync::atomic::AtomicBool;
use bootstrap::{Config,load_handlebars_templates};
use handlebars::Handlebars;
use types::hex_to_addr;

error_chain! {
    foreign_links {
        Fmt(rustc_hex::FromHexError);
    }    
}

pub struct GlobalState {
    pub stop_signal: AtomicBool,
    pub db: AppDB,
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
        
        let mut hb = Handlebars::new();
        load_handlebars_templates(&mut hb);

        // create global stop signal
        let stop_signal = AtomicBool::new(false);

        // load database & init if not
        let db = AppDB::open_default(cfg.db_path.as_str()).expect("cannot open database");
        if None == db.get_last_block().expect("error reading last block") {
            db.set_last_block(cfg.scan_start_block.unwrap_or(1))
                .expect("error setting last block");
        }

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
