/*
use web3::types::{Address,H256,U256,BlockId,Transaction,TransactionId,BlockNumber,Bytes};
use model::{hex_to_addr,hex_to_hash};
use serde::{Serialize,Serializer};

#[derive(Debug)]
enum RecordStreamError {
    EOF
}

struct SliceReader<'a>(&'a [u8]);

impl<'a> SliceReader<'a> {
    fn from(s : &'a [u8]) -> SliceReader {
        SliceReader(s)
    }
    fn u8(&'a mut self) -> Result<(u8,&'a mut Self),RecordStreamError> {
        if self.0.len() < 1 {
            Err(RecordStreamError::EOF)
        } else {
            let v = self.0[0];
            self.0 = &self.0[1..];
            Ok((v,self))
        }
    }
}

impl SliceReader<u64> for [u8] {
    fn sread<'a>(&mut self, v : &mut u64) -> Result<&Self,RecordStreamError> {
        if self.len() < 1 {
            Err(RecordStreamError::EOF)
        } else {
            let mut le = [0;8];
            le[..].copy_from_slice(&self[..8]);
            *v = u64::from_le_bytes(le);
            Ok(&self[8..])
        }
    }
}

impl SliceReader<Address> for [u8] {
    fn sread<'a>(&mut self, v : &mut Address) -> Result<&Self,RecordStreamError> {
        if self.len() < 1 {
            Err(RecordStreamError::EOF)
        } else {
            *v = Address::from_slice(&self[..20]);
            Ok(&self[20..])
        }
    }
}

impl SliceReader<H256> for [u8] {
    fn sread<'a>(&mut self, v : &mut H256) -> Result<&Self,RecordStreamError> {
        if self.len() < 1 {
            Err(RecordStreamError::EOF)
        } else {
            *v = H256::from_slice(&self[..32]);
            Ok(&self[32..])
        }
    }
}

trait VectorWriter {
    fn swrite_u8<'a>(&mut self, v : u8) -> &mut Self;
    fn swrite_u64<'a>(&mut self, v : u64) -> &mut Self;
    fn swrite_addr<'a>(&mut self, v : &Address) -> &mut Self;
    fn swrite_h256<'a>(&mut self, v : &H256) -> &mut Self;
}

impl VectorWriter for Vec<u8>  {
    fn swrite_u8(&mut self, v : u8) -> &mut Self {
        self.push(v);
        self       
    }
    fn swrite_u64(&mut self, v : u64) -> &mut Self {
        self.extend_from_slice(&v.to_le_bytes());
        self       
    }
    fn swrite_addr<'a>(&mut self, v : &Address) -> &mut Self {
        self.extend_from_slice(&v);
        self       
    }
    fn swrite_h256<'a>(&mut self, v : &H256) -> &mut Self {
        self.extend_from_slice(&v);
        self       
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serial_u8() {
        let mut stream : Vec<u8> = Vec::new();
        stream.swrite_u8(2u8).swrite_u8(3u8);

        let reader = SliceReader::from(stream.as_slice());
        let (u1,_) = reader.u8().unwrap();
        let (u2,_) = reader.u8().unwrap();

        assert_eq!(2u8,u1);
        assert_eq!(3u8,u2);
    }

    #[test]
    fn test_serial_u64() {
        let mut stream : Vec<u8> = Vec::new();
        stream.swrite_u64(2001u64).swrite_u64(399181u64);

        let mut u1 : u64 = 0;
        let mut u2 : u64 = 0;
        let e = stream.as_slice()
            .sread(&mut u1)
            .and_then(|s| s.sread(&mut u2))
            .expect("unserialize");   
        
        assert_eq!(2001u64,u1);
        assert_eq!(399181u64,u2);
    }
    #[test]
    fn test_serial() {
        let a1 = hex_to_addr("0x1eb983836ea12dc37cc4da2effae9c9fbd0b395a").unwrap();
        let h1 = hex_to_hash("0xd69fc1890a1b2742b5c2834d031e34ba55ef3820d463a8d0a674bb5dd9a3b74b").unwrap();

        let mut stream : Vec<u8> = Vec::new();
        stream
            .swrite_u8(12)
            .swrite_u64(399181u64)
            .swrite_addr(&a1)
            .swrite_h256(&h1);

        let mut a1 : u8 = 0;
        let mut a2 : u64 = 0;
        
        let e = stream.as_slice()
            .sread(&mut u1)
            .and_then(|s| s.sread(&mut u2))
            .expect("unserialize");   
        
        assert_eq!(2001u64,u1);
        assert_eq!(399181u64,u2);
    }

}
*/