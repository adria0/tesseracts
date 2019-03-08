use state::{GlobalState, Web3Client};
use std::sync::atomic::Ordering;
use std::{thread, time};
use web3::futures::Future;
use web3::types::{BlockId, BlockNumber, Transaction};
use eth::geth;

use super::super::eth::types::*;
use super::error::Result;

/// scan the blockchain 
fn scrap_blocks(gs: &GlobalState, wc: &Web3Client) -> Result<()>{

    // get next block to scan
    let mut next_block = gs.db.get_next_block_to_scan()?.unwrap();
    
    if let Some(cfg_start_block) = gs.cfg.scan_start_block {
        if cfg_start_block > next_block {
            next_block = cfg_start_block;
        }
    }

    // loop until last block number or stop_signal
    let until_block = wc.web3.eth().block_number().wait()?.low_u64();
    while next_block <= until_block && !gs.stop_signal.load(Ordering::SeqCst) {

        let progress = (next_block * 1000) / until_block;
        info!(
            "Adding block {}/{} ({}‰)...",
            next_block, until_block, progress
        );

        // read block and with its transactions
        let block = wc
            .web3
            .eth()
            .block_with_txs(BlockId::Number(BlockNumber::Number(next_block)))
            .wait()?
            .unwrap();

        // process each transaction
        for tx in &block.transactions {

            // read transaction receipt
            let re = wc.web3.eth().transaction_receipt(tx.hash).wait()?.unwrap();
            
            // read internal transactions
            if gs.cfg.db_store_itx && gs.cfg.web3_itx {
                let dbg : geth::web3::Debug<_> = wc.web3.api();
                let itxs = dbg.internal_txs(&tx).wait()?.parse()?;
                gs.db.add_tx(&tx, &re, Some(&itxs))?;
            } else {
                gs.db.add_tx(&tx, &re, None)?;
            };
        }

        // write to the db the receieved data
        gs.db
            .add_block(&into_block(block, |tx: Transaction| tx.hash))?;

        // process next block
        next_block += 1;
        gs.db.set_next_block_to_scan(next_block)?;
    }
    Ok(())
}

/// scan the blockchain until the stop_signal is recieved
pub fn start_scrapper(gs: &GlobalState) {
    let wc = gs.new_web3client();

    while !gs.stop_signal.load(Ordering::SeqCst) {
        thread::sleep(time::Duration::from_secs(5));
        if let Err(err) = scrap_blocks(&gs, &wc) {
            error!("Scan result failed: {:?}", err);
        }
    }

    info!("Finished scanning transactions, self-killing.");
    std::process::exit(0);
}
