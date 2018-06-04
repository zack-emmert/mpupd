use crypto_hash::{hex_digest,Algorithm};
use std::{
    io::Read,
    thread,
    fs
};
use super::{CLIENT,CLI_OPTIONS,SELF_UPDATE_URL};
use semver::Version;
use regex::Regex;

lazy_static! {
    static ref RX_FILENAME: Regex = Regex::new("[^/]+$").unwrap();
}

#[derive(Deserialize,Debug,Clone)]
pub struct Channel {
    #[serde(rename = "update")]
    pub updates: Vec<Update>
}

impl Channel {
    pub fn sort_by_version(mut self) -> Self {
        self.updates.sort_unstable_by(|a,b|{a.version().cmp(&b.version())});

        Channel { updates: self.updates }
    }
}

#[derive(Deserialize,Debug,Clone)]
pub struct Update {
    url: String,
    version: Version,
}

impl Update {
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn version(&self) -> &Version {
        &self.version
    }
}

#[derive(Deserialize,Debug,Clone)]
pub struct UpdateFile {
    #[serde(rename = "addition",default)]
    additions: Vec<Addition>,
    #[serde(rename = "deletion",default)]
    deletions: Vec<Deletion>,
}

impl UpdateFile {
    pub fn update(self) { // Carries out all additions and deletions in a single update
        let d = self.deletions.clone();

        let t = thread::spawn(move || { // Move deletion exec to a separate thread for better performance
            for del in d.iter() {
                del.exec();
            }
        });

        for add in self.additions.iter() {
            add.exec();
        }

        t.join().unwrap();

    }
}

#[derive(Deserialize,Debug,Clone)]
pub struct Addition {
    loc: String,
    url: String,
    sha256sum: String,
}

impl Addition {

    pub fn exec(&self) { // Adds a single file
        if &self.loc == "mpupd" || &self.loc ==  "mpupd.exe" {
            SELF_UPDATE_URL.set(self.loc.clone()).unwrap();
        }
        else {
            if let Ok(f) = fs::read(&self.loc) {
                if hex_digest(Algorithm::SHA256,&f) != self.sha256sum {
                    self.write_file();
                }
            }
                else {
                    self.write_file();
                }
        }
    }
    fn write_file(&self) { // Writes an addition to disk
        if let Ok(r) = CLIENT.get(&self.url).send() {
            if CLI_OPTIONS.verbose {
                let filename = RX_FILENAME.find(&self.url).unwrap().as_str();
                println!("\tDownloading file {}",filename);
                println!("\tWriting file {} to disk at {}",filename,self.loc);
            }
            let file: Vec<u8> = r.bytes().map(|v|{v.unwrap()}).collect();
            let _ = fs::write(&self.loc,&file);
        }
    }
}

#[derive(Deserialize,Debug,Clone)]
pub struct Deletion {
    loc: String,
    sha256sum: String
}

impl Deletion {
    pub fn exec(&self) { // Deletes a single file
        if let Ok(f) = fs::read(&self.loc) {

            if hex_digest(Algorithm::SHA256,&f) == self.sha256sum {

                if CLI_OPTIONS.verbose {println!("Deleting file at {}",self.loc);};

                let _ = fs::remove_file(&self.loc);
            }
        }
    }
}