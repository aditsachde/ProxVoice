use std::{error::Error, path::Path, fs};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub client_id: Option<String>,
    pub channel_id: String,
    pub users: Vec<User>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub name: String,
    pub boost: u32,
    pub discord_id: String,
    pub minecraft_id: String
}

impl Config {
    pub fn new(path: &Path) -> Result<Config, Box<dyn Error>> {
        let config = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config[..])?;
        Ok(config)
    }
}