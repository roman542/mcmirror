use clap::Parser;
use std::path::{PathBuf, Path};
use std::io;
use log::info;
use pretty_env_logger;
use std::process;
use std::env::{self, set_current_dir, current_dir};
use std::fmt::Formatter;
use std::fs::create_dir_all;
use serde::{Deserialize, Deserializer};
use serde::de::{Error, Visitor};

mod aria2;
mod url;
mod main_manifest;
mod verify;
mod version_manifest;
mod asset_manifest;
mod asset;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Output repository
    #[arg(value_name = "DIR")]
    repo: PathBuf,
}

struct Sha1Visitor;

impl<'de> Visitor<'de> for Sha1Visitor {
    type Value = [u8; 20];

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("Hex string of sha1")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
        let hash = hex::decode(&v).expect("Couldn't convert hash");
        Ok(hash.try_into().expect("Cannot fit hash into array"))
    }
}
/// Converts String to [u8; 20]
#[derive(Debug)]
pub struct Sha1 (pub [u8; 20]);

impl<'de> Deserialize<'de> for Sha1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        Ok(Sha1(deserializer.deserialize_str(Sha1Visitor).unwrap()))
    }
}


fn chdir_creat(path: &Path) {
    if let Err(e) = set_current_dir(path) {
        match e.kind() {
            io::ErrorKind::NotFound => {
                create_dir_all(path).expect(&format!("Cannot create directory '{}'", path.display()));
                set_current_dir(path).expect(&format!("Cannot change directory to '{}'", path.display()));
                info!("Switched to directory: {}", current_dir().unwrap().display());
            },
            _ => {
                panic!("Cannot change directory to '{}': {:?}", path.display(), e);
            }
        };
    };
    info!("Switched to directory: {}", current_dir().unwrap().display());
}

fn main() {
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    };

    chdir_creat(&args.repo);

    main_manifest::sync();
    version_manifest::sync();
    asset_manifest::sync();
    asset::sync();
}
