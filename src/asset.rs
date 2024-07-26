//! Assets

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::exit;
use log::{error, info};
use crate::{main_manifest, version_manifest, url, aria2, asset_manifest, verify};
use crate::verify::CorruptedReason;

/// Sync all assets
pub fn sync() {
    info!("Loading links");
    let mut files = HashMap::<String, [u8; 20]>::new();
    for (m, _) in main_manifest::manifests().iter() {
        let vm = version_manifest::Version::from_url(m);
        for (l, lh) in vm.links().iter() {
            files.insert(l.clone(), *lh);
        };
        for (l, lh) in asset_manifest::AssetIndex::from_url(&vm.asset_index().0).urls().iter() {
            files.insert(l.clone(), *lh);
        };
    };

    let mut total: usize = 0;
    for _ in 0..2 {
        info!("Verifying {} files", files.keys().len());
        let res = verify::verify_url(&files);

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
        error!("Redownload didn't fix files");
        exit(1);
    };
}