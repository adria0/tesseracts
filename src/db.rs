use web3::types::{Address,Block,H256,U256,U128,BlockId,Transaction,TransactionReceipt,TransactionId,BlockNumber,Bytes};
use rocksdb::{DB,Direction,DBIterator,IteratorMode};
use types::*;
use std::iter;
use rand::{thread_rng,Rng};
use rand::distributions::Alphanumeric;
use serde_cbor::{from_slice,to_vec};

#[derive(Copy,Clone,PartialEq)]
#[repr(u8)]
enum RecordType {
    TxLink    = 1,
    NextBlock = 2,
    Tx        = 3,
    Block     = 4,
    Receipt   = 5,
}

#[derive(Copy,Clone,PartialEq)]
#[repr(u8)]
enum TxLinkType {
    In = 1,
    Out = 2,
    InOut = 3,
    CreateContract = 4,
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

fn u64_to_le(v : u64) -> [u8;8] {
    [
        ((v>>56) & 0xff) as u8,
        ((v>>48) & 0xff) as u8,
        ((v>>40) & 0xff) as u8,
        ((v>>32) & 0xff) as u8,
        ((v>>24) & 0xff) as u8,
        ((v>>16) & 0xff) as u8,
        ((v>>8 ) & 0xff) as u8,
        ((v>>0 ) & 0xff) as u8
    ]
}
fn le_to_u64(v: &[u8;8]) -> u64 {
    (  v[7] as u64) +
    ( (v[6] as u64) << 8  )+
    ( (v[5] as u64) << 16 )+
    ( (v[4] as u64) << 24 )+
    ( (v[3] as u64) << 32 )+
    ( (v[2] as u64) << 40 )+
    ( (v[1] as u64) << 48 )+ 
    ( (v[0] as u64) << 56 )
}


impl<'a> Iterator for AddrTxs {
    type Item = H256;

    fn next(& mut self) -> Option<H256> {
        if let Some(kv) = self.iter.next() {
            let key = &*(kv.0);
            if key.len() > self.key.len() && key[..self.key.len()]==self.key[..] {
                let tx = H256::from_slice(&key[self.key.len()+16..]);
                return Some(tx);
            }
        }
        None
    }
}

#[derive(Debug)]
pub enum Error {
    Rocks(rocksdb::Error),
    SerdeCbor(serde_cbor::error::Error)
}
impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        format!("{:?}",self) == format!("{:?}",other)
    }
}

impl From<rocksdb::Error> for Error {
    fn from(err: rocksdb::Error) -> Self {
        Error::Rocks(err)
    }
}
impl From<serde_cbor::error::Error> for Error {
    fn from(err: serde_cbor::error::Error) -> Self {
        Error::SerdeCbor(err)
    }
}

fn u64_from_slice(v: &[u8]) -> u64 {
    let mut le = [0;8];
    le[..].copy_from_slice(v);
    le_to_u64(&le)
}

impl AppDB {

    pub fn open_default(path : &str) -> Result<AppDB, Error> {
        Ok(DB::open_default(path).map(|x| AppDB { db : x })?)
    }
    
    pub fn push_tx(&self, tx : &Transaction, tr: &TransactionReceipt) -> Result<(),Error> {

        // store tx
        let mut tx_k = vec![RecordType::Tx as u8];
        tx_k.extend_from_slice(&tx.hash);
        self.db.put(tx_k.as_slice(),to_vec(tx).unwrap().as_slice())?;

        // store receipt
        let mut r_k = vec![RecordType::Receipt as u8];
        r_k.extend_from_slice(&tx.hash);
        self.db.put(r_k.as_slice(),to_vec(tr).unwrap().as_slice())?;

        // store receipt -> tx
        let revblockno = u64_to_le(std::u64::MAX - tx.block_number.unwrap().low_u64());
        let revtxindex = u64_to_le(std::u64::MAX -tx.block_number.unwrap().low_u64());

        if let Some(addr) = tr.contract_address {
            let mut contract_k : Vec<u8> = vec![RecordType::TxLink as u8];
            contract_k.extend_from_slice(&addr);
            contract_k.extend_from_slice(&revblockno);
            contract_k.extend_from_slice(&revtxindex);
            contract_k.extend_from_slice(&tx.hash);
            self.db.put(&contract_k.to_owned(),&[TxLinkType::CreateContract as u8])?;
        }

        // store addr->tx
        if let Some(to) = tx.to {
            let mut to_k : Vec<u8> = vec![RecordType::TxLink as u8];
            to_k.extend_from_slice(&to);
            to_k.extend_from_slice(&revblockno);
            to_k.extend_from_slice(&revtxindex);
            to_k.extend_from_slice(&tx.hash);
            
            let link_type = if tx.from==to { TxLinkType::InOut } else { TxLinkType::Out };
            self.db.put(to_k.as_slice(),&[link_type as u8])?;
            if link_type == TxLinkType::InOut  {
                return Ok(());
            }
        }

        let mut from_k : Vec<u8> = vec![RecordType::TxLink as u8];
        from_k.extend_from_slice(&tx.from);
        from_k.extend_from_slice(&revblockno);
        from_k.extend_from_slice(&revtxindex);
        from_k.extend_from_slice(&tx.hash);

        Ok(self.db.put(&from_k.to_owned(),&[TxLinkType::In as u8])?)
    }

    pub fn get_tx(&self, txhash : &H256) -> Result<Option<Transaction>,Error> {
        let mut tx_k = vec![RecordType::Tx as u8];
        tx_k.extend_from_slice(&txhash);
        Ok(self.db.get(&tx_k).map(|bytes| {
            bytes.map(|v| from_slice::<Transaction>(&*v).unwrap())
        })?)
    }

    pub fn get_receipt(&self, txhash : &H256) -> Result<Option<TransactionReceipt>,Error> {
        let mut tx_k = vec![RecordType::Receipt as u8];
        tx_k.extend_from_slice(&txhash);
        Ok(self.db.get(&tx_k).map(|bytes| {
            bytes.map(|v| from_slice::<TransactionReceipt>(&*v).unwrap())
        })?)
    }

    pub fn push_block(&self, block: &Block<H256>) -> Result<(),Error> {
        let mut b_k = vec![RecordType::Block as u8];
        let block_no = u64_to_le(block.number.unwrap().low_u64());
        b_k.extend_from_slice(&block_no);

        Ok(self.db.put(b_k.as_slice(),to_vec(block)?.as_slice())?)
    }

    pub fn get_block(&self, blockno : u64) -> Result<Option<Block<H256>>,Error> {
        let mut b_k = vec![RecordType::Block as u8];
        b_k.extend_from_slice(&u64_to_le(blockno));
        Ok(self.db.get(&b_k).map(|bytes| {
            bytes.map(|v| from_slice::<Block<H256>>(&*v).unwrap())
        })?)
    }

    pub fn iter_addr_txs<'a>(&self, addr: &Address) -> AddrTxs {
        let mut key : Vec<u8> = vec![];
        key.push(RecordType::TxLink as u8);
        key.extend_from_slice(addr);
        
        let iter = self.db.iterator(
            IteratorMode::From(&key,Direction::Forward));

        AddrTxs::new(iter,key)
    }

    pub fn get_last_block(&self) -> Result<Option<u64>,Error> {
        Ok(self.db.get(&[RecordType::NextBlock as u8]).map(|bytes| {
            bytes.map(|v| u64_from_slice(&*v))
        })?)
    }

    pub fn set_last_block(&self, n : u64) -> Result<(),Error> {
        Ok(self.db.put(
            &[RecordType::NextBlock as u8],
            &u64_to_le(n)
        )?)
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
        let a3 = hex_to_addr("0x1eb983836ea12dc37cc4da2effae9c9fbd0b395c").unwrap();
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

        let tr = TransactionReceipt{
            block_hash: None,
            block_number: Some(U256::from_dec_str("10").unwrap()),
            transaction_index: one_u128,
            contract_address: Some(a3),
            gas_used:one_u256 ,
            cumulative_gas_used: one_u256,
            status:  None,
            transaction_hash: h1,
            logs : Vec::new(),
        };

        assert_eq!(Ok(()),appdb.push_tx(&tx,&tr));

        let mut it_a1 = appdb.iter_addr_txs(&a1);
        assert_eq!(Some(h1), it_a1.next());
        assert_eq!(None, it_a1.next());

        let mut it_a2 = appdb.iter_addr_txs(&a2);
        assert_eq!(Some(h1), it_a2.next());
        assert_eq!(None, it_a2.next());

        let mut it_a3 = appdb.iter_addr_txs(&a3);
        assert_eq!(Some(h1), it_a3.next());
        assert_eq!(None, it_a3.next());

    } 


    #[test]
    fn test_set_get_block() {
        let appdb = init();
        assert_eq!(Ok(None), appdb.get_last_block());
        assert_eq!(Ok(()), appdb.set_last_block(1));
        assert_eq!(Ok(Some(1)), appdb.get_last_block());
        assert_eq!(Ok(()), appdb.set_last_block(0xaabbccdd11223344));
        assert_eq!(Ok(Some(0xaabbccdd11223344)), appdb.get_last_block());
    }
}