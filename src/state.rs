#[derive(RustEmbed)]
#[folder = "tmpl"]
struct Asset;  

use handlebars::Handlebars;
use std::sync::atomic::{AtomicBool};
use db::AppDB;

pub struct Config {
    pub web3_url : String,
}

impl Config {
    pub fn new(web3_url : &str) -> Self {
        Config { web3_url : web3_url.to_string() }
    }
}

pub struct GlobalState {
    pub stop_signal : AtomicBool,
    pub db : AppDB,
    pub cfg : Config,
    pub tmpl : Handlebars,
}

pub struct LocalState<'a> {
    pub gs: &'a GlobalState,
    pub eloop : web3::transports::EventLoopHandle,
    pub web3 : web3::Web3<web3::transports::Http>,    
}

impl GlobalState {
    pub fn new(cfg: Config) -> Self {
        let mut reg = Handlebars::new();

        // process assets
        for asset in Asset::iter() {
            let file = asset.into_owned();
            
            let tmpl = String::from_utf8(
                Asset::get(file.as_str())
                .expect(&format!("Unable to read file {}",file))
                .to_vec()
            ).expect(&format!("Unable to decode file {}",file));

            reg.register_template_string(file.as_str(), &tmpl)
                .expect(&format!("Invalid template {}",file));
        }

        // create global stop signal
        let stop_signal = AtomicBool::new(false);

        // load database & init if not
        let db = AppDB::open_default("./db").expect("cannot open database");
        if None == db.get_last_block().expect("error reading last block") {
            db.set_last_block(3000000).expect("error setting last block");
        }

        GlobalState{ tmpl : reg, cfg: cfg, db: db, stop_signal : stop_signal }

    }
    pub fn create_local(&self) -> LocalState {
        let (eloop, transport) =
            web3::transports::Http::new(self.cfg.web3_url.as_str())
            .expect("opening http connection");

        LocalState { gs: self, eloop  : eloop, web3 : web3::Web3::new(transport) }
    }
}