#[derive(RustEmbed)]
#[folder = "tmpl"]
struct Asset;

use db::AppDB;

use std::fs::File;
use std::io::prelude::*;
use std::sync::atomic::AtomicBool;

use handlebars::Handlebars;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub db_path: String,
    pub web3_url: String,
    pub web3_client: String,
    pub scan: bool,
    pub scan_start_block: Option<u64>,
    pub bind: String,
    pub solc_path : String,
}

#[derive(Debug)]
pub enum Error {
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
        Ok(toml::from_str(&contents)?)
    }
    pub fn read_default() -> Result<Self, Error> {
        Config::read("tesseracts.toml")
    }
}

pub struct GlobalState {
    pub stop_signal: AtomicBool,
    pub db: AppDB,
    pub cfg: Config,
    pub hb: Handlebars,
}

pub struct Web3Client {
    pub eloop: web3::transports::EventLoopHandle,
    pub web3: web3::Web3<web3::transports::Http>,
}

impl GlobalState {

    pub fn new(cfg: Config) -> Self {
        
        if cfg.web3_client != GETH_CLIQUE {
            panic!("only {} allowed in web3_client",GETH_CLIQUE);
        }

        let mut hb = Handlebars::new();
        // process assets
        for asset in Asset::iter() {
            let file = asset.into_owned();

            let tmpl = String::from_utf8(
                Asset::get(file.as_str())
                    .unwrap_or_else(|| panic!("Unable to read file {}", file))
                    .to_vec(),
            )
            .unwrap_or_else(|_| panic!("Unable to decode file {}", file));

            hb.register_template_string(file.as_str(), &tmpl)
                .unwrap_or_else(|_| panic!("Invalid template {}", file));
        }

        // create global stop signal
        let stop_signal = AtomicBool::new(false);

        // load database & init if not
        let db = AppDB::open_default(cfg.db_path.as_str()).expect("cannot open database");
        if None == db.get_last_block().expect("error reading last block") {
            db.set_last_block(cfg.scan_start_block.unwrap_or(1))
                .expect("error setting last block");
        }

        GlobalState { cfg, db, hb, stop_signal }
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
