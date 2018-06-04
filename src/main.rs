extern crate reqwest;
extern crate crypto_hash;
extern crate semver;

extern crate serde;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate regex;

#[macro_use]
extern crate structopt;
extern crate config;
#[macro_use]
extern crate lazy_static;
extern crate mut_static;

mod cfg;
mod types;

use structopt::StructOpt;
use std::{
    collections::HashMap,
    fs::{self,OpenOptions},
    io::{Read,Write,Seek,SeekFrom},
    process::Command,
    ops::Deref,
};
use reqwest::Client;
use mut_static::MutStatic;

lazy_static! {
    pub static ref CLI_OPTIONS: cfg::Cli = cfg::Cli::from_args();
    pub static ref CONFIG: HashMap<String,String> = cfg::read_cfg("mpupd.toml");
    pub static ref CLIENT: Client = Client::new();
    pub static ref SELF_UPDATE_URL: MutStatic<String> = MutStatic::from(String::new());
}

fn main() {

    let mut ts_file = OpenOptions::new().read(true).write(true).create(true).open("version.txt").unwrap();
    let mut s = String::new();
    ts_file.read_to_string(&mut s).unwrap();
    let version = match semver::Version::parse(&s) {
        Ok(v) => {
            if CLI_OPTIONS.verbose {
                println!("Current version: {}",v);
            }
            v
        },
        Err(_) => semver::Version::new(0,0,0),
    };

    // Remove old self-update script if present
    let _ = fs::remove_file("selfupd.cmd");
    let _ = fs::remove_file("selfupd");

    if let Some(url) = get_channel_url(&CLI_OPTIONS.channel) {
        if CLI_OPTIONS.verbose {println!("Channel: {}",&CLI_OPTIONS.channel);}
        if let Some(mut channel) = toml_request::<types::Channel>(&url) {
            for update in channel.sort_by_version().updates.iter() {
                if update.version() > &version {
                    if CLI_OPTIONS.verbose {println!("Updating to version {}",update.version())}
                    if let Some(update_file) = toml_request::<types::UpdateFile>(update.url()) {
                        update_file.update();

                        // Clears, sets file cursor to start, then writes new version
                        ts_file.set_len(0).unwrap();
                        ts_file.seek(SeekFrom::Start(0)).unwrap();
                        ts_file.write_all(format!("{}",update.version()).as_bytes()).unwrap();
                    }
                }
            }
        }
    }
    if SELF_UPDATE_URL.read().unwrap().as_str() != "" {
        self_update();
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

#[cfg(target_os = "windows")]
fn self_update() { // Uses secondary script to update binary

    let url = SELF_UPDATE_URL.read().unwrap();

    let script =
        r#"
        sleep 10
        move mpupd.exe.new mpupd.exe
        "#;

    if let Ok(response) = CLIENT.get(url.deref()).send() {
        let new: Vec<u8> = response.bytes().map(|v|{v.unwrap()}).collect();

        fs::write("mpupd.exe.new",new).unwrap();
        fs::write("selfupd.cmd",script).unwrap();
        let _self_update_script = Command::new("selfupd.cmd").spawn().expect("Failed to spawn self-update process");
    }
}

#[cfg(not(target_os = "windows"))]
fn self_update() {
    use std::os::unix::fs::OpenOptionsExt;

    let url = SELF_UPDATE_URL.read().unwrap();

    let script =
        r##"
        #!/bin/bash
        sleep 10
        mv mpupd.new mpupd
        "##;

    if let Ok(response) = CLIENT.get(url.deref()).send() {
        let new: Vec<u8> = response.bytes().map(|v|{v.unwrap()}).collect();

        fs::write("mpupd.new",new).unwrap();
        fs::write("selfupd",script).unwrap();

        // Ensure self-update script has exec permissions
        {let _ = fs::OpenOptions::new().write(true).mode(0o750).open("selfupd");} // Nested scope ensures file is closed after permissions are set

        let _self_update_script = Command::new("./selfupd").spawn().expect("Failed to spawn self-update process");
    }
}