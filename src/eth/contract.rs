use std::process::Command;
use std::collections::HashMap;
use std::io::prelude::*;
use std::fs;
use std::fs::File;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use rustc_hex::{FromHex,ToHex};

use keccak_hash;
use ethabi;
use ethabi::param_type::{Writer, ParamType};
use web3::types::Address;

use bootstrap::Config;
use super::error::{Error,Result};

pub static ONLY_ABI : &str = "abi-only";
static FALLBACK : &str = "()";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SolcContract {
    pub abi : String,
    #[serde(rename = "bin-runtime")]
    pub binruntime : String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SolcJson {
    contracts : HashMap<String,SolcContract>,
    version   : String,
}

pub fn installed_compilers(cfg : &Config) -> Result<Vec<String>> {
    if let Some(path) = &cfg.solc_path {
        Ok(fs::read_dir(path)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().unwrap().is_file())
            .map(|entry| entry.file_name().into_string().unwrap())
            .collect()
        )
    } else {
        Ok(Vec::new())
    }
}

pub fn verify_abi(source: &str) -> Result<()>{
    ethabi::Contract::load(source.as_bytes())?;
    Ok(())
}

pub fn compile_and_verify(

    cfg : &Config,
    source: &str,
    contractname: &str,
    compiler: &str,
    optimized: bool,
    code: &[u8])

-> Result<String> {

    if installed_compilers(&cfg)?.into_iter().find(|c| c==compiler).is_none() {
        return Err(Error::CompilerNotFound);
    }

    let mut rng = thread_rng();
    let chars: String = std::iter::repeat(())
        .map(|()| rng.sample(Alphanumeric))
        .take(12)
        .collect();

    let mut tmp_dir_path = std::env::temp_dir();
    tmp_dir_path.push(chars);

    std::fs::create_dir(&tmp_dir_path)?;
    let tmp_dir = tmp_dir_path.into_os_string().into_string().unwrap();
    let input = format!("{}/contract.sol",tmp_dir);
    let output = format!("{}/combined.json",tmp_dir);

    File::create(&input)?.write_all(source.as_bytes())?;

    let args : Vec<&str> = if optimized {
        vec![&input,"-o",&tmp_dir,"--combined-json","abi,bin-runtime","--optimize","--optimize-runs","200"]
    } else {
        vec![&input,"-o",&tmp_dir,"--combined-json","abi,bin-runtime","--optimize-runs","200"]
    };

    let cmdoutput = Command::new(
        format!("{}/{}",&cfg.solc_path.clone().unwrap(),compiler)
    ).args(args).output()?;
    
    println!("stdout: {}", String::from_utf8_lossy(&cmdoutput.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&cmdoutput.stderr));

    let mut contents = String::new();
    File::open(&output)?.read_to_string(&mut contents)?;

    let deserialized: SolcJson = serde_json::from_str(&contents)?;
    let key = format!("{}:{}",&input,contractname);

    if let Some(contract) = deserialized.contracts.get(&key) {
        code_equals(&contract, &code)?;
        Ok(contract.abi.clone()) 
    } else {
        Err(Error::ContractNotFound)
    }
}


fn code_equals(contract : &SolcContract, code: &[u8]) -> Result<()> {
    let binruntime : Vec<u8> = contract.binruntime.from_hex()?;

    // compiled code may have multiple swam hashes in the following
    // way a1 65 62 7a 7a 72 30 58 20 + hash(32bytes) + 0029

    let mut l = code.len();
    while l > 50 
        && code[l-1]==0x29  && code[l-2]==0x00
        && code[l-35]==0x20 && code[l-36]==0x58  
        && code[l-37]==0x30 && code[l-38]==0x72  
        && code[l-39]==0x7a && code[l-40]==0x7a  
        && code[l-41]==0x62 && code[l-42]==0x65  
        && code[l-43]==0xa1  {

        l -= 43;
    }

    // compiled code includes the swarm hash (32 bytes) + 00 29
    if code.len() != binruntime.len() || code.len() < 34 {
        warn!("blockchain {}",code.to_hex::<String>());
        warn!("compiled   {}",binruntime.to_hex::<String>());
        Err(Error::ContractInvalid)
    } else if code[0..l] != binruntime[0..l] {
        warn!("blockchain {}",code.to_hex::<String>());
        warn!("compiled   {}",binruntime.to_hex::<String>());
        Err(Error::CodeDoesNotMatch)
    } else {
        Ok(())
    }
}

pub struct ContractParser {
    pub abis : HashMap<Address,ethabi::Contract>,
}

pub struct CallInfo<'a> {
    pub func : &'a str,
    pub params : Vec<(&'a String,ethabi::Token)>,
}

impl ContractParser {
    
    pub fn new() -> Self {
        ContractParser { abis : HashMap::new() }
    }
    pub fn add(&mut self, addr: Address, abistr : &str) -> Result<()> {
        if !self.abis.contains_key(&addr) {
            let abi = ethabi::Contract::load(abistr.as_bytes())?;
            self.abis.insert(addr, abi);
        }
        Ok(())
    }
    pub fn contains(&self, addr: &Address)-> bool {
        self.abis.contains_key(addr)
    }
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
    
    pub fn log_eventparams(&self, addr: &Address, txlog: web3::types::Log) -> Result<(&str,ethabi::Log)> {
        if let Some(abi) = self.abis.get(addr) {
            let event = abi.events().find(|e| e.signature()==txlog.topics[0]);

            if let Some(event) = event {
                let rawlog = ethabi::RawLog{topics: txlog.topics,data: txlog.data.0};
                let log = event.parse_log(rawlog)?;

                Ok((&event.name,log))
            } else {
                Err(Error::EventNotFound)
            }
        } else {
            Err(Error::ContractNotFound)
        }
    }
}

fn short_signature(name: &str, params: &[ParamType]) -> [u8; 4] {
	let mut result = [0u8; 4];
	fill_signature(name, params, &mut result);
	result
}

fn fill_signature(name: &str, params: &[ParamType], result: &mut [u8]) {
	let types = params.iter()
		.map(Writer::write)
		.collect::<Vec<String>>()
		.join(",");

	let data: Vec<u8> = From::from(format!("{}({})", name, types).as_str());
    keccak_hash::keccak_256(&data,result);
}