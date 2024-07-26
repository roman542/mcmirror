//! Get asset manifests
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;
use log::{error, info};
use crate::{main_manifest, version_manifest, Sha1, verify, url, aria2};
use serde::Deserialize;
use serde_json as json;
use crate::verify::CorruptedReason;

const ASSET_REPO_URL: &str = "https://resources.download.minecraft.net/";

pub fn sync() {
    info!("Loading asset index urls...");
    let mut indexes = HashMap::<String, [u8; 20]>::new();
    for (url, _) in main_manifest::manifests().iter() {
        let ver = version_manifest::Version::from_url(url);
        let index = ver.asset_index();
        indexes.insert(index.0, index.1);
    };

    let mut total: usize = 0;
    for _ in 0..2 {
        info!("Verifying {} manifests", indexes.keys().len());
        let res = verify::verify_url(&indexes);

        // All we need is pretty logs.
        let mut corrupted = HashMap::<String, PathBuf>::new();
        let mut missing = HashMap::<String, PathBuf>::new();
        for (file, status) in res.iter() {
            match status {
                CorruptedReason::Missing => missing.insert(url::url(file), file.clone()),
                CorruptedReason::Corrupted => corrupted.insert(url::url(file), file.clone()),
                _ => unreachable!(),
            };
        };
        total = corrupted.keys().len() + missing.keys().len();
        info!("Results: {} corrupted, {} missing, {} total", corrupted.keys().len(), missing.keys().len(), total);
        if total == 0 {
            break;
        }
        aria2::download(corrupted);
        aria2::download(missing);
    };
    if total != 0 {
        error!("Redownload didn't fix manifests");
        exit(1);
    };
}

#[derive(Deserialize, Debug)]
struct Object {
    hash: Sha1,
}

#[derive(Deserialize, Debug)]
pub struct AssetIndex {
    objects: HashMap<String, Object>,
}

impl AssetIndex {
    /// Get assetIndex from url.
    pub fn from_url(url: &str) -> Self {
        let mut file = File::open(url::path(url)).expect(&format!("Couldn't open '{url}'"));
        json::de::from_reader(&mut file).expect(&format!("Couldn't parse '{url}'"))
    }

    /// Get urls
    pub fn urls(&self) -> HashMap<String, [u8; 20]> {
        let mut ret = HashMap::<String, [u8; 20]>::new();
        for (_, a) in self.objects.iter() {
            let mut url = String::from(ASSET_REPO_URL);
            let hash = hex::encode(a.hash.0);
            url.push_str(&hash[..2]);
            url.push('/');
            url.push_str(&hash);
            ret.insert(url, a.hash.0);
        };
        ret
    }
}
