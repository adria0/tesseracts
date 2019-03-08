#[macro_use]
extern crate rust_embed;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate rouille;
#[macro_use]
extern crate log;
extern crate structopt;

extern crate stderrlog;
extern crate ctrlc;
extern crate handlebars;
extern crate rand;
extern crate rocksdb;
extern crate rustc_hex;
extern crate serde;
extern crate serde_cbor;
extern crate toml;
extern crate web3;
extern crate ethabi;
extern crate reqwest;
extern crate rlp;
extern crate chrono;
extern crate keccak_hash;
extern crate ethkey;

mod db;
mod explorer;
mod scrapper;
mod state;
mod bootstrap;
mod eth;

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;

use structopt::StructOpt;

/// A StructOpt example
#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Verbose mode (-v, -vv, -vvv, etc)
    #[structopt(short = "v", long = "verbose", parse(from_occurrences))]
    verbose: usize,

    /// Timestamp (sec, ms, ns, none)
    #[structopt(short = "t", long = "timestamp")]
    ts: Option<stderrlog::Timestamp>,

    /// Timestamp (sec, ms, ns, none)
    #[structopt(short = "cfg", long = "cfg")]
    cfg: String,
}

fn main() {

    // load cmdline parameters

    let opt = Opt::from_args();
    stderrlog::new()
        .module(module_path!())
        .verbosity(opt.verbose)
        .timestamp(opt.ts.unwrap_or(stderrlog::Timestamp::Off))
        .init()
        .unwrap();

    // load configuration
    let cfg = bootstrap::Config::read(&opt.cfg)
        .expect("cannot read config");

    // create the (arc) global state 
    let globalstate = Arc::new(state::GlobalState::new(cfg).unwrap());

    // start scrap the blockchain (if requiered)
    if globalstate.cfg.scan {
        let shared_ge_scan = globalstate.clone();
        thread::spawn(move || scrapper::start_scrapper(&shared_ge_scan));
    }

    // set the control-c handler
    let shared_ge_controlc = globalstate.clone();
    ctrlc::set_handler(move || {
        info!("Got Ctrl-C handler signal. Stopping...");
        shared_ge_controlc.stop_signal.store(true, Ordering::SeqCst);
        if !shared_ge_controlc.cfg.scan {
            std::process::exit(0);
        }
    })
    .expect("Error setting Ctrl-C handler");

    // start the http server
    info!("Lisening to {}...", &globalstate.cfg.bind.clone()); // TODO: remove clones

    //let shared_ge_explorer = shared_ge.0.clone();
    explorer::start_explorer(globalstate);

}