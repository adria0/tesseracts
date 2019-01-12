use db;
use db::AppDB;
use state::*;
use std::collections::HashMap;
use types::into_block;
use web3::futures::Future;
use web3::types::{
    Address, Block, BlockId, BlockNumber, Bytes, Transaction, TransactionId, TransactionReceipt,
    H256, U256,
};

#[derive(Debug)]
pub enum Error {
    Web3(web3::Error),
    Db(db::Error),
}

impl From<web3::Error> for Error {
    fn from(err: web3::Error) -> Self {
        Error::Web3(err)
    }
}
impl From<db::Error> for Error {
    fn from(err: db::Error) -> Self {
        Error::Db(err)
    }
}

pub struct BlockchainReader<'a> {
    wc: &'a Web3Client,
    pub db: &'a AppDB,
}

impl<'a> BlockchainReader<'a> {
    pub fn new(wc: &'a Web3Client, db: &'a AppDB) -> Self {
        BlockchainReader { wc, db }
    }
    pub fn current_block_number(&self) -> Result<u64, Error> {
        Ok(self.wc.web3.eth().block_number().wait()?.low_u64())
    }
    pub fn current_balance(&self, addr: &Address) -> Result<U256, Error> {
        Ok(self.wc.web3.eth().balance(*addr, None).wait()?)
    }
    pub fn current_code(&self, addr: &Address) -> Result<Bytes, Error> {
        Ok(self.wc.web3.eth().code(*addr, None).wait()?)
    }

    pub fn block(&self, blockno: u64) -> Result<Option<Block<H256>>, Error> {
        if let Some(blk) = self.db.get_block(blockno)? {
            Ok(Some(blk))
        } else {
            let blockid = BlockId::Number(BlockNumber::Number(blockno));
            if let Some(blk) = self.wc.web3.eth().block(blockid).wait()? {
                Ok(Some(blk))
            } else {
                Ok(None)
            }
        }
    }
    pub fn block_with_txs(&self, blockno: u64) -> Result<Option<Block<Transaction>>, Error> {
        // assume that if the block exists all transactions will also exist
        if let Some(blk) = self.db.get_block(blockno)? {
            let mut txs = HashMap::new();
            for txhash in &blk.transactions {
                let tx = self.db.get_tx(&txhash)?.unwrap(); // TODO: remove unwrap
                txs.insert(tx.hash, tx);
            }
            Ok(Some(into_block(blk, move |h: H256| {
                txs.remove(&h).unwrap()
            })))
        } else {
            let blockid = BlockId::Number(BlockNumber::Number(blockno));
            if let Some(blk) = self.wc.web3.eth().block_with_txs(blockid).wait()? {
                Ok(Some(blk))
            } else {
                Ok(None)
            }
        }
    }
    pub fn tx(
        &self,
        txhash: H256,
    ) -> Result<Option<(Transaction, Option<TransactionReceipt>)>, Error> {
        let mut tx = self.db.get_tx(&txhash)?;
        if tx.is_none() {
            tx = self
                .wc
                .web3
                .eth()
                .transaction(TransactionId::Hash(txhash))
                .wait()?;
        }
        if let Some(tx) = tx {
            let mut receipt = self.db.get_receipt(&txhash)?;
            if receipt.is_none() {
                receipt = self.wc.web3.eth().transaction_receipt(txhash).wait()?
            }
            Ok(Some((tx, receipt)))
        } else {
            Ok(None)
        }
    }

}
