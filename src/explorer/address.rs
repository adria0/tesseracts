use handlebars::Handlebars;
use web3::types::Address;
use reader::BlockchainReader;

use super::error::Error;
use super::html::*;

use super::super::state;
use super::super::contract;

pub fn html(
    cfg : &state::Config,
    reader: &BlockchainReader,
    hb: &Handlebars,
    addr: &Address,
) -> Result<String, Error> {

    let balance = reader.current_balance(addr)?;
    let code = reader.current_code(addr)?;
    let mut txs = Vec::new();

    for txhash in reader.db.iter_addr_txs(&addr).take(20) {
        if let Some(txrc) = reader.tx(txhash)? {
            txs.push(tx_short_json(&txrc.0));
        }
    }
 
    if !code.0.is_empty() {

        let rawcodehtml = code.html().text;
        let rawcode = rawcodehtml.split(',').collect::<Vec<&str>>();
        
        if let Some(contract) = reader.db.get_contract(addr)? {
            Ok(hb.render(
                "address.handlebars",
                &json!({
                    "address" : format!("0x{:x}",addr),
                    "balance" : Ether(balance).html().text,
                    "txs" : txs,
                    "hascode" : true,
                    "rawcode" : rawcode,
                    "hascontract" : true,
                    "contract_source" : contract.source,
                    "contract_name" : contract.name,
                    "contract_abi" : contract.abi,
                    "contract_compiler" : contract.compiler,
                    "contract_optimized": contract.optimized
                })
            )?)            
        } else {
            let solcversions =  contract::compilers(&cfg)?;
            Ok(hb.render(
                "address.handlebars",
                &json!({
                    "address" : format!("0x{:x}",addr),
                    "balance" : Ether(balance).html().text,
                    "txs" : txs,
                    "hascode" : true,
                    "rawcode" : rawcode,
                    "hascontract" : false,
                    "solcversions" : solcversions,
                })
            )?)
        }
    } else {    
        Ok(hb.render(
            "address.handlebars",
            &json!({
                "address" : format!("0x{:x}",addr),
                "balance" : Ether(balance).html().text,
                "txs"     : txs,
                "hascode" : false,
            })
        )?)
    }
}
