use db;

#[derive(Debug)]
pub enum Error {
    Uninitialized,
    Web3(web3::Error),
    DB(db::Error),
    FromHex(rustc_hex::FromHexError),
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
impl From<rustc_hex::FromHexError> for Error {
    fn from(err: rustc_hex::FromHexError) -> Self {
        Error::FromHex(err)
    }
}

