#![cfg_attr(feature = "cargo-clippy", allow(clippy::mem_replace_option_with_none))]

use std::sync::Arc;
use web3::types::{Address,H256};
use rouille::{Request,Response};

use super::super::state::GlobalState;
use super::super::bootstrap::get_resource;

use super::super::eth::{
    BlockchainReader,
    contract::{verify_abi,compile_and_verify,ONLY_ABI}
};

use super::super::eth::types::{hex_to_addr,hex_to_h256};
use super::super::db;

#[derive(Serialize)]
pub enum Id {
    Addr(Address),
    Tx(H256),
    Block(u64),
}

impl Id {
    pub fn from(id: &str) -> Option<Self> {
        if id.len() == 42 /* address */
        {
            hex_to_addr(id).map(Id::Addr).ok()
        } else if id.len() == 66 /* tx */
        {
            hex_to_h256(id).map(Id::Tx).ok()
        } else if let Ok(blockno_u64) = id.parse::<u64>() {
            Some(Id::Block(blockno_u64))
        } else {
            None
        }
    }
}

pub fn error_page(innerhtml: &str) -> String {
    let mut html = String::from("");
    html.push_str("<html><style>body {font-family: Courier;}</style>");
    html.push_str(&innerhtml.replace(" ", "&nbsp;").replace("_", " "));
    html.push_str("</html>");
    html
}

fn get_home(request: &Request, ge: &GlobalState) -> Response {
    let page_no = request.get_param("p").unwrap_or_else(|| "0".to_string()).parse::<u64>();
    if let Ok(page_no) = page_no {
        Response::html(
            match super::home::render(&ge,page_no) {
                Ok(html) => html,
                Err(err) => error_page(format!("Error: {:?}", err).as_str())
            }
        )
    } else {
        Response::html(error_page("invalid parameter"))
    }
}

fn get_object(request: &Request,ge: &GlobalState, id: &str) -> Response {
    
    if id == "neb" {
        let page_no = request.get_param("p").unwrap_or_else(|| "0".to_string()).parse::<u64>().unwrap();
        let html = super::neb::render(&ge,page_no);
        Response::html(match html {
            Ok(html) => html,
            Err(err) => error_page(format!("Error: {:?}", err).as_str())
        })
    } else if let Some(id) = Id::from(&id) {
        let page_no = request.get_param("p").unwrap_or_else(|| "0".to_string()).parse::<u64>().unwrap();
        let html = match id {
            Id::Addr(addr) => super::address::render(&ge,&addr,page_no),
            Id::Tx(txid) => super::tx::render(&ge,txid),
            Id::Block(block) => super::block::render(&ge,block)
        };
        Response::html(match html {
            Ok(html) => html,
            Err(err) => error_page(format!("Error: {:?}", err).as_str())
        })
    } else {
        Response::html(error_page("Not found"))
    }
}

fn post_contract(
    ge: &GlobalState,
    id: &str,
    contract_source: &str,
    contract_compiler: &str,
    contract_optimized: bool,
    contract_name: &str

) -> Response {

    if let Some(Id::Addr(addr)) = Id::from(&id) {
        let reader = BlockchainReader::new(&ge);

        let code = reader.current_code(&addr).expect("failed to read contract code").0;

        let contractentry = db::Contract{
            source : contract_source.to_string(),
            compiler : contract_compiler.to_string(),
            optimized: contract_optimized,
            name : contract_name.to_string(),
            constructor : Vec::new(),
            abi : if ge.cfg.solc_bypass && contract_compiler==ONLY_ABI {
                verify_abi(contract_source).expect("cannot verify abi");
                contract_source.to_string()
            } else {
                compile_and_verify(&ge.cfg,
                    &contract_source,
                    &contract_name,
                    &contract_compiler,
                    contract_optimized,
                    &code
                ).expect("cannot verify contract code")
            }
        };
        ge.db.set_contract(&addr,&contractentry).expect("cannot update db");

        Response::redirect_302(format!("/{}",id))
    } else {
        Response::html(error_page("bad input"))
    }
}

pub fn start_explorer(gs: Arc<GlobalState>) {

    rouille::start_server(gs.cfg.bind.clone(), move |request| router!(request,
        (GET)  (/) => {
            get_home(&request,&gs)
        },
        (GET)  (/s/{name: String}) => {
            if let Some(res) = get_resource(&name) {
                rouille::Response::from_data("", res)
            } else {
                rouille::Response::empty_404()
            }
        },
        (GET)  (/{id: String}) => {
            get_object(&request,&gs,&id)
        },
        (POST) (/{id: String}/contract) => {
            let data = try_or_400!(post_input!(request, {
                contract_source: String,
                contract_compiler: String,
                contract_optimized: bool,
                contract_name: String,
            }));
            post_contract(&gs, &id,
                &data.contract_source, &data.contract_compiler,
                data.contract_optimized, &data.contract_name
            )
        },
        _ => rouille::Response::empty_404()
    ));

}
