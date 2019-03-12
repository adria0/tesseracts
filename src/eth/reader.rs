use state::*;
use std::collections::HashMap;
use web3::futures::Future;
use web3::types::{
    Address, Block, BlockId, BlockNumber, Bytes, Transaction, TransactionId, TransactionReceipt,
    H256, U256,
};

use super::error::Result;
use super::types::*;

use super::super::eth::geth;
use super::super::state::GlobalState;

pub struct BlockchainReader<'a> {
    wc: Web3Client,
    pub ge: &'a GlobalState,
}

/// Blockchain reader provides an unified way to access to the blockchain data 
impl<'a> BlockchainReader<'a> {
    pub fn new(ge: &'a GlobalState) -> Self {
        let wc = ge.new_web3client();
        BlockchainReader { wc, ge }
    }

    /// retrieve the current block
    pub fn current_block_number(&self) -> Result<u64>{
        Ok(self.wc.web3.eth().block_number().wait()?.low_u64())
    }

    /// retrieve the current balance for an address
    pub fn current_balance(&self, addr: &Address) -> Result<U256>{
        Ok(self.wc.web3.eth().balance(*addr, None).wait()?)
    }

    /// retrieve the current code for an address
    pub fn current_code(&self, addr: &Address) -> Result<Bytes>{
        Ok(self.wc.web3.eth().code(*addr, None).wait()?)
    }

    /// retrieve a block
    pub fn block(&self, blockno: u64) -> Result<Option<Block<H256>>>{
        if let Some(blk) = self.ge.db.get_block(blockno)? {
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
    /// retrieve a block with its transactions
    pub fn block_with_txs(&self, blockno: u64) -> Result<Option<Block<Transaction>>>{
        // assume that if the block exists all transactions will also exist
        if let Some(blk) = self.ge.db.get_block(blockno)? {
            let mut txs = HashMap::new();
            for txhash in &blk.transactions {
                let tx = self.ge.db.get_tx(&txhash)?.unwrap(); // TODO: remove unwrap
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

    /// retrieve a transaction
    pub fn tx(
        &self,
        txhash: H256,
    ) -> Result<Option<(Transaction, Option<TransactionReceipt>)>>{
        let mut tx = self.ge.db.get_tx(&txhash)?;
        if tx.is_none() {
            tx = self
                .wc
                .web3
                .eth()
                .transaction(TransactionId::Hash(txhash))
                .wait()?;
        }
        if let Some(tx) = tx {
            let mut receipt = self.ge.db.get_receipt(&txhash)?;
            if receipt.is_none() {
                receipt = self.wc.web3.eth().transaction_receipt(txhash).wait()?
            }
            Ok(Some((tx, receipt)))
        } else {
            Ok(None)
        }
    }

    /// retrieve an internal transaction
    pub fn itx(
        &self,
        tx: &Transaction
    ) -> Result<Vec<InternalTx>> {
        let mut itxs : Vec<InternalTx> = self.ge.db.iter_itxs(&tx.hash).map(|(_,t)| t).collect();
        if itxs.is_empty() && self.ge.cfg.web3_itx {
            let dbg : geth::web3::Debug<_> = self.wc.web3.api();
            itxs = dbg.internal_txs(&tx).wait()?.parse()?;
        }
        Ok(itxs)
    }

}
