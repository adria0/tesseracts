use db;
use std::io;

#[derive(Debug)]
pub enum Error {
    Web3(web3::Error),
    DB(db::Error),
    FromHex(rustc_hex::FromHexError),
    EthAbi(ethabi::Error),
    SerdeJson(serde_json::Error),
    Io(std::io::Error),
    Time(std::time::SystemTimeError),
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

impl From<web3::Error> for Error {
    fn from(err: web3::Error) -> Self {
        Error::Web3(err)
    }
}
impl From<db::Error> for Error {
    fn from(err: db::Error) -> Self {
        Error::DB(err)
    }
}
impl From<std::time::SystemTimeError> for Error {
    fn from(err: std::time::SystemTimeError) -> Self {
        Error::Time(err)
    }
}



pub type Result<T> = std::result::Result<T,Error>;

