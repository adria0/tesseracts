#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rust_embed;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde;
#[macro_use] extern crate serde_cbor;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;

extern crate web3;
extern crate rustc_hex;
extern crate handlebars;

mod render;
mod state;
mod model;
mod db;

use std::env;
use rocket::response::content;
use rocket::State;

use model::Id;

use rustc_hex::{FromHex};

use web3::types::Address;
use web3::types::H256;
use web3::types::BlockId;
use web3::types::BlockNumber;

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
            Id::Block(block) => render::block_info(&gs,
                BlockId::Number(BlockNumber::Number(block)))
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
 
