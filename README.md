[![Build Status](https://travis-ci.org/adriamb/tesseracts.svg?branch=master)](https://travis-ci.org/adriamb/tesseracts)
[![Docker build](https://img.shields.io/docker/automated/adriamb/tesseracts.svg?style=flat)](https://cloud.docker.com/repository/docker/adriamb/tesseracts)

# tesseracts
A minimalistic block explorer initially created to learn rust.

![screenshot](https://raw.githubusercontent.com/adriamb/tesseracts/master/extra/screenshot.png)


This is an experimantal block explorer written in rust. At this moment it comes with the folowing features (checked items) and there's a roadmap for the next ones (unchecked items)

- [X] Last blocks page
- [X] Show block
- [X] Show transaction
- [X] Show address and their transactions
- [X] Have a copy of blockchain in the local db
- [X] Gracefull termination with control-C
- [X] Configuration file
- [X] Embeeded templates (does not need external files)
- [X] Upload contracts and parse calls and logs
- [X] Block & Tx pagination
- [X] Command line parameters with better debug 
- [X] Internal transactions
- [X] Parse clique block headers
- [X] Named accounts
- [ ] Download receipts in batch
- [ ] Forward-backwards block scanning 
- [ ] Set postly URL... `/tx` `/addr` `/block`
- [ ] Automatic ERC20 parsing `/erc20`
- [ ] Suport for user configuration
  - [ ] Naming addresses support
  - [ ] Specify token address

## Set up

To run tesseracts, you need to install rust nightly, so first install rust with rustup 

`curl https://sh.rustup.rs -sSf | sh` 

create a .toml config file (see `cfg.example.toml`)

run the application with (if your config file is named `cfg.toml`)

`cargo run -- --cfg cfg.toml`
