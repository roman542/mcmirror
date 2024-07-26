//! Sync main manifest.

use std::arch::x86_64::_mm256_permute_ps;
use std::collections::HashMap;
use std::path::PathBuf;
use std::string::ToString;
use log::info;
use crate::url;
use crate::aria2;
use crate::Sha1;
use serde::Deserialize;
use serde_json as json;
use std::fs::File;

const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// Download main manifest.
pub fn sync() {
    info!("Downloading main manifest: {MANIFEST_URL}");
    let mut file = HashMap::<String, PathBuf>::new();
    file.insert(MANIFEST_URL.to_string(), url::path(MANIFEST_URL));
    aria2::download(file);
}

#[derive(Deserialize, Debug)]
struct Manifest {
    versions: Vec<Version>
}

#[derive(Deserialize, Debug)]
struct Version {
    url: String,
    sha1: Sha1,
}

/// Get version manifests and their hashes.
///
/// String is url, array is sha1 hash.
pub fn manifests() -> HashMap<String, [u8; 20]> {
    let manifest: Manifest = {
        let file = File::open(url::path(MANIFEST_URL)).expect(&format!("Couldn't open '{MANIFEST_URL}"));
            json::de::from_reader(file).expect(&format!("Couldn't parse json '{MANIFEST_URL}'"))
    };
    let mut ret = HashMap::<String, [u8; 20]>::new();
    for v in manifest.versions.iter() {
        ret.insert(v.url.clone(), v.sha1.0);
    }
    ret
}