use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use toml;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u32,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
}


pub fn read_config() -> Config {
    let mut file = File::open("config.toml").expect("config.toml file required");
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let config: Config = toml::from_str(&contents).unwrap();

    return config;
}