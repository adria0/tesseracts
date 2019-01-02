use state::GlobalState;
use std::sync::atomic::Ordering;
use std::{thread, time};
use web3::types::{BlockId,BlockNumber};
use web3::futures::Future;

pub fn scan(gs: &GlobalState) {

    let ls = gs.create_local();

    let mut next_block = gs.db
        .get_last_block().unwrap().unwrap();
    
    while !gs.stop_signal.load(Ordering::SeqCst) {
        
        let until_block = ls.web3.eth().block_number().wait().unwrap().low_u64();
        if next_block > until_block {
            thread::sleep(time::Duration::from_secs(5));
            continue;
        }
        
        while next_block <= until_block && !gs.stop_signal.load(Ordering::SeqCst){
            let block = ls.web3.eth()
                .block_with_txs(BlockId::Number(BlockNumber::Number(next_block)))
                .wait().unwrap().unwrap();
        
            let progress = (next_block*1000)/until_block;
            println!("Adding block {}/{} ({}‰)...",next_block,until_block,progress);
            for tx in &block.transactions {
                gs.db.push_tx(&tx).expect("add tx");
            }
            next_block += 1;
            gs.db.set_last_block(next_block).expect("cannot set last block");
        }

    }
    println!("Finished scanning transactions, self-killing.");
    std::process::exit(0);    
}