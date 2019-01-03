#[derive(RustEmbed)]
#[folder = "tmpl"]
struct Asset;  
 
use db::AppDB;

use std::sync::atomic::{AtomicBool};
use std::fs::File;
use std::io::prelude::*;

use handlebars::Handlebars;

#[derive(Deserialize,Debug)]
pub struct Config {
    pub db_path          : String,
    pub web3_url         : String,
    pub scan             : bool,
    pub scan_start_block : Option<u64>,
}

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Toml(toml::de::Error)
}
impl From<std::io::Error> for Error {
    fn from(err : std::io::Error) -> Self {
        Error::Io(err)
    }
}
impl From<toml::de::Error> for Error {
    fn from(err : toml::de::Error) -> Self {
        Error::Toml(err)
    }
}

impl Config {
    pub fn read(path : &str) -> Result<Self,Error> {
        let mut contents = String::new();
        File::open(path)?
            .read_to_string(&mut contents)?;
        Ok(toml::from_str(&contents)?)
    }
    pub fn read_default() -> Result<Self,Error> {
        Config::read("rchain.toml")
    }
}

pub struct GlobalState {
    pub stop_signal : AtomicBool,
    pub db : AppDB,
    pub cfg : Config,
    pub hb : Handlebars,
}

pub struct Web3Client {
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
        let db = AppDB::open_default(cfg.db_path.as_str()).expect("cannot open database");
        if None == db.get_last_block().expect("error reading last block") {
            db.set_last_block(cfg.scan_start_block.unwrap_or(1))
                .expect("error setting last block");
        }

        GlobalState{ hb : reg, cfg: cfg, db: db, stop_signal : stop_signal }

    }
    pub fn new_web3client(&self) -> Web3Client {
        let (eloop, transport) =
            web3::transports::Http::new(self.cfg.web3_url.as_str())
            .expect("opening http connection");

        Web3Client { eloop  : eloop, web3 : web3::Web3::new(transport) }
    }
}