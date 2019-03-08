#[derive(Debug)]
pub enum Error {
    InvalidOption(String),
    Io(std::io::Error),
    Toml(toml::de::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Toml(err)
    }
}

pub type Result<T> = std::result::Result<T,Error>;
