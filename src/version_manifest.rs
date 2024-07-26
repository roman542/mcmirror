//! Version manifests sync.
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::process::exit;
use crate::{aria2, main_manifest};
use crate::verify;
use log::{error, info};
use crate::verify::CorruptedReason;
use crate::url;
use serde::Deserialize;
use serde_json as json;
use crate::Sha1;
/// Sync all version manifests.
pub fn sync() {
    let manifests = main_manifest::manifests();
    let mut total: usize = 0;
    for _ in 0..2 {
        info!("Verifying {} manifests", manifests.keys().len());
        let res = verify::verify_url(&manifests);

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
    }
    if total != 0 {
        error!("Redownload didn't fix manifests");
        exit(1);
    }
}

#[derive(Deserialize, Debug)]
struct AssetIndex {
    url: String,
    sha1: Sha1,
}

#[derive(Deserialize, Debug)]
struct Download {
    url: String,
    sha1: Sha1,
}

#[derive(Deserialize, Debug)]
struct Artifact {
    url: String,
    sha1: Sha1,
}

#[derive(Deserialize, Debug)]
struct Classifier {
    url: String,
    sha1: Sha1,
}

#[derive(Deserialize, Debug)]
struct LibDownload {
    artifact: Option<Artifact>,
    classifiers: Option<HashMap<String, Classifier>>,
}

#[derive(Deserialize, Debug)]
struct Library {
    downloads: LibDownload,
}

#[derive(Deserialize, Debug)]
struct LogFile {
    url: String,
    sha1: Sha1,
}

#[derive(Deserialize, Debug)]
struct Logging {
    file: LogFile,
}

#[derive(Deserialize, Debug)]
pub struct Version {
    #[serde(rename="assetIndex")]
    asset_index: AssetIndex,
    downloads: HashMap<String, Download>,
    libraries: Vec<Library>,
    logging: Option<HashMap<String, Logging>>,
}

impl Version {
    /// Get version manifest from url.
    pub fn from_url(url: &str) -> Self {
        let mut file = File::open(url::path(url)).expect(&format!("Couldn't open '{url}'"));
        json::de::from_reader(&mut file).expect(&format!("Couldn't parse '{url}'"))
    }
    
    /// Get links (except assetIndex).
    pub fn links(&self) -> HashMap<String, [u8;20]> {
        let mut ret = HashMap::<String, [u8;20]>::new();
        for (_, d) in self.downloads.iter() {
            ret.insert(d.url.clone(), d.sha1.0);
        };
        for l in self.libraries.iter() {
            let d = &l.downloads;
            if let Some(a) = &d.artifact {
              ret.insert(a.url.clone(), a.sha1.0);
            };
            if let Some(cls) = &d.classifiers {
                for (_, c) in cls.iter() {
                    ret.insert(c.url.clone(), c.sha1.0);
                };
            };
        };
        if let Some(log) = &self.logging {
            for (_, l) in log.iter() {
                ret.insert(l.file.url.clone(), l.file.sha1.0);
            };
        };
        ret
    }

    /// Get assetIndex url.
    pub fn asset_index(&self) -> (String, [u8; 20]) {
        (self.asset_index.url.clone(), self.asset_index.sha1.0)
    }

    
}