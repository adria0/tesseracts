use reqwest;
use ethabi;

use super::super::db;
use super::super::reader;
use super::super::contract;

#[derive(Debug)]
pub enum Error {
    Unexpected,
    NotFound,
    Handlebars(handlebars::RenderError),
    Reqwest(reqwest::Error),
    Reader(reader::Error),
    Io(std::io::Error),
    Db(db::Error),
    EthAbi(ethabi::Error),
    Contract(contract::Error),
}

impl From<handlebars::RenderError> for Error {
    fn from(err: handlebars::RenderError) -> Self {
        Error::Handlebars(err)
    }
}
impl From<reader::Error> for Error {
    fn from(err: reader::Error) -> Self {
        Error::Reader(err)
    }
}
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err)
    }
}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
impl From<db::Error> for Error {
    fn from(err: db::Error) -> Self {
        Error::Db(err)
    }
}
impl From<ethabi::Error> for Error {
    fn from(err: ethabi::Error) -> Self {
        Error::EthAbi(err)
    }
}
impl From<contract::Error> for Error {
    fn from(err: contract::Error) -> Self {
        Error::Contract(err)
    }
}
