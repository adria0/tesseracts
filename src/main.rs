#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
extern crate web3;
extern crate rustc_hex;
mod render;
mod state;

use std::env;
use rocket::response::content;
use rocket::State;

use rustc_hex::{FromHex};

use web3::types::Address;
use web3::types::H256;
use web3::types::BlockId;
use web3::types::BlockNumber;

enum Id {
    Addr(Address),
    Tx(H256),
    Block(BlockId)
}
impl Id {
    fn from(id : String) -> Option<Self> {
        if id.len() == 42 /* address */ {
            let hex : String = id.chars().skip(2).collect();
            let addr : Address = hex.as_str()
                .from_hex::<Vec<u8>>()
                .map(|v| Address::from_slice(&v))
                .expect("Error decoding value");
            Some(Id::Addr(addr))
        } else if id.len() == 66 /* tx */ {
            let hex : String = id.chars().skip(2).collect();
            let txid : H256 = hex.as_str()
                .from_hex::<Vec<u8>>()
                .map(|v| H256::from_slice(&v))
                .expect("Error decoding value");
            Some(Id::Tx(txid))
        } else {
            let blockno_u64 = id.parse::<u64>().unwrap();
            let blockno = BlockNumber::Number(blockno_u64);
            Some(Id::Block(BlockId::Number(blockno)))
        }
    }
}

#[get("/")]
fn home(cfg: State<state::Config>) -> content::Html<String>  {
    render::home(&cfg.gen_state())
}

#[get("/<idstr>")]
fn object(cfg: State<state::Config>, idstr: String) -> content::Html<String> {
    if let Some(id) = Id::from(idstr) {
        let st = cfg.gen_state();
        match id {
            Id::Addr(addr) => render::addr_info(&st,addr),
            Id::Tx(txid) => render::tx_info(&st,txid),
            Id::Block(block) => render::block_info(&st,block)
        }
    } else {
        render::page("Not found")
    }
}

fn main() {
   let args: Vec<String> = env::args().collect();
    rocket::ignite()
        .manage(state::Config::new(args[1].as_str()))
        .mount("/", routes![home,object])
        .launch();        
}

