use web3::api::Namespace;
use web3::helpers::{CallFuture};
use web3::types::{U256,Address,Transaction};
use web3::Transport;
use rustc_hex::{FromHexError};
use types::*;

/// `Dbg` namespace
#[derive(Debug, Clone)]
pub struct Dbg<T> {
    transport: T,
}

impl<T: Transport> Namespace<T> for Dbg<T> {
    fn new(transport: T) -> Self
    where
        Self: Sized,
    {
        Dbg { transport }
    }

    fn transport(&self) -> &T {
        &self.transport
    }
}

#[derive(Clone,Debug,Serialize,Deserialize)]
pub struct DbgCallEntry {
    pub from: String,
    pub gas: String,
    #[serde(rename = "gasUsed")] 
    pub gas_used: String,
    pub input: String,
    pub output: String,
    pub to: String,
    #[serde(rename = "type")] 
    pub op: String,
    pub value: String,
}

#[derive(Debug,Serialize,Deserialize)]
pub struct DbgInternalTxs {
    calls : Vec<DbgCallEntry>,
}

impl DbgInternalTxs {
    pub fn parse(&self) -> Result<Vec<InternalTx>,FromHexError> {

        let zero_u256 = U256::default();

        let mut itxs = Vec::new();

        fn opthex_to_addr(s:&str) -> Result<Option<Address>,FromHexError> {
            if s.is_empty() {
                Ok(None)
            } else {
                Ok(Some(hex_to_addr(s)?))
            }
        } 
        for call in &self.calls {
            info!("call.value=={}", call.value);
            itxs.push(InternalTx{
                from     : hex_to_addr(&call.from)?,
                to       : opthex_to_addr(&call.to)?,
                contract : opthex_to_addr(&call.to)?, // TODO: fix internal contract creation
                input    : hex_to_vec(&call.input)?,
                value    : if call.value == "0x0" { zero_u256 } else { hex_to_u256(&call.value)? }
            })
        }
        Ok(itxs)
    }
}

impl<T: Transport> Dbg<T> {
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
