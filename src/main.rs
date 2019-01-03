#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use] extern crate rocket;
#[macro_use] extern crate rust_embed;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;

extern crate serde;
extern crate serde_cbor;
extern crate rocksdb;
extern crate toml;
extern crate web3;
extern crate rustc_hex;
extern crate handlebars;
extern crate rand;
extern crate ctrlc;

mod reader;
mod render;
mod state;
mod types;
mod db;
mod scanner;

use std::env;
use std::sync::atomic::{Ordering};
use std::sync::Arc;
use std::thread;

use rocket::response::content;
use rocket::State;

use types::Id;

// it turns out that is not possible to put an Arc inside a rocket::State,
//  rocket internally crashes when unreferencing, so it can be solved by
//  wrapping it inside a one-element tuple
pub struct SharedGlobalState(Arc<state::GlobalState>);

#[get("/")]
fn home(sgs: State<SharedGlobalState>) -> content::Html<String>  {
    let wc = sgs.0.new_web3client();
    let reader = reader::BlockchainReader::new(&wc,&sgs.0.db);
    match render::home(&reader,&sgs.0.hb) {
        Ok(html) => html,
        Err(err) => render::page(format!("Error: {:?}", err).as_str())
    }
}

#[get("/<idstr>")]
fn object(sgs: State<SharedGlobalState>, idstr: String) -> content::Html<String> {
    let wc = sgs.0.new_web3client();
    let reader = reader::BlockchainReader::new(&wc,&sgs.0.db);

    if let Some(id) = Id::from(idstr) {
        let html = match id {
            Id::Addr(addr) => render::addr_info(&reader,&sgs.0.hb,&addr),
            Id::Tx(txid) => render::tx_info(&reader,&sgs.0.hb,txid),
            Id::Block(block) => render::block_info(&reader,&sgs.0.hb,block)
        };
        match html {
            Ok(html) => html,
            Err(err) => render::page(format!("Error: {:?}", err).as_str())
        }
    } else {
        render::page("Not found")
    }
}

fn main() {

   let args: Vec<String> = env::args().collect();
   let cfg = if args.len() > 1 {
       state::Config::read(args[1].as_str()) 
   } else {
       state::Config::read_default() 
   }.expect("cannot read config");

   let shared_ge = SharedGlobalState(Arc::new(state::GlobalState::new(cfg)));

   if shared_ge.0.cfg.scan {
       let shared_ge_scan = shared_ge.0.clone();
        thread::spawn(move || {
            scanner::scan(&shared_ge_scan)
        });
   }

   let shared_ge_controlc = shared_ge.0.clone();
   ctrlc::set_handler(move || {
       println!("Got Ctrl-C handler signal. Stopping...");
       shared_ge_controlc.stop_signal.store(true, Ordering::SeqCst);
       if !shared_ge_controlc.cfg.scan{
           std::process::exit(0);
       }
   }).expect("Error setting Ctrl-C handler");

   rocket::ignite().manage(shared_ge)
        .mount("/", routes![home,object])
        .launch();        
}
 
