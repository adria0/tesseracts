use std::process::Command;
use std::collections::HashMap;
use std::io::prelude::*;
use std::fs;
use std::fs::File;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use rustc_hex::{FromHex,ToHex};

use ethabi;

use bootstrap::Config;
use super::error::{Error,Result};

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

/// get the solc complilers intalled in config solc_path
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

/// verify if an abi is ok
pub fn verify_abi(source: &str) -> Result<()>{
    ethabi::Contract::load(source.as_bytes())?;
    Ok(())
}

/// compile a contract and check if it maches with blockchain bytecode
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


/// check if two bytecodes matches
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