use crypto_hash::{hex_digest,Algorithm};
use std::{
    io::Read,
    thread,
    fs
};
use super::CLIENT; // Use client already defined earlier

#[derive(Deserialize,Debug,Clone)]
pub struct Channel {
    pub updates: Vec<String>
}

#[derive(Deserialize,Debug,Clone)]
pub struct Update {
    additions: Vec<Addition>,
    deletions: Vec<Deletion>,
}

impl Update {
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
        if let Ok(f) = fs::read(&self.loc) {
            if hex_digest(Algorithm::SHA256,&f) != self.sha256sum {
                self.write_file();
            }
        }
        else {
            self.write_file();
        }
    }
    fn write_file(&self) { // Writes an addition to disk
        if let Ok(r) = CLIENT.get(&self.url).send() {
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
                let _ = fs::remove_file(&self.loc);
            }
        }
    }
}