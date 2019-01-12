use handlebars::Handlebars;
use web3::types::{BlockId, BlockNumber,H256};
use reader::BlockchainReader;

use super::super::db;
use super::super::contract;

use super::error::Error;
use super::html::*;

pub fn html(db : &db::AppDB, reader: &BlockchainReader, hb: &Handlebars, txid: H256) -> Result<String, Error> {

    if let Some((tx, receipt)) = reader.tx(txid)? {

        let mut logs = Vec::new();
        let mut cumulative_gas_used = String::from("");
        let mut gas_used = String::from("");
        let mut contract_address = TextWithLink::blank();
        let mut status = String::from("");
        
        if let Some(receipt) = receipt {

            cumulative_gas_used = format!("{}", receipt.cumulative_gas_used.low_u64());
            gas_used = format!("{}", receipt.gas_used.low_u64());
            contract_address = receipt
                .contract_address
                .map_or_else( TextWithLink::blank, |c| c.html());
            status = receipt
                .status
                .map_or_else(|| String::from(""), |s| format!("{}", s));


            for (_, log) in receipt.logs.into_iter().enumerate() {
                
                let mut txt = Vec::new();

                if let Some(contract) = db.get_contract(&log.address)? {
                    // TODO: remove clone
                    let callinfo = contract::log_to_string(&contract.abi,log.clone())?;
                    txt.extend_from_slice(&callinfo);
                    txt.push(String::from(""));
                }

                txt.push("data".to_string());
                for ll in log.data.html().text.split(',') {
                    txt.push(format!("  {}",ll));
                }
                
                txt.push("topics".to_string());
                for (t, topic) in log.topics.into_iter().enumerate() {
                    txt.push(format!("  [{}] {:?}",t,topic));
                }

                logs.push(json!({
                    "address" : log.address.html(),
                    "txt"     : txt,
                }));

            }

        }

        // log_to_string
        let mut input: Vec<String> = Vec::new();
        if let Some(to) = tx.to {
            if let Some(contract) = db.get_contract(&to)? {
                let callinfo = contract::call_to_string(&contract.abi,&tx.input.0)?;
                input.extend_from_slice(&callinfo);
                input.push(String::from(""));
            }

            let inputhtml = tx.input.html();
            let inputvec : Vec<String> = inputhtml.text.split(',').map(|x| x.to_string()).collect(); 
            input.extend_from_slice(&inputvec);
        }

        let block = tx.block_number.map_or_else(
            TextWithLink::blank,
            |b| BlockId::Number(BlockNumber::Number(b.low_u64())).html(),
        );

        Ok(hb.render(
            "tx.handlebars",
            &json!({
            "txhash"              : format!("0x{:x}",txid),
            "from"                : tx.from.html(),
            "to"                  : tx.to.html(),
            "value"               : Ether(tx.value).html().text,
            "block"               : block,
            "gas"                 : tx.gas.low_u64(),
            "gas_price"           : GWei(tx.gas_price).html().text,
            "cumulative_gas_used" : cumulative_gas_used,
            "gas_used"            : gas_used,
            "contract_address"    : contract_address,
            "status"              : status,
            "input"               : input,
            "logs"                : logs,
            }),
        )?)
    } else {
        Err(Error::NotFound)
    }
}
