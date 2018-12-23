#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rust_embed;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate web3;
extern crate rustc_hex;
extern crate handlebars;

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
                .expect("unable to parse address");
            Some(Id::Addr(addr))
        } else if id.len() == 66 /* tx */ {
            let hex : String = id.chars().skip(2).collect();
            let txid : H256 = hex.as_str()
                .from_hex::<Vec<u8>>()
                .map(|v| H256::from_slice(&v))
                .expect("unable to parse tx");
            Some(Id::Tx(txid))
        } else if let Ok(blockno_u64) = id.parse::<u64>() {
            let blockno = BlockNumber::Number(blockno_u64);
            Some(Id::Block(BlockId::Number(blockno)))
        } else {
            None
        }
    }
}

#[get("/")]
fn home(gs: State<state::GlobalState>) -> content::Html<String>  {
    render::home(&gs)
}

#[get("/<idstr>")]
fn object(gs: State<state::GlobalState>, idstr: String) -> content::Html<String> {
    if let Some(id) = Id::from(idstr) {
        match id {
            Id::Addr(addr) => render::addr_info(&gs,addr),
            Id::Tx(txid) => render::tx_info(&gs,txid),
            Id::Block(block) => render::block_info(&gs,block)
        }
    } else {
        render::page("Not found")
    }
}

fn main() {
   let args: Vec<String> = env::args().collect();
   let cfg = state::Config::new(args[1].as_str());
   rocket::ignite()
        .manage(state::GlobalState::new(cfg))
        .mount("/", routes![home,object])
        .launch();        
}
 
