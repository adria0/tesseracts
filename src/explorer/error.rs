use reqwest;
use ethabi;

use super::super::db;
use super::super::eth;

#[derive(Debug)]
pub enum Error {
    Unexpected,
    NotFound,
    Handlebars(handlebars::RenderError),
    Reqwest(reqwest::Error),
    Eth(eth::Error),
    Io(std::io::Error),
    Db(db::Error),
    EthAbi(ethabi::Error),
    Contract(eth::contract::Error),
}

impl From<handlebars::RenderError> for Error {
    fn from(err: handlebars::RenderError) -> Self {
        Error::Handlebars(err)
    }
}
impl From<eth::Error> for Error {
    fn from(err: eth::Error) -> Self {
        Error::Eth(err)
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

impl From<eth::contract::Error> for Error {
    fn from(err: eth::contract::Error) -> Self {
        Error::Contract(err)
    }
}

pub type Result<T> = std::result::Result<T,Error>;

