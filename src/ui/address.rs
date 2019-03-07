use web3::types::Address;

use super::error::*;
use super::html::*;

use super::super::eth::{BlockchainReader,installed_compilers,ONLY_ABI};
use super::super::state::GlobalState;
use super::utils;

pub fn html(
    ge: &GlobalState,
    addr: &Address,
    page_no : u64,
) -> Result<String> {

    let cfg = &ge.cfg;
    let mut hr = HtmlRender::new(&ge); 
    let reader = BlockchainReader::new(&ge);
    let db = &ge.db;
    let hb = &ge.hb;

    let balance = reader.current_balance(addr)?;
    let code = reader.current_code(addr)?;
    let mut txs = Vec::new();

    let count_addr_tx_links = db.count_addr_tx_links(&addr)?;
    let  limit = if count_addr_tx_links > 200 {
        200
    } else {
        count_addr_tx_links
    };
    
    let pg = utils::paginate(limit,20,page_no);
    if pg.from <= pg.to {
        let it = db.iter_addr_tx_links(&addr).skip(pg.from as usize);
        for (txhash,itx_no) in it.take((pg.to-pg.from) as usize) {
            let tx = reader.tx(txhash)?.unwrap();
            if itx_no == 0 {
                txs.push(hr.tx(&tx.0,&tx.1)?);
            } else {
                let itx = db.get_itx(&txhash,itx_no)?.unwrap();
                txs.push(hr.tx_itx(&tx.0,&itx)?)
            }
        }
    }

    if !code.0.is_empty() {

        let mut solcversions =  installed_compilers(&cfg)?;
        if cfg.solc_bypass {
            solcversions.push(ONLY_ABI.to_string());
        }

        let rawcode = hr.bytes(&code.0,50);
        let contract = db.get_contract(addr)?;

        if let Some(contract) = contract {
            
            let can_set_source = contract.compiler == ONLY_ABI;

            Ok(hb.render(
                "address.handlebars",
                &json!({
                    "address" : format!("0x{:x}",addr),
                    "balance" : hr.ether(&balance).text,
                    "txs" : txs,
                    "txs_count" : count_addr_tx_links,
                    "has_next_page": pg.next_page.is_some(),
                    "next_page": pg.next_page.unwrap_or(0),
                    "has_prev_page": pg.prev_page.is_some(),
                    "prev_page": pg.prev_page.unwrap_or(0),                    
                    "hascode" : true,
                    "rawcode" : rawcode,
                    "can_set_source" : can_set_source,
                    "solcversions" : solcversions,
                    "contract_source" : contract.source,
                    "contract_name" : contract.name,
                    "contract_abi" : contract.abi,
                    "contract_compiler" : contract.compiler,
                    "contract_optimized": contract.optimized
                })
            )?)
        
        } else {

            Ok(hb.render(
                "address.handlebars",
                &json!({
                    "address" : format!("0x{:x}",addr),
                    "balance" : hr.ether(&balance).text,
                    "txs" : txs,
                    "txs_count" : count_addr_tx_links,
                    "has_next_page": pg.next_page.is_some(),
                    "next_page": pg.next_page.unwrap_or(0),
                    "has_prev_page": pg.prev_page.is_some(),
                    "prev_page": pg.prev_page.unwrap_or(0),                    
                    "hascode" : true,
                    "rawcode" : rawcode,
                    "can_set_source" : true,
                    "solcversions" : solcversions,
                })
            )?)

        }

    } else {    

        Ok(hb.render(
            "address.handlebars",
            &json!({
                "address" : format!("0x{:x}",addr),
                "balance" : hr.ether(&balance).text,
                "txs"     : txs,
                "txs_count" : count_addr_tx_links,
                "has_next_page": pg.next_page.is_some(),
                "next_page": pg.next_page.unwrap_or(0),
                "has_prev_page": pg.prev_page.is_some(),
                "prev_page": pg.prev_page.unwrap_or(0),                    
                "hascode" : false,
            })
        )?)
    }
}