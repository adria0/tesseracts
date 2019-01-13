use rocksdb::{DBIterator, Direction, IteratorMode, DB};
use serde_cbor::{from_slice, to_vec};
use web3::types::{Address, Block, Transaction, TransactionReceipt, H256};

#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
enum RecordType {
    TxLink = 1,
    NextBlock = 2,
    Tx = 3,
    Block = 4,
    Receipt = 5,
    ContractAbi = 6,
    TxLinkCount = 7,
    NonEmptyBlock = 8,
    NonEmptyBlockCount = 9,
}

#[derive(Copy, Clone, PartialEq)]
#[repr(u8)]
enum TxLinkType {
    In = 1,
    Out = 2,
    InOut = 3,
    CreateContract = 4,
}

pub struct AppDB {
    db: DB,
}

pub struct AddrTxLinks {
    iter: DBIterator,
    key: Vec<u8>,
}

impl AddrTxLinks {
    fn new(iter: DBIterator, key: Vec<u8>) -> Self {
        AddrTxLinks { iter, key }
    }
}

pub struct NonEmptyBlocks {
    iter: DBIterator,
    key: Vec<u8>,
}

impl NonEmptyBlocks {
    fn new(iter: DBIterator, key: Vec<u8>) -> Self {
        NonEmptyBlocks { iter, key }
    }
}


#[derive(Debug,Serialize,Deserialize)]
pub struct Contract {
    pub source : String,
    pub abi : String,
    pub name : String,
    pub compiler: String,
    pub optimized: bool,
    pub constructor : Vec<u8>, 
}

fn u64_to_le(v: u64) -> [u8; 8] {
    [
        ((v >> 56) & 0xff) as u8,
        ((v >> 48) & 0xff) as u8,
        ((v >> 40) & 0xff) as u8,
        ((v >> 32) & 0xff) as u8,
        ((v >> 24) & 0xff) as u8,
        ((v >> 16) & 0xff) as u8,
        ((v >> 8) & 0xff) as u8,
        ((v     ) & 0xff) as u8,
    ]
}
fn le_to_u64(v: [u8; 8]) -> u64 {
    u64::from(v[7])
    + (u64::from(v[6]) << 8 )
    + (u64::from(v[5]) << 16)
    + (u64::from(v[4]) << 24)
    + (u64::from(v[3]) << 32)
    + (u64::from(v[2]) << 40)
    + (u64::from(v[1]) << 48)
    + (u64::from(v[0]) << 56)
}
 
impl<'a> Iterator for AddrTxLinks {
    type Item = H256;

    fn next(&mut self) -> Option<H256> {
        if let Some(kv) = self.iter.next() {
            let key = &*(kv.0);
            if key.len() > self.key.len() && key[..self.key.len()] == self.key[..] {
                let tx = H256::from_slice(&key[self.key.len() + 16..]);
                return Some(tx);
            }
        }
        None
    }
}

impl<'a> Iterator for NonEmptyBlocks {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if let Some(kv) = self.iter.next() {
            let key = &*(kv.0);
            if key.len() > self.key.len() && key[..self.key.len()] == self.key[..] {
                let blockno = u64_from_slice(&key[self.key.len()..]);
                return Some(std::u64::MAX -blockno);
            }
        }
        None
    }
}

#[derive(Debug)]
pub enum Error {
    Rocks(rocksdb::Error),
    SerdeCbor(serde_cbor::error::Error),
}
impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
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
    let mut le = [0; 8];
    le[..].copy_from_slice(v);
    le_to_u64(le)
}

impl AppDB {
    pub fn open_default(path: &str) -> Result<AppDB, Error> {
        Ok(DB::open_default(path).map(|x| AppDB { db: x })?)
    }

    fn add_tx_link(&self, tx: &Transaction, link_type: TxLinkType, addr: &Address) -> Result<(), Error> {
        let revblockno = u64_to_le(std::u64::MAX - tx.block_number.unwrap().low_u64());
        let revtxindex = u64_to_le(std::u64::MAX - tx.block_number.unwrap().low_u64());

        let mut key: Vec<u8> = vec![RecordType::TxLink as u8];
        key.extend_from_slice(&addr);
        key.extend_from_slice(&revblockno);
        key.extend_from_slice(&revtxindex);
        key.extend_from_slice(&tx.hash);

        if self.db.get(&key)?.is_none() {
            self.db.put(&key.to_owned(), &[link_type as u8])?;
            self.inc_addr_tx_links(&addr)?;
        }

        Ok(())
    }

    pub fn push_tx(&self, tx: &Transaction, tr: Option<&TransactionReceipt>) -> Result<(), Error> {
        // store tx
        let mut tx_k = vec![RecordType::Tx as u8];
        tx_k.extend_from_slice(&tx.hash);
        self.db
            .put(tx_k.as_slice(), to_vec(tx).unwrap().as_slice())?;

        if let Some(tr) = tr {
            // store receipt
            let mut r_k = vec![RecordType::Receipt as u8];
            r_k.extend_from_slice(&tx.hash);
            self.db.put(r_k.as_slice(), to_vec(tr).unwrap().as_slice())?;
            // TxLink for contract
            if let Some(addr) = tr.contract_address {
                self.add_tx_link(&tx,TxLinkType::CreateContract,&addr)?;
            }
        }

        // TxLink for to/from
        if let Some(to) = tx.to {
            let link_type = if tx.from == to {
                TxLinkType::InOut
            } else {
                TxLinkType::Out
            };
            self.add_tx_link(&tx,link_type,&to)?;
            if link_type == TxLinkType::InOut {
                return Ok(());
            }
        }

        // TxLink for from 
        self.add_tx_link(&tx,TxLinkType::In,&tx.from)?;
        Ok(())
    }

    pub fn get_tx(&self, txhash: &H256) -> Result<Option<Transaction>, Error> {
        let mut tx_k = vec![RecordType::Tx as u8];
        tx_k.extend_from_slice(&txhash);
        Ok(self
            .db
            .get(&tx_k)
            .map(|bytes| bytes.map(|v| from_slice::<Transaction>(&*v).unwrap()))?)
    }

    pub fn get_receipt(&self, txhash: &H256) -> Result<Option<TransactionReceipt>, Error> {
        let mut tx_k = vec![RecordType::Receipt as u8];
        tx_k.extend_from_slice(&txhash);
        Ok(self
            .db
            .get(&tx_k)
            .map(|bytes| bytes.map(|v| from_slice::<TransactionReceipt>(&*v).unwrap()))?)
    }

    pub fn push_block(&self, block: &Block<H256>) -> Result<(), Error> {
        
        // add the block
        let mut b_k = vec![RecordType::Block as u8];
        let block_no = u64_to_le(block.number.unwrap().low_u64());
        b_k.extend_from_slice(&block_no);

        self.db.put(b_k.as_slice(), to_vec(block)?.as_slice())?;

        if block.transactions.len() > 0 {

            // annotate a non-empty-block
            let mut neb_k = vec![RecordType::NonEmptyBlock as u8];
            let block_no_rev = u64_to_le(std::u64::MAX - block.number.unwrap().low_u64());
            neb_k.extend_from_slice(&block_no_rev);
            self.db.put(neb_k.as_slice(), &[])?;
            
            // increment counter of non-empty-blocks
            println!("Incrementing NonEmptyBlockCount");
            self.inc_u64(&vec![RecordType::NonEmptyBlockCount as u8])?;
        }
        Ok(())
    }

    pub fn iter_non_empty_blocks(&self) -> Result<NonEmptyBlocks,Error> {
        let key = vec![RecordType::NonEmptyBlock as u8];
        let iter = self
            .db
            .iterator(IteratorMode::From(&key, Direction::Forward));
        Ok(NonEmptyBlocks::new(iter, key))
    }

    pub fn count_non_empty_blocks(&self) -> Result<u64,Error> {
        let key = vec![RecordType::NonEmptyBlockCount as u8];
        Ok(self.get_u64(&key)?.unwrap_or(0))
    }

    pub fn get_block(&self, blockno: u64) -> Result<Option<Block<H256>>, Error> {
        let mut b_k = vec![RecordType::Block as u8];
        b_k.extend_from_slice(&u64_to_le(blockno));
        Ok(self
            .db
            .get(&b_k)
            .map(|bytes| bytes.map(|v| from_slice::<Block<H256>>(&*v).unwrap()))?)
    }

    pub fn iter_addr_tx_links(&self, addr: &Address) -> AddrTxLinks {
        let mut key: Vec<u8> = vec![RecordType::TxLink as u8];
        key.extend_from_slice(addr);

        let iter = self
            .db
            .iterator(IteratorMode::From(&key, Direction::Forward));

        AddrTxLinks::new(iter, key)
    }

    pub fn count_addr_tx_links(&self, addr: &Address) -> Result<u64,Error> {
        let mut key: Vec<u8> = vec![RecordType::TxLinkCount as u8];
        key.extend_from_slice(addr);
        Ok(self.get_u64(&key)?.unwrap_or(0))
    }

    pub fn inc_addr_tx_links(&self, addr: &Address) -> Result<(),Error> {
        let mut key: Vec<u8> = vec![RecordType::TxLinkCount as u8];
        key.extend_from_slice(addr);
        self.inc_u64(&key)
    }

    pub fn set_contract(&self, addr: &Address, contract: &Contract) -> Result<(),Error> {
        let mut key: Vec<u8> = vec![RecordType::ContractAbi as u8];
        key.extend_from_slice(addr);
        self.db.put(&key, &to_vec(contract)?)?;
        Ok(())
    }

    pub fn get_contract(&self, addr: &Address) -> Result<Option<Contract>,Error> {
        let mut key: Vec<u8> = vec![RecordType::ContractAbi as u8];
        key.extend_from_slice(addr);

        if let Some(bytes) = self.db.get(&key)? {
            Ok(Some(from_slice::<Contract>(&bytes.to_vec())?))
        } else {
            Ok(None)
        }
    }

    pub fn get_last_block(&self) -> Result<Option<u64>, Error> {
        self.get_u64(&[RecordType::NextBlock as u8])
    }

    pub fn set_last_block(&self, n: u64) -> Result<(), Error> {
        self.set_u64(&[RecordType::NextBlock as u8],n)
    }

    fn inc_u64(&self, key : &[u8]) -> Result<(),Error> {
         self.set_u64(&key,1+self.get_u64(&key)?.unwrap_or(0))       
    }

    fn get_u64(&self, key : &[u8]) -> Result<Option<u64>, Error> {
        Ok(self
            .db
            .get(&key)
            .map(|bytes| bytes.map(|v| u64_from_slice(&*v)))?)
    }

    fn set_u64(&self, key: &[u8], n: u64) -> Result<(), Error> {
        self.db.put(&key, &u64_to_le(n))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::iter;
    use types::*;
    use web3::types::{Bytes, Transaction, TransactionReceipt,U128, U256};

    fn init() -> AppDB {
        let mut rng = thread_rng();
        let chars: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(7)
            .collect();

        let mut tmpfile = std::env::temp_dir();
        tmpfile.push(chars);
        AppDB::open_default(tmpfile.as_os_str().to_str().expect("bad OS filename"))
            .expect("unable to create db")
    }

    #[test]
    fn test_add_and_iter() {
        let appdb = init();
        let one_u128 = U128::from_dec_str("1").unwrap();
        let one_u256 = U256::from_dec_str("1").unwrap();
        let a1 = hex_to_addr("0x1eb983836ea12dc37cc4da2effae9c9fbd0b395a").unwrap();
        let a2 = hex_to_addr("0x1eb983836ea12dc37cc4da2effae9c9fbd0b395b").unwrap();
        let a3 = hex_to_addr("0x1eb983836ea12dc37cc4da2effae9c9fbd0b395c").unwrap();
        let h1 = hex_to_h256("0xd69fc1890a1b2742b5c2834d031e34ba55ef3820d463a8d0a674bb5dd9a3b74b")
            .unwrap();
        let h2 = hex_to_h256("0xe69fc1890a1b2742b5c2834d031e34ba55ef3820d463a8d0a674bb5dd9a3b74b")
            .unwrap();

        let tx1 = Transaction {
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

        let tx2 = Transaction {
            hash: h2,
            nonce: one_u256,
            block_hash: None,
            block_number: Some(U256::from_dec_str("10").unwrap()),
            transaction_index: Some(one_u128),
            from: a1,
            to: None,
            value: one_u256,
            gas_price: one_u256,
            gas: one_u256,
            input: Bytes(Vec::new()),
        };

        let tr1 = TransactionReceipt {
            block_hash: None,
            block_number: Some(U256::from_dec_str("10").unwrap()),
            transaction_index: one_u128,
            contract_address: Some(a3),
            gas_used: one_u256,
            cumulative_gas_used: one_u256,
            status: None,
            transaction_hash: h1,
            logs: Vec::new(),
        };

        assert_eq!(Ok(0), appdb.count_addr_tx_links(&a1));
        assert_eq!(Ok(0), appdb.count_addr_tx_links(&a2));
        assert_eq!(Ok(0), appdb.count_addr_tx_links(&a3));

        assert_eq!(Ok(()), appdb.push_tx(&tx1, Some(&tr1)));
        assert_eq!(Ok(1), appdb.count_addr_tx_links(&a1));
        assert_eq!(Ok(1), appdb.count_addr_tx_links(&a2));
        assert_eq!(Ok(1), appdb.count_addr_tx_links(&a3));

        let mut it_a1 = appdb.iter_addr_tx_links(&a1);

        assert_eq!(Some(h1), it_a1.next());
        assert_eq!(None, it_a1.next());

        let mut it_a2 = appdb.iter_addr_tx_links(&a2);
        assert_eq!(Some(h1), it_a2.next());
        assert_eq!(None, it_a2.next());

        let mut it_a3 = appdb.iter_addr_tx_links(&a3);
        assert_eq!(Some(h1), it_a3.next());
        assert_eq!(None, it_a3.next());

        // add again, should be unchanged
        assert_eq!(Ok(()), appdb.push_tx(&tx1, Some(&tr1)));
        assert_eq!(Ok(1), appdb.count_addr_tx_links(&a1));
        assert_eq!(Ok(1), appdb.count_addr_tx_links(&a2));
        assert_eq!(Ok(1), appdb.count_addr_tx_links(&a3));

        // add second transaction
        assert_eq!(Ok(()), appdb.push_tx(&tx2, None));
        assert_eq!(Ok(2), appdb.count_addr_tx_links(&a1));
        assert_eq!(Ok(1), appdb.count_addr_tx_links(&a2));
        assert_eq!(Ok(1), appdb.count_addr_tx_links(&a3));

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

    #[test]
    fn test_inc() {
        let appdb = init();
        let key = vec![1];
        assert_eq!(Ok(None), appdb.get_u64(&key));
        assert_eq!(Ok(()), appdb.inc_u64(&key));
        assert_eq!(Ok(Some(1)), appdb.get_u64(&key));
        assert_eq!(Ok(()), appdb.inc_u64(&key));
        assert_eq!(Ok(Some(2)), appdb.get_u64(&key));
    }

}
