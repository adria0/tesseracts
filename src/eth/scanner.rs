use state::{GlobalState, Web3Client};
use std::sync::atomic::Ordering;
use std::{thread, time};
use web3::futures::Future;
use web3::types::{BlockId, BlockNumber, Transaction};
use eth::geth;

use super::types::*;

use super::error::Result;

fn scan_blocks(gs: &GlobalState, wc: &Web3Client) -> Result<()>{
    let mut next_block = gs.db.get_last_block()?.unwrap();
    
    if let Some(cfg_start_block) = gs.cfg.scan_start_block {
        if cfg_start_block > next_block {
            next_block = cfg_start_block;
        }
    }           
    let until_block = wc.web3.eth().block_number().wait()?.low_u64();
    while next_block <= until_block && !gs.stop_signal.load(Ordering::SeqCst) {
        let block = wc
            .web3
            .eth()
            .block_with_txs(BlockId::Number(BlockNumber::Number(next_block)))
            .wait()?
            .unwrap();

        let progress = (next_block * 1000) / until_block;
        info!(
            "Adding block {}/{} ({}‰)...",
            next_block, until_block, progress
        );

        for tx in &block.transactions {
            // read transaction receipt
            let re = wc.web3.eth().transaction_receipt(tx.hash).wait()?.unwrap();
            
            if gs.cfg.db_store_itx && gs.cfg.web3_itx {
                // read internal transactions
                let dbg : geth::web3::Debug<_> = wc.web3.api();
                let itxs = dbg.internal_txs(&tx).wait()?.parse()?;
                gs.db.add_tx(&tx, &re, Some(&itxs))?;
            } else {
                gs.db.add_tx(&tx, &re, None)?;
            };
        }

        gs.db
            .push_block(&into_block(block, |tx: Transaction| tx.hash))?;

        next_block += 1;
        gs.db.set_last_block(next_block)?;
    }
    Ok(())
}

pub fn scan(gs: &GlobalState) {
    let wc = gs.new_web3client();
    while !gs.stop_signal.load(Ordering::SeqCst) {
        thread::sleep(time::Duration::from_secs(5));
        if let Err(err) = scan_blocks(&gs, &wc) {
            error!("Scan result failed: {:?}", err);
        }
    }
    info!("Finished scanning transactions, self-killing.");
    std::process::exit(0);
}
