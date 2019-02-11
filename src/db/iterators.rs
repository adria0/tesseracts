use rocksdb::{DBIterator};
use web3::types::{H256};
use serde_cbor::{from_slice};

use super::utils::*;
use super::super::types::InternalTx;

impl AddrTxLinks {
    pub fn new(iter: DBIterator, key: Vec<u8>) -> Self {
        AddrTxLinks { iter, key }
    }
}

pub struct AddrTxLinks {
    iter: DBIterator,
    key: Vec<u8>,
}

#[allow(deprecated)]
impl<'a> Iterator for AddrTxLinks {
    type Item = (H256,u64);

    fn next(&mut self) -> Option<(H256,u64)> {
        if let Some((key,_)) = self.iter.next() {
            if key.len() > self.key.len() && key[..self.key.len()] == self.key[..] {
                let tx = H256::from_slice(&key[self.key.len()+16..self.key.len()+16+32]);
                let itx = u64_from_slice(&key[self.key.len()+16+32..self.key.len()+16+32+8]); 
                return Some((tx,std::u64::MAX-itx));
            }
        }
        None
    }
}

pub struct NonEmptyBlocks {
    iter: DBIterator,
    key: Vec<u8>,
}


impl NonEmptyBlocks {
    pub fn new(iter: DBIterator, key: Vec<u8>) -> Self {
        NonEmptyBlocks { iter, key }
    }
}

impl<'a> Iterator for NonEmptyBlocks {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if let Some((k,_)) = self.iter.next() {
            if k.len() > self.key.len() && k[..self.key.len()] == self.key[..] {
                let blockno = u64_from_slice(&k[self.key.len()..]);
                return Some(std::u64::MAX -blockno);
            }
        }
        None
    }
}

pub struct InternalTxs {
    iter: DBIterator,
    key: Vec<u8>,
}

impl InternalTxs {
    pub fn new(iter: DBIterator, key: Vec<u8>) -> Self {
        InternalTxs { iter, key }
    }
}

impl<'a> Iterator for InternalTxs {
    type Item = (u64,InternalTx);

    fn next(&mut self) -> Option<(u64,InternalTx)> {
        if let Some((k,v)) = self.iter.next() {
            if k.len() > self.key.len() && k[..self.key.len()] == self.key[..] {
                let no = std::u64::MAX - u64_from_slice(&k[self.key.len()..]);
                let data : InternalTx = from_slice(&v).unwrap();
                return Some((no,data));
            }
        }
        None
    }
}
