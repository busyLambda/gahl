use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use toml::de;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub project: Project,
    pub clibs: Option<Clibs>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub author: String,
    pub exec_entry: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Clibs {
    pub clibs: Vec<Clib>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Clib {
    pub name: String,
    pub path: String,
    pub flags: Vec<String>,
}

pub fn parse_config() -> Result<Config, String> {
    let file = match std::fs::read_to_string("config.toml") {
        Ok(f) => f,
        Err(err) => return Err(err.to_string()),
    };

    match toml::from_str(&file) {
        Ok(c) => Ok(c),
        Err(err) => Err(err.message().to_string()),
    }
}
