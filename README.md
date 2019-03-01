![](https://www.rust-lang.org/logos/rust-logo-32x32-blk.png)
[![Build Status](https://travis-ci.org/adriamb/tesseracts.svg?branch=master)](https://travis-ci.org/adriamb/tesseracts)
[![Docker build](https://img.shields.io/docker/automated/adriamb/tesseracts.svg?style=flat)](https://cloud.docker.com/repository/docker/adriamb/tesseracts)

# Tesseracts
A minimalistic block explorer initially created to learn rust.

This block explorer has been created as a rust self-learning project to give support to [nou.network](https://nou.network), a small beta PoA for social projects with nodes from university teachers ([UPC](https://www.upc.edu), [UAB](https://www.uab.edu), [UOC](https://www.uoc.edu), [UdG](https://www.udg.edu), [UIB](https://www.uib.es/es)), [GuifiNet](https://guifi.net/en) and members the [White Hat Group](https://giveth.io/#heronav).

## Disclaimer

This is an experimental block explorer, my first attempt to write something in rust, and expect to find newbie rustacean antipatterns here. Nonetheless it seems that it works as expected.

## Features

![screenshot](https://raw.githubusercontent.com/adriamb/tesseracts/master/extra/screenshot.png)

At this moment it comes with the folowing features (checked items) and there's a roadmap for the next ones (unchecked items)

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
