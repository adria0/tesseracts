mod staticres;
mod config;
mod error;

pub use self::error::{Error,Result};
pub use self::staticres::{load_handlebars_templates,get_resource};
pub use self::config::{Config,GETH_CLIQUE,GETH_POW,GETH_AUTO};
