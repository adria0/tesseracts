use rocksdb::{Direction, IteratorMode, DB};
use serde_cbor::{from_slice, to_vec};
use web3::types::{Address, Block, Transaction, TransactionReceipt, H256};
use rustc_hex::ToHex;

use super::error::*;
use super::types::*;
use super::utils::*;
use super::iterators::*;

use super::super::types::InternalTx;

pub struct Options {
    pub store_itx : bool,
    pub store_tx : bool,
    pub store_addr : bool,
    pub store_neb : bool,
}

pub struct AppDB {
    db: DB,
    opt: Options,
}

/*

  Internal structure of the database
  ----------------------------------
  Tx       <txhash>                                     cbor-encoded-transactionwithreceipt
  IntTx    <txhash> <count>                             cbor-encoded-{from,to,value,data}
  Receipt  <txhash>                                     cbor-encoded-receipt
  AddrLink <addr> <blockno> <txindex> <txhash> <inttx*> TxNewContract(contract_addr)
                                                        TxTo(to_addr)
                                                        TxFromTo()
                                                        TxFrom(from_addr)
  AddrLinkCount <addr>                                  u64
  Block    <blockno>                                    cbor-encoded-block
  ContractAbi <addr>                                    cbor-encoded abi and compile params
  NonEmptyBlock <blockno>                               none
  NonEmptyBlockCount                                    u64
  NextBlock                                             u64

  if *inttx == 0 means a main transaction
  else           means an internal transaction num n

  We need the following proprties:
  
  1) Can be updated in paralel 
  2) Data cannot be manipulated

  inttx is the internal transaction count 

*/

impl AppDB {

    pub fn open_default(path: &str, opt: Options) -> Result<AppDB> {
        Ok(DB::open_default(path).map(|x| AppDB{ opt, db: x })?)
    }

    pub fn add_addrtx_link(&self, addr: &Address, tx: &Transaction, inttxno : u64) -> Result<()> {        
        let revblockno = u64_to_le(std::u64::MAX - tx.block_number.unwrap().low_u64());
        let revtxindex = u64_to_le(std::u64::MAX - tx.block_number.unwrap().low_u64());
        let revinttx = u64_to_le(std::u64::MAX - inttxno);

        let mut key: Vec<u8> = vec![RecordType::TxLink as u8];
        key.extend_from_slice(&addr);
        key.extend_from_slice(&revblockno);
        key.extend_from_slice(&revtxindex);
        key.extend_from_slice(&tx.hash);
        key.extend_from_slice(&revinttx);

        let zero : Vec<u8> = vec![];
        self.db.put(&key.to_owned(), &zero)?;
 
        self.inc_addr_tx_links(&addr)?;

        Ok(())
    }

    fn add_addrtx_links(&self, tx: &Transaction, from: Address, to:Option<Address>, contract:Option<Address>, int_tx_no : u64 ) 
    -> Result<()> {
        if let Some(contract) = contract {
            self.add_addrtx_link(&from,&tx,int_tx_no)?;
            self.add_addrtx_link(&contract,&tx,int_tx_no)?;
        } else {
            let to = to.unwrap();
            if from == to {
                self.add_addrtx_link(&from,&tx,int_tx_no)?;
            } else {
                self.add_addrtx_link(&from,&tx,int_tx_no)?;
                self.add_addrtx_link(&to,&tx,int_tx_no)?;
            }
        }
        Ok(())
    }

    fn add_itx(&self, tx: &Transaction, itx: &InternalTx, itx_no: u64) -> Result<()> {

        // store inttx
        let mut itx_k = vec![RecordType::IntTx as u8];
        let rev_itx_no = u64_to_le(std::u64::MAX - itx_no);
        itx_k.extend_from_slice(&tx.hash);
        itx_k.extend_from_slice(&rev_itx_no);
        self.db
            .put(itx_k.as_slice(), to_vec(itx).unwrap().as_slice())?;

        // store links, TODO
        self.add_addrtx_links(&tx,itx.from, itx.to, itx.contract,itx_no)?;

        Ok(())
    }

    pub fn add_tx(&self, tx: &Transaction, tr: &TransactionReceipt, itxs : Option<&[InternalTx]>) -> Result<()> {
        // check pecondition: tx.to == Some || tr.contract_address == Some
        match (tx.to, tr.contract_address) {
            (Some(_),None) => {},
            (None,Some(_)) => {},
            (_,_) => return Err(Error::Precondition("push_tx"))
        }

        if self.opt.store_tx {

            // store tx
            let mut tx_k = vec![RecordType::Tx as u8];
            tx_k.extend_from_slice(&tx.hash);
            self.db
                .put(tx_k.as_slice(), to_vec(tx).unwrap().as_slice())?;

            // store receipt
            let mut r_k = vec![RecordType::Receipt as u8];
            r_k.extend_from_slice(&tx.hash);
            self.db.put(r_k.as_slice(), to_vec(tr).unwrap().as_slice())?;

            if self.opt.store_itx {
                if let Some(itxs) = itxs {
                    // InternalTxs
                    for (i, itx) in itxs.into_iter().enumerate() {
                        self.add_itx(&tx,itx,i as u64 + 1)?;
                    }
                }
            }
        }

        if self.opt.store_addr {
            // TxLinks
            self.add_addrtx_links(&tx,tx.from,tx.to,tr.contract_address,0)?;
        }

        Ok(())
    }

    pub fn get_tx(&self, txhash: &H256) -> Result<Option<Transaction>> {
        let mut tx_k = vec![RecordType::Tx as u8];
        tx_k.extend_from_slice(&txhash);
        Ok(self
            .db
            .get(&tx_k)
            .map(|bytes| bytes.map(|v| from_slice::<Transaction>(&*v).unwrap()))?)
    }

    pub fn get_itx(&self, txhash: &H256, itx_no: u64) ->  Result<Option<InternalTx>> {
        let mut itx_k = vec![RecordType::IntTx as u8];
        let rev_itx_no = u64_to_le(std::u64::MAX - itx_no);
        itx_k.extend_from_slice(&txhash);
        itx_k.extend_from_slice(&rev_itx_no);
        Ok(self
            .db
            .get(&itx_k)
            .map(|bytes| bytes.map(|v| from_slice::<InternalTx>(&*v).unwrap()))?)
    }

    pub fn get_receipt(&self, txhash: &H256) -> Result<Option<TransactionReceipt>> {
        let mut tx_k = vec![RecordType::Receipt as u8];
        tx_k.extend_from_slice(&txhash);
        Ok(self
            .db
            .get(&tx_k)
            .map(|bytes| bytes.map(|v| from_slice::<TransactionReceipt>(&*v).unwrap()))?)
    }

    pub fn push_block(&self, block: &Block<H256>) -> Result<()> {
        
        if self.opt.store_tx {
            // add the block
            let mut b_k = vec![RecordType::Block as u8];
            let block_no = u64_to_le(block.number.unwrap().low_u64());
            b_k.extend_from_slice(&block_no);

            self.db.put(b_k.as_slice(), to_vec(block)?.as_slice())?;
        }

        if self.opt.store_neb && !block.transactions.is_empty() {
            // annotate a non-empty-block
            let mut neb_k = vec![RecordType::NonEmptyBlock as u8];
            let block_no_rev = u64_to_le(std::u64::MAX - block.number.unwrap().low_u64());
            neb_k.extend_from_slice(&block_no_rev);
            self.db.put(neb_k.as_slice(), &[])?;
            
            // increment counter of non-empty-blocks
            self.inc_u64(&[RecordType::NonEmptyBlockCount as u8])?;
        }
        Ok(())
    }

    pub fn iter_itxs(&self, txhash: &H256) -> InternalTxs {
        let mut key = vec![RecordType::IntTx as u8];
        key.extend_from_slice(&txhash);
        let iter = self
            .db
            .iterator(IteratorMode::From(&key, Direction::Forward));
        InternalTxs::new(iter, key)
    }

    pub fn _count_itxs(&self, txhash: &H256) -> u64 {        
        match self.iter_itxs(txhash).next() {
            Some((n,_)) => n,
            None    => 0
        }
    }

    pub fn iter_non_empty_blocks(&self) -> Result<NonEmptyBlocks> {
        let key = vec![RecordType::NonEmptyBlock as u8];
        let iter = self
            .db
            .iterator(IteratorMode::From(&key, Direction::Forward));
        Ok(NonEmptyBlocks::new(iter, key))
    }

    pub fn count_non_empty_blocks(&self) -> Result<u64> {
        let key = vec![RecordType::NonEmptyBlockCount as u8];
        Ok(self.get_u64(&key)?.unwrap_or(0))
    }

    pub fn get_block(&self, blockno: u64) -> Result<Option<Block<H256>>> {
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

    pub fn count_addr_tx_links(&self, addr: &Address) -> Result<u64> {
        let mut key: Vec<u8> = vec![RecordType::TxLinkCount as u8];
        key.extend_from_slice(addr);
        Ok(self.get_u64(&key)?.unwrap_or(0))
    }

    pub fn inc_addr_tx_links(&self, addr: &Address) -> Result<u64> {
        let mut key: Vec<u8> = vec![RecordType::TxLinkCount as u8];
        key.extend_from_slice(addr);
        self.inc_u64(&key)
    }

    pub fn set_contract(&self, addr: &Address, contract: &Contract) -> Result<()> {
        let mut key: Vec<u8> = vec![RecordType::ContractAbi as u8];
        key.extend_from_slice(addr);
        self.db.put(&key, &to_vec(contract)?)?;
        Ok(())
    }

    pub fn get_contract(&self, addr: &Address) -> Result<Option<Contract>> {
        let mut key: Vec<u8> = vec![RecordType::ContractAbi as u8];
        key.extend_from_slice(addr);

        if let Some(bytes) = self.db.get(&key)? {
            Ok(Some(from_slice::<Contract>(&bytes.to_vec())?))
        } else {
            Ok(None)
        }
    }

    pub fn get_last_block(&self) -> Result<Option<u64>> {
        self.get_u64(&[RecordType::NextBlock as u8])
    }

    pub fn set_last_block(&self, n: u64) -> Result<()> {
        self.set_u64(&[RecordType::NextBlock as u8],n)
    }

    fn inc_u64(&self, key : &[u8]) -> Result<u64> {
         let value = 1+self.get_u64(&key)?.unwrap_or(0);
         self.set_u64(&key,value)?;
         Ok(value)       
    }

    fn get_u64(&self, key : &[u8]) -> Result<Option<u64>> {
        Ok(self
            .db
            .get(&key)
            .map(|bytes| bytes.map(|v| u64_from_slice(&*v)))?)
    }

    fn set_u64(&self, key: &[u8], n: u64) -> Result<()> {
        self.db.put(&key, &u64_to_le(n))?;
        Ok(())
    }

    pub fn _dump(&self) -> Result<()> {
        let key = vec![RecordType::TxLink as u8];
        let iter = self
            .db
            .iterator(IteratorMode::From(&key, Direction::Forward));
        
        for (key,_value) in iter {
            println!("**Key.iter {}",key.to_hex::<String>());
        }

        Ok(())
    }
}

