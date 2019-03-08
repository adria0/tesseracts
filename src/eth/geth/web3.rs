use web3::api::Namespace;
use web3::helpers::{CallFuture};
use web3::types::{U256,Address,Transaction};
use web3::Transport;
use rustc_hex::{FromHexError};

use super::super::types::*;

/// `Debug` namespace
#[derive(Debug, Clone)]
pub struct Debug<T> {
    transport: T,
}

/// A transport for debug_ calls
impl<T: Transport> Namespace<T> for Debug<T> {
    fn new(transport: T) -> Self
    where
        Self: Sized,
    {
        Debug { transport }
    }

    fn transport(&self) -> &T {
        &self.transport
    }
}

/// Entry returned by calltracer 
#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct DbgCallEntry {
    pub from: String,
    pub input: String,
    pub to: String,
    #[serde(rename = "type")] 
    pub op: String,
    pub value: String,
}

/// The structure returned by debug_traceTransaction(calltracer)
#[derive(Debug,Serialize,Deserialize)]
pub struct DbgInternalTxs {
    calls : Option<Vec<DbgCallEntry>>,
}

impl DbgInternalTxs {

    /// Parse hex string address
    fn opthex_to_addr(&self, s:&str) -> Result<Option<Address>,FromHexError> {
        if s.is_empty() {
            Ok(None)
        } else {
            Ok(Some(hex_to_addr(s)?))
        }
    }

    /// Parse debug_ call and return a vector of InternalTx
    pub fn parse(&self) -> Result<Vec<InternalTx>,FromHexError> {
        let zero_u256 = U256::default();
        let mut itxs = Vec::new();
        
        if let Some(calls) = &self.calls { 
            for call in calls {
                if call.op == "CREATE" || call.op == "CREATE2" {
                    itxs.push(InternalTx{
                        from     : hex_to_addr(&call.from)?,
                        to       : None,
                        contract : self.opthex_to_addr(&call.to)?,
                        input    : hex_to_vec(&call.input)?,
                        value    : if call.value == "0x0" { zero_u256 } else { hex_to_u256(&call.value)? }
                    })
                } else {
                    itxs.push(InternalTx{
                        from     : hex_to_addr(&call.from)?,
                        to       : self.opthex_to_addr(&call.to)?,
                        contract : None, 
                        input    : hex_to_vec(&call.input)?,
                        value    : if call.value == "0x0" { zero_u256 } else { hex_to_u256(&call.value)? }
                    })
                }
            }
        }
        Ok(itxs)
    }
}

impl<T: Transport> Debug<T> {
    
    /// Retrieve internal transactions by calling debug_traceTransaction
    pub fn internal_txs(&self, tx: &Transaction) -> CallFuture<DbgInternalTxs, T::Out> {
        CallFuture::new(
            self.transport.execute  (
                "debug_traceTransaction",
                vec![
                    web3::helpers::serialize(&tx.hash),
                    serde_json::from_str("{\"tracer\":\"callTracer\"}").unwrap(),
                ]
        ))
    }
}
