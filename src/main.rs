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
mod ui;
mod state;
mod bootstrap;
mod eth;

use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use rouille::{Request,Response};

// it turns out that is not possible to put an Arc inside a rocket::State,
//  rocket internally crashes when unreferencing, so it can be solved by
//  wrapping it inside a one-element tuple
pub struct SharedGlobalState(Arc<state::GlobalState>);

use structopt::StructOpt;

/// A StructOpt example
#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Silence all output
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,
    
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
    let opt = Opt::from_args();

    stderrlog::new()
        .module(module_path!())
        .quiet(opt.quiet)
        .verbosity(opt.verbose)
        .timestamp(opt.ts.unwrap_or(stderrlog::Timestamp::Off))
        .init()
        .unwrap();

    let cfg = bootstrap::Config::read(&opt.cfg)
        .expect("cannot read config");

    let shared_ge = SharedGlobalState(Arc::new(state::GlobalState::new(cfg).unwrap()));
    debug!("non-empty-blocks {:?}",shared_ge.0.db.count_non_empty_blocks());

    if shared_ge.0.cfg.scan {
        let shared_ge_scan = shared_ge.0.clone();
        thread::spawn(move || eth::scan(&shared_ge_scan));
    }

    let shared_ge_controlc = shared_ge.0.clone();
    ctrlc::set_handler(move || {
        info!("Got Ctrl-C handler signal. Stopping...");
        shared_ge_controlc.stop_signal.store(true, Ordering::SeqCst);
        if !shared_ge_controlc.cfg.scan {
            std::process::exit(0);
        }
    })
    .expect("Error setting Ctrl-C handler");

    info!("Lisening to {}...", &shared_ge.0.cfg.bind.clone()); // TODO: remove clones
    rouille::start_server(&shared_ge.0.cfg.bind.clone(), move |request| {
        router!(request,
            (GET)  (/) => {
                ui::get_home(&request,&shared_ge.0)
            },
            (GET)  (/s/{name: String}) => {
                if let Some(res) = bootstrap::get_resource(&name) {
                    rouille::Response::from_data("", res)
                } else {
                    rouille::Response::empty_404()
                }
            },
            (GET)  (/{id: String}) => {
                ui::get_object(&request,&shared_ge.0,&id)
            },
            (POST) (/{id: String}/contract) => {
                let data = try_or_400!(post_input!(request, {
                    contract_source: String,
                    contract_compiler: String,
                    contract_optimized: bool,
                    contract_name: String,
                }));
                ui::post_contract(&shared_ge.0, &id,
                    &data.contract_source, &data.contract_compiler,
                    data.contract_optimized, &data.contract_name
                )
            },
            _ => rouille::Response::empty_404()
        )
    })
}
