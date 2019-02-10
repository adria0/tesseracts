# tesseracts
A minimalistic block explorer to learn rust.

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

## Set up

To run tesseracts, you need to install rust nightly, so first install rust with rustup 

`curl https://sh.rustup.rs -sSf | s` 

create a .toml config file (see `cfg.example.toml`)

```toml
# where the database is located
db_path          = 

# web3 json-rpc port, e.g. http://localhost:8545
web3_client      = "geth_clique"
web3_url         = 

# true|false if we want to scan blocks 
scan             =  

# the starting block to start to retrieve blocks (only iff scan==true)
scan_start_block = 

# the path where solc binaries are stored
solc_path = 

# http server binding (e.g. "0.0.0.0:8000")
bind         = 

# multiple named_address entries can be added to name accouts
[[named_address]]
name =    # e.g. "me"
address = # e.g. "0x5d03df716ebf0e11bfb3e178fb39ed672c59ee61"
```

run the application with (if your config file is named `cfg.toml`)

`cargo run -- --cfg cfg.toml`
