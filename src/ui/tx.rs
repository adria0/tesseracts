use web3::types::H256;

use super::error::*;
use super::html::*;

use super::super::eth::{BlockchainReader,ContractParser};
use super::super::state::GlobalState;

pub fn html(
    ge: &GlobalState,
    txid: H256) -> Result<String> {

    let hr = HtmlRender::new(&ge); 
    let reader = BlockchainReader::new(&ge);
    let db = &ge.db;
    let hb = &ge.hb;

    if let Some((tx, receipt)) = reader.tx(txid)? {

        let mut logs = Vec::new();
        let mut cumulative_gas_used = String::from("");
        let mut gas_used = String::from("");
        let mut contract_address = TextWithLink::blank();
        let mut status = String::from("");
        
        if let Some(receipt) = receipt {

            cumulative_gas_used = format!("{}", receipt.cumulative_gas_used.low_u64());
            gas_used = format!("{}", receipt.gas_used.unwrap().low_u64());
            
            if let Some(contract) = receipt.contract_address {
                contract_address = hr.addr(&contract);
            }

            status = receipt.status.map_or(
                "".to_string(),
                |x| if x.as_u64() == 1 { "Success".to_string() } else { "Failed".to_string() }
            );  

            for (_, log) in receipt.logs.into_iter().enumerate() {
                
                let mut txt = Vec::new();

                if let Some(contract) = db.get_contract(&log.address)? {
                    let parser = ContractParser::from(&contract.abi)?;
                    let callinfo = hr.tx_log(&parser,log.clone())?;
                    
                    txt.extend_from_slice(&callinfo);
                    txt.push(String::from(""));
                }

                txt.push("data".to_string());
                for ll in hr.bytes(&log.data.0,50) {
                    txt.push(format!("  {}",ll));
                }
                
                txt.push("topics".to_string());
                for (t, topic) in log.topics.into_iter().enumerate() {
                    txt.push(format!("  [{}] {:?}",t,topic));
                }

                logs.push(json!({
                    "address" : hr.addr(&log.address),
                    "txt"     : txt,
                }));
            }
        }

        // log_to_string
        let mut input: Vec<String> = Vec::new();
        if let Some(to) = tx.to {
            if let Some(contract) = db.get_contract(&to)? {
                let parser = ContractParser::from(&contract.abi)?;
                let callinfo = hr.tx_call(&parser,&tx.input.0)?;
                input.extend_from_slice(&callinfo);
                input.push(String::from(""));
            }

            let inputvec = hr.bytes(&tx.input.0,50);
            input.extend_from_slice(&inputvec);
        }

        // internal transactions
        let itxs = reader.itx(&tx)?
            .into_iter()
            .map(|itx| hr.tx_itx(&tx,&itx))
            .collect::<Vec<_>>();

        // render page
        Ok(hb.render(
            "tx.handlebars",
            &json!({
            "txhash"              : format!("0x{:x}",txid),
            "from"                : hr.addr(&tx.from),
            "tonewcontract"       : tx.to.is_none(),
            "to"                  : hr.addr_or(&tx.to,"New contract"),
            "value"               : hr.ether(&tx.value).text,
            "block"               : hr.blockno(tx.block_number.unwrap().low_u64()),
            "gas"                 : tx.gas.low_u64(),
            "gas_price"           : hr.gwei(&tx.gas_price).text,
            "cumulative_gas_used" : cumulative_gas_used,
            "gas_used"            : gas_used,
            "contract_address"    : contract_address,
            "status"              : status,
            "input"               : input,
            "logs"                : logs,
            "itxs"                : itxs,
            }),
        )?)
    } else {
        Err(Error::NotFound)
    }
}