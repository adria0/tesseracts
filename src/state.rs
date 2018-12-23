#[derive(RustEmbed)]
#[folder = "tmpl"]
struct Asset;  

use handlebars::Handlebars;

pub struct Config {
    pub web3_url : String,
}
impl Config {
    pub fn new(web3_url : &str) -> Self {
        Config { web3_url : web3_url.to_string() }
    }
}

pub struct GlobalState {
    pub cfg  : Config,
    pub tmpl : Handlebars,
}
pub struct LocalState {
    pub eloop : web3::transports::EventLoopHandle,
    pub web3 : web3::Web3<web3::transports::Http>,    
}

impl GlobalState {
    pub fn new(cfg: Config) -> Self {
        let mut reg = Handlebars::new();
        for asset in Asset::iter() {
            let file = asset.into_owned();
            let tmpl = String::from_utf8(Asset::get(file.as_str()).unwrap().to_vec());   
            reg.register_template_string(file.as_str(), &tmpl.unwrap()).unwrap();
        }
        GlobalState{ tmpl : reg, cfg: cfg }
    }
    pub fn create_local(&self) -> LocalState {
        let (eloop, transport) = web3::transports::Http::new(self.cfg.web3_url.as_str()).unwrap();
        LocalState { eloop  : eloop, web3 : web3::Web3::new(transport) }
    }
}
