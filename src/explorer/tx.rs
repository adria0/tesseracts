use web3::types::H256;

use super::error::*;
use super::contract;
use super::html::*;

use super::super::bcio::BlockchainReader;
use super::super::state::GlobalState;

pub fn html(
    ge: &GlobalState,
    txid: H256) -> Result<String> {

    let wc = ge.new_web3client();
    let hr = HtmlRender::new(&ge); 
    let reader = BlockchainReader::new(&wc,&ge.db);
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
                for ll in hr.bytes(&log.data.0).split(',') {
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
                let callinfo = contract::call_to_string(&contract.abi,&tx.input.0)?;
                input.extend_from_slice(&callinfo);
                input.push(String::from(""));
            }

            let inputhtml = hr.bytes(&tx.input.0);
            let inputvec : Vec<String> = inputhtml.split(',').map(|x| x.to_string()).collect(); 
            input.extend_from_slice(&inputvec);
        }

        // internal transactions
        let itxs : Vec<_>= reader.db.iter_itxs(&txid)
            .map(|(_,itx)| hr.itx(&itx))
            .collect();

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