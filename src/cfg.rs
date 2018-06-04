use structopt;
use config::{Config,File,FileFormat};
use std::collections::HashMap;

#[derive(Debug,StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
pub struct Cli {
    /// Update channel to follow.
    #[structopt(short = "c", long = "channel")]
    pub channel: String,

    /// Verbose mode
    #[structopt(short = "v", long = "verbose")]
    pub verbose: bool,
}

pub fn read_cfg(loc: &str) -> HashMap<String,String> {
    let mut cfg = Config::default();
    cfg.merge(File::new(loc,FileFormat::Toml)).unwrap();

    let mut values = cfg.get_table("channels").unwrap();

    let mut out = HashMap::new();

    values.drain().for_each(|p|{
        let value = p.1.into_str().unwrap();
        out.insert(p.0,value);
    });

    out
}