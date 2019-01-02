#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rust_embed;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_cbor;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate rocksdb;

extern crate web3;
extern crate rustc_hex;
extern crate handlebars;
extern crate rand;
extern crate ctrlc;

mod render;
mod state;
mod model;
mod db;
mod scanner;
mod stream;

use model::Id;

use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use rocket::response::content;
use rocket::State;
use rustc_hex::{FromHex};
use web3::types::Address;
use web3::types::H256;
use web3::types::BlockId;
use web3::types::BlockNumber;

// it turns out that is not possible to put an Arc inside a rocket::State,
//  rocket internally crashes when unreferencing, so it can be solved by
//  wrapping it inside a one-element tuple
pub struct SharedGlobalState(Arc<state::GlobalState>);

#[get("/")]
fn home(sgs: State<SharedGlobalState>) -> content::Html<String>  {
    render::home(&sgs.0)
}

#[get("/<idstr>")]
fn object(sgs: State<SharedGlobalState>, idstr: String) -> content::Html<String> {
    if let Some(id) = Id::from(idstr) {
        match id {
            Id::Addr(addr) => render::addr_info(&sgs.0,addr),
            Id::Tx(txid) => render::tx_info(&sgs.0,txid),
            Id::Block(block) => render::block_info(&sgs.0,
                BlockId::Number(BlockNumber::Number(block)))
        }
    } else {
        render::page("Not found")
    }
}


fn main() {
   
   let args: Vec<String> = env::args().collect();

   let cfg = state::Config::new(args[1].as_str());
   let shared_ge = SharedGlobalState(Arc::new(state::GlobalState::new(cfg)));

   let scan = false;
   if scan {
       let shared_ge_scan = shared_ge.0.clone();
        thread::spawn(move || {
            scanner::scan(&shared_ge_scan)
        });
   }

   let shared_ge_controlc = shared_ge.0.clone();
   ctrlc::set_handler(move || {
       println!("Got Ctrl-C handler signal. Stopping...");
       shared_ge_controlc.stop_signal.store(true, Ordering::SeqCst);
       if !scan {
           std::process::exit(0);
       }
   }).expect("Error setting Ctrl-C handler");

   rocket::ignite().manage(shared_ge)
        .mount("/", routes![home,object])
        .launch();        
}
 
