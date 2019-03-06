
use std::process::Command;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::fs;
use std::fs::File;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;

use bootstrap::Config;
use rustc_hex::{FromHex,ToHex};

use keccak_hash;
use ethabi;
use ethabi::param_type::{Writer, ParamType};

pub static ONLY_ABI : &str = "abi-only";

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

#[derive(Debug)]
pub enum Error {
    ContractInvalid,
    ContractNotFound,
    FunctionNotFound,
    CodeDoesNotMatch,
    CompilerNotFound,
    EventNotFound,
    Io(std::io::Error),
    FromHex(rustc_hex::FromHexError),
    EthAbi(ethabi::Error),
    SerdeJson(serde_json::Error),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
impl From<rustc_hex::FromHexError> for Error {
    fn from(err: rustc_hex::FromHexError) -> Self {
        Error::FromHex(err)
    }
}
impl From<ethabi::Error> for Error {
    fn from(err: ethabi::Error) -> Self {
        Error::EthAbi(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeJson(err)
    }
}

pub fn compilers(cfg : &Config) -> Result<Vec<String>,io::Error> {
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

pub fn verify_abi(source: &str) -> Result<(),Error>{
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

-> Result<String,Error> {

    if compilers(&cfg)?.into_iter().find(|c| c==compiler).is_none() {
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

fn code_equals(contract : &SolcContract, code: &[u8]) -> Result<(),Error> {
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

pub fn call_to_string(abistr : &str, input: &[u8]) -> Result<Vec<String>,Error> {
    let abi = ethabi::Contract::load(abistr.as_bytes())?;

    for func in abi.functions() {

        let paramtypes : Vec<ParamType> = func.inputs.iter().map(|p| p.kind.clone()).collect();
        let sig = short_signature(&func.name,&paramtypes);

        if input.len() >= 4 && input[0..4] == sig[0..4] {
            let mut out = Vec::new();
            out.push(format!("function {}",&func.name));

            if !func.inputs.is_empty() {
                let max_param_length = func.inputs.iter().map(|p| p.name.len()).max().unwrap();        

                for (i,token) in ethabi::decode(&paramtypes, &input[4..])?.iter().enumerate() {
                    let padding = (func.inputs[i].name.len()..max_param_length)
                        .map(|_| " ").collect::<String>();
                    out.push(format!("  [{}{}]  {:?}",func.inputs[i].name,padding,token));
                }
            }

            return Ok(out);
        }
    }

    if abi.fallback {
        let mut out = Vec::new();
        out.push(format!("function ()"));
        return Ok(out);
    } else {
        Err(Error::FunctionNotFound)
    }
}

pub fn log_to_string(abistr : &str, txlog: web3::types::Log) -> Result<Vec<String>,Error> {
    let abi = ethabi::Contract::load(abistr.as_bytes())?;
    let event = abi.events().find(|e| e.signature()==txlog.topics[0]);

    if let Some(event) = event {
        let mut out = Vec::new();
        out.push(format!("event {}",&event.name));
        
        let rawlog = ethabi::RawLog{topics: txlog.topics,data: txlog.data.0};
        let log = event.parse_log(rawlog)?;

        if !log.params.is_empty() {
            let max_param_length = log.params.iter().map(|p| p.name.len()).max().unwrap();        

            for param in log.params {
                let padding = (param.name.len()..max_param_length)
                    .map(|_| " ").collect::<String>();
                out.push(format!("  [{}{}] {:?}",param.name,padding,param.value));
            }
        }
        Ok(out) 
    } else {
        Err(Error::EventNotFound)
    }
}

pub fn short_signature(name: &str, params: &[ParamType]) -> [u8; 4] {
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