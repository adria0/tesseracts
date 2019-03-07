use web3::types::{Address, H256, Transaction, TransactionReceipt, U256};
use rustc_hex::ToHex;
use serde_derive::Serialize;
use chrono::prelude::*;
use std::collections::HashMap;
use ethabi;

use super::error::Result;

use super::super::eth::types::InternalTx;
use super::super::state::GlobalState;
use super::super::eth::ContractParser;

const DATETIME_FORMAT : &str = "%Y-%m-%d %H:%M:%S";

lazy_static! {
    static ref GWEI: U256 = U256::from_dec_str("1000000000").unwrap();
    static ref ETHER: U256 = U256::from_dec_str("1000000000000000000").unwrap();
}

#[derive(Serialize)]
pub struct TextWithLink {
    pub text: String,
    pub link: Option<String>,
}

impl TextWithLink {
    fn new_link(text: String, link: String) -> Self {
        TextWithLink {
            text,
            link: Some(link),
        }
    }
    fn new_text(text: String) -> Self {
        TextWithLink {
            text,
            link: None,
        }
    }
    pub fn blank() -> Self {
        TextWithLink {
            text: "".to_string(),
            link: None,
        }
    }
    pub fn html(&self) -> String {
        if let Some(link) = &self.link {
            format!("<a href={}>{}</a>",self.text,link)
        } else {
            self.text.clone()  
        }
    }
}

pub struct HtmlRender<'a> {
    ge : &'a GlobalState,
    parser : ContractParser,
    parsed : HashMap<Address,bool>,

}

impl<'a> HtmlRender<'a> {
    
    pub fn new(ge :&'a GlobalState) -> HtmlRender<'a> {
        HtmlRender {
            ge,
            parser : ContractParser::new(),
            parsed : HashMap::new(),
        }
    }
    
    pub fn addr(&self, addr : &Address) -> TextWithLink {
        TextWithLink::new_link(
            self.ge.named_address.get(addr).unwrap_or(&format!("0x{:x}", addr)).to_string(),
            format!("/0x{:x}", addr)
        )
    }

    pub fn addr_or(&self, optaddr : &Option<Address>, or : &str) -> TextWithLink {
        if let Some(addr) = optaddr {
            self.addr(&addr)
        } else {
             TextWithLink::new_text(or.to_string())
        }
    }

    pub fn bytes(&self, bytes : &[u8], chunk_size: usize) -> Vec<String> {
        bytes.chunks(chunk_size)
            .map(|c| c.to_hex::<String>())
            .collect()
    }

    pub fn txid(&self, txid : &H256) -> TextWithLink {
        TextWithLink::new_link(
            format!("{:x}", txid),
            format!("/0x{:x}", txid),
        )
    }

    pub fn blockno(&self, no : u64) -> TextWithLink {
        TextWithLink::new_link(format!("{}", no), format!("/{}", no))
    }

    pub fn gwei(&self, wei : &U256) -> TextWithLink {
        TextWithLink::new_text(format!("{} GWei ({})", wei / *GWEI, wei))
    }

    pub fn ether(&self, wei : &U256) -> TextWithLink {
        if *wei == U256::zero()  {
            TextWithLink::new_text("0 Ξ".to_string())
        } else {
            let ether  = wei / *ETHER;
            let mut remain = wei % *ETHER;
            while remain > U256::zero() && remain % 10 == U256::zero() {
                remain /= 10; 
            }
            TextWithLink::new_text(format!("{}.{} Ξ", ether, remain))
        }
    }

    pub fn timestamp(&self, sec1970 : &U256) -> TextWithLink {
        let dt = Utc.timestamp(sec1970.low_u64() as i64, 0);
        TextWithLink::new_text(format!("{}",dt.format(DATETIME_FORMAT)))
    }

    pub fn tx(&mut self,tx: &Transaction, rcpt: &Option<TransactionReceipt>) -> Result<serde_json::Value> {
        
        let shortdata = if let Some(to) = tx.to {
            if self.has_contract(&to)? {
                let callinfo = self.parser.tx_funcparams(&to, &tx.input.0,false)?;
                callinfo.func.to_string()
            } else {
                tx.input.0.to_hex::<String>()
                .chars().take(8).collect::<String>()
            }
        } else {
            String::from("")
        };

        let (to_link,to_label) = if let Some(to) = tx.to {             
            (self.addr(&to),"")
        } else if let Some(rcpt) = rcpt {
            (self.addr_newcontract(&rcpt.contract_address.unwrap()),"")
        } else {
            (TextWithLink::blank(),"New contract")
        };

        Ok(json!({
            "type"          : "EXT",
            "blockno"       : self.blockno(tx.block_number.unwrap().low_u64()),
            "tx"            : self.txid(&tx.hash),
            "from"          : self.addr(&tx.from),
            "to_link"       : to_link,
            "to_label"      : to_label,
            "shortdata"     : shortdata,
            "value"         : self.ether(&tx.value)
        }))
    }

    fn addr_newcontract(&self, addr: &Address) -> TextWithLink {
        let mut twl = self.addr(&addr);
        twl.text = format!("🆕 {}",twl.text);
        twl
    }

    fn addr_to(&self, to: &Option<Address>, contract: &Option<Address>) -> TextWithLink {
        if let Some(to) = to {
            self.addr(&to)
        } else {
            self.addr_newcontract(&contract.unwrap())
        }
    }

    pub fn tx_itx(&mut self,tx: &Transaction, itx: &InternalTx) -> Result<serde_json::Value> {
        
        let shortdata = if let Some(to) = itx.to {
            if self.has_contract(&to)? {
                let callinfo = self.parser.tx_funcparams(&to, &itx.input,false)?;
                callinfo.func.to_string()
            } else {
                itx.input.to_hex::<String>()
                .chars().take(8).collect::<String>()
            }
        } else {
            String::from("")
        };

        Ok(json!({
            "type"          : "int",
            "blockno"       : self.blockno(tx.block_number.unwrap().low_u64()),
            "tx"            : self.txid(&tx.hash),
            "from"          : self.addr(&itx.from),
            "to_link"       : self.addr_to(&itx.to,&itx.contract),
            "shortdata"     : shortdata,
            "value"         : self.ether(&itx.value)
        }))
    }

    pub fn tx_abi_call(&mut self, addr: &Address, input: &[u8]) -> Result<Option<Vec<String>>> {
        if self.has_contract(addr)? {
            let callinfo = self.parser.tx_funcparams(addr, input,true)?;

            let mut out = Vec::new();
            out.push(format!("function {}",&callinfo.func));

            if !callinfo.params.is_empty() {
                let max_param_length = callinfo.params.iter().map(|p| p.0.len()).max().unwrap();        

                for (name,value) in callinfo.params {
                    let padding = (name.len()..max_param_length)
                        .map(|_| " ").collect::<String>();
                    let token = self.abi_token(&value);
                    out.push(format!("  [{}{}]  {}",name,padding,token));
                }
            }
            Ok(Some(out))
        } else {
            Ok(None)
        }
    }

    pub fn tx_abi_log(&mut self, addr: &Address, txlog: web3::types::Log) -> Result<Option<Vec<String>>> {
        if self.has_contract(addr)? {
            let (name,log) = self.parser.log_eventparams(addr, txlog)?;

            let mut out = Vec::new();
            out.push(format!("event {}",&name));
                
            if !log.params.is_empty() {
                let max_param_length = log.params.iter().map(|p| p.name.len()).max().unwrap();        
                for param in log.params {
                    let padding = (param.name.len()..max_param_length)
                        .map(|_| " ").collect::<String>();
                    let token = self.abi_token(&param.value);
                    out.push(format!("  [{}{}] {}",param.name,padding,token));
                }
            }
            Ok(Some(out))
        } else {
            Ok(None)
        }
    }

    fn abi_token(&self, token : &ethabi::Token) -> String {
        match token {
            ethabi::Token::Address(v) =>
                format!("0x{:x}",v),

            ethabi::Token::Int(v) | ethabi::Token::Uint(v) => 
                format!("{:} (0x{:x})",v,v),

            ethabi::Token::Bool(v) => 
                format!("{:}",v),
                              
            ethabi::Token::FixedBytes(v) | ethabi::Token::Bytes(v) =>
                format!("0x{}",v.to_hex::<String>()),

            _ =>
                format!("{:?}",token)
        }
    }

    fn has_contract(&mut self, addr: &Address) -> Result<bool> {
        if !self.parsed.contains_key(addr) {
            self.parsed.insert(*addr, true);
            if let Some(contract) = self.ge.db.get_contract(&addr)? {
                self.parser.add(*addr, &contract.abi)?;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(self.parser.contains(addr))
        }
    }
}
