pub struct Config {
    pub web3_url : String,
}

pub struct State {
    pub eloop : web3::transports::EventLoopHandle,
    pub web3 : web3::Web3<web3::transports::Http>,
}

impl Config {
    pub fn new(web3_url : &str) -> Self {
        Config { web3_url : web3_url.to_string() }
    }
    pub fn gen_state(&self) -> State {
        let (eloop, transport) = web3::transports::Http::new(self.web3_url.as_str()).unwrap();
        State { eloop  : eloop, web3 : web3::Web3::new(transport) }
    }
}
