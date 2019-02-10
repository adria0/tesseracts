use web3::types::Address;

use super::error::*;
use super::html::*;

use super::super::bcio::BlockchainReader;
use super::super::state::GlobalState;
use super::contract;
use super::paginate;

pub fn html(
    ge: &GlobalState,
    addr: &Address,
    page_no : u64,
) -> Result<String> {

    let wc = ge.new_web3client();
    let cfg = &ge.cfg;
    let hr = HtmlRender::new(&ge); 
    let reader = BlockchainReader::new(&wc,&ge.db);
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
    let pg = paginate::paginate(limit,20,page_no);
    if pg.from <= pg.to {
        let it = reader.db.iter_addr_tx_links(&addr).skip(pg.from as usize);
        for (txhash,_,_) in it.take((pg.to-pg.from) as usize) {
            if let Some(txrc) = reader.tx(txhash)? {
                txs.push(hr.tx(&txrc.0));
            }
        }
    }
    if !code.0.is_empty() {

        let rawcodehtml = hr.bytes(&code.0);
        let rawcode = rawcodehtml.split(',').collect::<Vec<&str>>();
        
        if let Some(contract) = reader.db.get_contract(addr)? {
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
                    "balance" : hr.ether(&balance).text,
                    "txs" : txs,
                    "txs_count" : count_addr_tx_links,
                    "has_next_page": pg.next_page.is_some(),
                    "next_page": pg.next_page.unwrap_or(0),
                    "has_prev_page": pg.prev_page.is_some(),
                    "prev_page": pg.prev_page.unwrap_or(0),                    
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