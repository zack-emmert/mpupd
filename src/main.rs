extern crate reqwest;
extern crate crypto_hash;
extern crate semver;

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
    io::{Read,Write,Seek,SeekFrom},
};
use reqwest::Client;

lazy_static! {
    static ref CLI_OPTIONS: cfg::Cli = cfg::Cli::from_args();
    static ref CONFIG: HashMap<String,String> = cfg::read_cfg("mpupd.toml");
    pub static ref CLIENT: Client = Client::new();
}

fn main() {

    let mut ts_file = OpenOptions::new().read(true).write(true).create(true).open("mpupd_latestversion.log").unwrap();
    let mut s = String::new();
    ts_file.read_to_string(&mut s).unwrap();
    let version = match semver::Version::parse(&s) {
        Ok(v) => v,
        Err(_) => semver::Version::new(0,0,0),
    };

    if let Some(url) = get_channel_url(&CLI_OPTIONS.channel) {
        if let Some(mut channel) = toml_request::<types::Channel>(&url) {
            for update in channel.sort_by_version().updates.iter() {
                if update.version() > &version {
                    println!("{:?}",update);
                    if let Some(update_file) = toml_request::<types::UpdateFile>(update.url()) {
                        update_file.update();
                        ts_file.set_len(0).unwrap();
                        ts_file.seek(SeekFrom::Start(0)).unwrap();
                        ts_file.write_all(format!("{}",update.version()).as_bytes()).unwrap();
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

            match toml::from_str(&response.text().unwrap()) {
                Ok(out) => Some(out),
                Err(e) => {
                    eprintln!("{}",e);
                    None
                }
            }
            
        }
        Err(_) => {
            eprintln!("Unable to fetch TOML at {}",url);
            None
        }
    }
}