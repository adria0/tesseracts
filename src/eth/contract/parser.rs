use std::collections::HashMap;

use keccak_hash;
use ethabi;
use ethabi::param_type::{Writer, ParamType};
use web3::types::Address;

use super::error::{Error,Result};

static FALLBACK : &str = "()";

pub struct ContractParser {
    pub abis : HashMap<Address,ethabi::Contract>,
}

pub struct CallInfo<'a> {
    pub func : &'a str,
    pub params : Vec<(&'a String,ethabi::Token)>,
}

impl ContractParser {
    
    /// create a new contract parser
    pub fn new() -> Self {
        ContractParser { abis : HashMap::new() }
    }
    
    /// add a new contract and its abi
    pub fn add(&mut self, addr: Address, abistr : &str) -> Result<()> {
        if !self.abis.contains_key(&addr) {
            let abi = ethabi::Contract::load(abistr.as_bytes())?;
            self.abis.insert(addr, abi);
        }
        Ok(())
    }

    /// return true if the contract has been already added
    pub fn contains(&self, addr: &Address)-> bool {
        self.abis.contains_key(addr)
    }

    /// parse a function call
    pub fn tx_funcparams(&self, addr: &Address, input: &[u8], parse_params: bool) -> Result<CallInfo> {
        if let Some(abi) = self.abis.get(addr) {
            for func in abi.functions() {
                let paramtypes : &Vec<ParamType> = &func.inputs.iter().map(|p| p.kind.clone()).collect();
                let sig = short_signature(&func.name,&paramtypes);

                if input.len() >= 4 && input[0..4] == sig[0..4] {

                    let params = if parse_params {
                        func.inputs.iter()
                            .map(|input| &input.name)
                            .zip(ethabi::decode(&paramtypes, &input[4..])?)
                            .collect::<Vec<_>>()
                    } else {
                        Vec::new()
                    };

                    return Ok(CallInfo{
                        func : &func.name,
                        params: params
                    });
                }
            } 
            if abi.fallback {
                return Ok(CallInfo{
                    func : FALLBACK,
                    params : Vec::new(),
                });
            } else {
                Err(Error::FunctionNotFound)
            }
        } else {
            Err(Error::ContractNotFound)
        }
    }

    /// find the event that matches with the current log
    fn log_findevent(&self, txlog: &web3::types::Log) -> Result<&ethabi::Event> {
        if let Some(abi) = self.abis.get(&txlog.address) {
            let event = abi.events().find(|e| e.signature()==txlog.topics[0]);

            if let Some(event) = event {
                Ok(&event)
            } else {
                Err(Error::EventNotFound)
            }
        } else {
            Err(Error::ContractNotFound)
        }
    }

    /// get the event name and parameters of a log 
    pub fn log_eventparams(&self, txlog: web3::types::Log) -> Result<(&str,ethabi::Log)> {
        let event = self.log_findevent(&txlog)?;
        let rawlog = ethabi::RawLog{topics: txlog.topics,data: txlog.data.0};
        let log = event.parse_log(rawlog)?;
        Ok((&event.name,log))
    }

}

/// taken from libraries, return a method 4-byte signature
fn short_signature(name: &str, params: &[ParamType]) -> [u8; 4] {

    fn fill_signature(name: &str, params: &[ParamType], result: &mut [u8]) {
        let types = params.iter()
            .map(Writer::write)
            .collect::<Vec<String>>()
            .join(",");

        let data: Vec<u8> = From::from(format!("{}({})", name, types).as_str());
        keccak_hash::keccak_256(&data,result);
    }

	let mut result = [0u8; 4];
	fill_signature(name, params, &mut result);
	result
}
