use web3::types::{Address,H256,U256,U128,BlockId,Transaction,TransactionId,BlockNumber,Bytes};
use rocksdb::{DB,Direction,DBIterator,IteratorMode};
use model::*;
use std::iter;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

#[derive(Copy,Clone,PartialEq)]
#[repr(u8)]
enum RecordType {
    TxLink = 1,
    NextBlock = 2,
}

#[derive(Copy,Clone,PartialEq)]
#[repr(u8)]
enum TxLinkType {
    In = 1,
    Out = 2,
    InOut = 3
}

pub struct AppDB {
    db  : DB,
}

pub struct AddrTxs {
    iter : DBIterator,
    key  : Vec<u8>,
}

impl AddrTxs {
    fn new(iter : DBIterator, key : Vec<u8> ) -> Self {
        AddrTxs{iter,key}
    }
}

impl<'a> Iterator for AddrTxs {
    type Item = H256;

    fn next(& mut self) -> Option<H256> {
        if let Some(kv) = self.iter.next() {
            let key = &*(kv.0);
            if key.len() > self.key.len() && key[..self.key.len()]==self.key[..] {
                // unserialize blockno, txindex 
                return Some(H256::from_slice(&key[self.key.len()+16..]));
            }
        }
        None
    }
}

impl AppDB {
    
    pub fn open_default(path : &str) -> Result<AppDB, rocksdb::Error> {
        match DB::open_default(path) {
            Err(err) => Err(err),
            Ok(db) => Ok(AppDB { db : db })
        }        
    }
    
    pub fn push_tx(&self, tx : &Transaction) -> Result<(),rocksdb::Error> {

        let blockno = tx.block_number.unwrap().low_u64().to_le_bytes();
        let txindex = tx.block_number.unwrap().low_u64().to_le_bytes();

        if let Some(to) = tx.to {
            let mut tok : Vec<u8> = vec![];
            tok.push(RecordType::TxLink as u8);
            tok.extend_from_slice(&to);
            tok.extend_from_slice(&blockno);
            tok.extend_from_slice(&txindex);
            tok.extend_from_slice(&tx.hash);
            
            let link_type = if tx.from==to { TxLinkType::InOut } else { TxLinkType::Out };
            if let Err(err) = self.db.put(&tok.to_owned(),&[link_type as u8]) {
                return Err(err);
            }
            if link_type == TxLinkType::InOut  {
                return Ok(());
            }
        }

        let mut fromk : Vec<u8> = vec![];
        fromk.push(RecordType::TxLink as u8);
        fromk.extend_from_slice(&tx.from);
        fromk.extend_from_slice(&blockno);
        fromk.extend_from_slice(&txindex);
        fromk.extend_from_slice(&tx.hash);
        self.db.put(&fromk.to_owned(),&[TxLinkType::In as u8])

    }

    pub fn iter_addr_txs<'a>(&self, addr: &Address) -> AddrTxs {
        let snapshot = self.db.snapshot();

        let mut key : Vec<u8> = vec![];
        key.push(RecordType::TxLink as u8);
        key.extend_from_slice(addr);
        
        let iter = self.db.iterator(
            IteratorMode::From(&key,Direction::Forward));

        AddrTxs::new(iter,key)
    }

    pub fn get_last_block(&self) -> Result<Option<u64>,rocksdb::Error> {
        match self.db.get(&[RecordType::NextBlock as u8]) {
            Ok(Some(value)) => {
                let mut le = [0;8];
                le[..].copy_from_slice(&*value);
                Ok(Some(u64::from_le_bytes(le)))
            }
            Ok(None)        => Ok(None),
            Err(e)          => Err (e),
        }
    }

    pub fn set_last_block(&self, n : u64) -> Result<(),rocksdb::Error> {
        self.db.put(&[RecordType::NextBlock as u8],&n.to_le_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init() -> AppDB {
        let mut rng = thread_rng();
        let chars: String = iter::repeat(())
                .map(|()| rng.sample(Alphanumeric))
                .take(7)
                .collect();
        
        let mut tmpfile= std::env::temp_dir();
        tmpfile.push(chars);
        AppDB::open_default(
            tmpfile.as_os_str().to_str().expect("bad OS filename")
        ).expect("unable to create db")
    }

    #[test]
    fn test_add_and_iter() {
        let appdb = init();
        let one_u128 = U128::from_dec_str("1").unwrap();
        let one_u256 = U256::from_dec_str("1").unwrap();
        let a1 = hex_to_addr("0x1eb983836ea12dc37cc4da2effae9c9fbd0b395a").unwrap();
        let a2 = hex_to_addr("0x1eb983836ea12dc37cc4da2effae9c9fbd0b395b").unwrap();
        let h1 = hex_to_h256("0xd69fc1890a1b2742b5c2834d031e34ba55ef3820d463a8d0a674bb5dd9a3b74b").unwrap();

        let tx = Transaction{
            hash: h1,
            nonce: one_u256,
            block_hash: None,
            block_number: Some(U256::from_dec_str("10").unwrap()),
            transaction_index: Some(one_u128),
            from: a1,
            to: Some(a2),
            value: one_u256,
            gas_price: one_u256,
            gas: one_u256,
            input: Bytes(Vec::new()),            
        };

        assert_eq!(Ok(()),appdb.push_tx(&tx));

        let mut it_a1 = appdb.iter_addr_txs(&a1);
        assert_eq!(Some(h1), it_a1.next());
        assert_eq!(None, it_a1.next());

        let mut it_a2 = appdb.iter_addr_txs(&a2);
        assert_eq!(Some(h1), it_a2.next());
        assert_eq!(None, it_a2.next());
    }

    #[test]
    fn test_set_get_block() {
        let appdb = init();
        assert_eq!(Ok(None), appdb.get_last_block());
        assert_eq!(Ok(()), appdb.set_last_block(1));
        assert_eq!(Ok(Some(1)), appdb.get_last_block());
        assert_eq!(Ok(()), appdb.set_last_block(2));
        assert_eq!(Ok(Some(2)), appdb.get_last_block());
    }
}