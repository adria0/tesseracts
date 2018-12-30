use rocksdb::{open_default,DB};
use std::sync::atomic::AtomicUsize;
use serde_cbor::{to_vec,from_slice};
use model::*;

struct AppDB {
    db  : DB,
}

enum AppDBError {
    Engine(rocksdb::Error)
}

/*
impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Id::Addr(addr) => {
                let mut state = serializer.serialize_tuple_variant("Id",0name: &'static str, variant_index: u32, variant: &'static str, len: usize).serialize_struct("Color", 3)?;
                if let Err(err) = serializer.serialize_char('a') {
                    return Err(err);
                }
                if let Err(err) = serializer.serialize_bytes(addr) {
                    return Err(err);
                }
            },
            Id::Tx(tx) => {
                if let Err(err) = serializer.serialize_char('t') {
                    return Err(err);
                }
                if let Err(err) = serializer.serialize_bytes(tx) {
                    return Err(err);
                }
            },
            Id::Block(blk) => {
                if let Err(err) = serializer.serialize_char('b') {
                    return Err(err);
                }
                if let Err(err) = serializer.serialize_u64(*blk) {
                    return Err(err);
                }
            }
        }
    Ok(S::Ok)
}
*/

/// RocksDb is not a multimap, so, we cannot store In order to prevent 
impl AppDB {
    
    fn new() -> Result<AppDB, AppDBError> {
        match DB::open_default("db") {
            DB::Error(err) => Err(AppDBError::Engine(err)),
            Ok(db) => Ok(AppDB { db : db })
        }        
    }
    
    /// this is stored in the following way ----------------------------------------------
    /// from : account, padded 32 bytes 
    /// type : IsTxFrom || IsTxTo
    /// to   : tx
    fn add_link(&self, from: &Id, linktype: &LinkType, to: &Id, ) -> Result<(),AppDBError> {
        let mut key : Vec<u8> = vec![];

        match from {
            Id::Addr(a) => {
                key.push(b'a');
                key.extend_from_slice(a);
            },
            Id::Tx(h) => {
                key.push(b't');
                key.extend_from_slice(h);
            }
            Id::Block(n) => {
                key.push(b'b');
                key.extend_from_slice(&n.to_le_bytes());
            }
        }
        while key.len() < 33 {
            key.push(0);
        }
        key.push(linktype.id());
        
        return 

//        self.put()
    }
}
