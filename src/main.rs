extern crate reqwest;
extern crate crypto_hash;

extern crate serde;
extern crate toml;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate structopt;
extern crate config;
#[macro_use]
extern crate lazy_static;

mod cfg;
mod types;

use structopt::StructOpt;
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{Read,Write},
};
use reqwest::Client;

lazy_static! {
    static ref CLI_OPTIONS: cfg::Cli = cfg::Cli::from_args();
    static ref CONFIG: HashMap<String,String> = cfg::read_cfg("mpupd.toml");
    pub static ref CLIENT: Client = Client::new();
}

fn main() {

    let mut log = OpenOptions::new().read(true).write(true).append(true).open("mpupd.log").unwrap();
    let mut s = String::new();
    log.read_to_string(&mut s).unwrap();
    let log_data: Vec<String> = {
        let mut out: Vec<String> = s.lines().map(|v|{v.to_owned()}).collect();
        out.sort_unstable();
        out
    };

    if let Some(url) = get_channel_url(&CLI_OPTIONS.channel) {
        if let Some(channel) = toml_request::<types::Channel>(&url) {
            for update_url in channel.updates.iter() {
                if let Err(_) = log_data.binary_search(update_url) {
                    if let Some(update) = toml_request::<types::Update>(update_url) {
                        update.update();
                        log.write_all(format!("{}\n",update_url).as_bytes()).unwrap();
                    }
                }
            }
        }
    }
}

fn get_channel_url(channel: &str) -> Option<String> { // Converts channel CLI option into URL from config

    if let Some(v) = CONFIG.get(channel) {
        Some(v.to_string())
    }
    else {
        eprintln!("Invalid update channel.");
        None
    }
}

fn toml_request<T>(url: &str) -> Option<T> where for<'de> T: serde::Deserialize<'de> { // Sends an HTTP req and returns the deserialized body
    match CLIENT.get(url).send() {
        Ok(mut response) => {

            if let Ok(out) = toml::from_str(&response.text().unwrap()) {
                Some(out)
            }
            else {
                None
            }
        }
        Err(_) => {
            eprintln!("Unable to fetch TOML at {}",url);
            None
        }
    }
}