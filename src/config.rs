use serde_derive::*;
use std::fs;
use std::io::{BufReader, Read};
// use toml::*;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: Option<Host>,
    pub game: Option<Game>,
}

#[derive(Debug, Deserialize)]
pub struct Host {
    pub domain: Option<String>,
    pub port: Option<u16>,
}

#[derive(Debug, Deserialize)]
pub struct Game {
    pub player_number: Option<i32>,
    pub player1_name: Option<String>,
    pub player2_name: Option<String>,
    pub first_color: Option<String>,
    pub board_size: Option<i32>,
    pub time_minutes: Option<i32>,
    pub seconds_read: Option<i32>,
    pub command_interval_msec: Option<u64>,
}

#[allow(dead_code)]
pub fn read_toml(path: String) -> Result<Config, toml::de::Error> {
    let contents = match read_file(path.to_owned()) {
        Ok(s) => s,
        Err(e) => panic!("fail to read file: {}", e),
    };

    println!("contents:\n----\n{}\n----", contents);

    toml::from_str(&contents)
}

#[allow(dead_code)]
fn read_file(path: String) -> Result<String, String> {
    let mut file_content = String::new();

    let mut file_read = fs::File::open(path)
        .map(|f| BufReader::new(f))
        .map_err(|e| e.to_string())?;

    file_read
        .read_to_string(&mut file_content)
        .map_err(|e| e.to_string())?;

    Ok(file_content)
}
