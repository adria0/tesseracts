#[derive(Debug,Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum RecordType {
    TxLink = 1,
    NextBlock = 2,
    Tx = 3,
    Block = 4,
    Receipt = 5,
    ContractAbi = 6,
    TxLinkCount = 7,
    NonEmptyBlock = 8,
    NonEmptyBlockCount = 9,
    IntTx = 10,
}


#[derive(Debug,Serialize,Deserialize)]
pub struct Contract {
    pub source : String,
    pub abi : String,
    pub name : String,
    pub compiler: String,
    pub optimized: bool,
    pub constructor : Vec<u8>, 
}

