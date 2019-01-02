use state::{GlobalState,LocalState};
use std::sync::atomic::Ordering;
use std::{thread, time};
use web3::types::{BlockId,BlockNumber};
use web3::futures::Future;
use db;

#[derive(Debug)]
pub enum Error {
    Uninitialized,
    Web3(web3::Error),
    DB(db::Error)
}

impl From<web3::Error> for Error {
    fn from(err: web3::Error) -> Self {
        Error::Web3(err)
    }
}
impl From<db::Error> for Error {
    fn from(err: db::Error) -> Self {
        Error::DB(err)
    }
}

fn scan_blocks(gs: &GlobalState, ls:&LocalState) -> Result<(),Error>{
    if let Some(mut next_block) = gs.db.get_last_block()? {
        let until_block = ls.web3.eth().block_number().wait()?.low_u64();      
        
        while next_block <= until_block && !gs.stop_signal.load(Ordering::SeqCst){    
            
            let block = ls.web3.eth()
                .block_with_txs(BlockId::Number(BlockNumber::Number(next_block)))
                .wait()?.unwrap();

            let progress = (next_block*1000)/until_block;
            println!("Adding block {}/{} ({}‰)...",next_block,until_block,progress);
            for tx in &block.transactions {
                gs.db.push_tx(&tx).expect("add tx");
            }
            next_block += 1;
            gs.db.set_last_block(next_block)?;
        }
        Ok(())
    } else {
        Err(Error::Uninitialized)
    }
}

pub fn scan(gs: &GlobalState) {
    let ls = gs.create_local();
    while !gs.stop_signal.load(Ordering::SeqCst) {
        thread::sleep(time::Duration::from_secs(5));
        let ret = scan_blocks(&gs,&ls);
        println!("Scan result: {:?}",ret);
    }
    println!("Finished scanning transactions, self-killing.");
    std::process::exit(0);    
}