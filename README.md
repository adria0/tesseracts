# rustalleda
A minimalistic block explorer to learn rust.

This is an experimantal block explorer written in rust, the name
comes from the famous catalan chef [Carme Ruscalleda](https://en.wikipedia.org/wiki/Carme_Ruscalleda). At this moment it comes with the folowing features (checked items) and there's a roadmap for the next ones (unchecked items)

- [X] Last blocks page
- [X] Show block
- [X] Show transaction
- [X] Show address and their transactions
- [X] Have a copy of blockchain in the local db
- [X] Gracefull termination with control-C
- [X] Configuration file
- [X] Embeeded templates (does not need external files)
- [ ] Parse clique block headers
- [ ] Define ABI for contract and parse calls and logs
- [ ] Block & Tx pagination
- [ ] Parallel download recipts
- [ ] Copy the database via DevP2P instead Web3
- [ ] See personal ERC20 tokens status
- [ ] Collect ERC20 accounts information
- [ ] User-defined alerts on blockchain data changes
- [ ] Do it graphql friendly

## Set up

To run rustalleda, you need to install rust nightly, so first install rust with rustup 

`curl https://sh.rustup.rs -sSf | s` 

install and set the nightly version to default

`rustup install nightly && rustup default nightly`

create a .toml config file (see `cfg.example.toml`)

```toml
# where the database is located
db_path          = 

# web3 json-rpc port, e.g. http://localhost:8545
web3_url         = 

# true|false if we want to scan blocks 
scan             =  

# the starting block to start to retrieve blocks (only iff scan==true)
scan_start_block = 
```

run the application with (if your config file is named `cfg.toml`)

`cargo run cfg.toml`
