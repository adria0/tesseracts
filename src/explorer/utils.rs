use web3::types::{Address,Block,H160};

use super::super::bootstrap::{Config,GETH_CLIQUE,GETH_POW,GETH_AUTO};
use super::super::geth::parse_clique_header;

#[derive(Debug)]
pub struct Pagination {
    pub from : u64,
    pub to   : u64,
    pub prev_page : Option<u64>,
    pub next_page : Option<u64>,
}

pub fn paginate( max : u64, page_size : u64, page_no : u64 ) -> Pagination {
    let from = page_no*page_size;
    let to = if (page_no+1)*page_size > max {
        max
    } else {
        (page_no+1)*page_size
    };
    let prev_page = if page_no > 0 {
        Some(page_no-1)
    } else {
        None
    };
    let next_page = if to < max {
        Some(page_no+1)
    } else {
        None
    };
    Pagination{from,to,prev_page,next_page}
}

pub fn author<T>(cfg: &Config, block: &Block<T>) -> Address {
    
    let client = if &cfg.web3_client == GETH_AUTO {
        if block.author == H160::default() {
            GETH_CLIQUE
        } else {
            GETH_POW
        }
    } else {
        &cfg.web3_client
    };
    
    if client == GETH_CLIQUE {       
        parse_clique_header(&block).unwrap()
    } else {
        block.author
    }

    
}
