//! Verification of files.
use std::collections::HashMap;
use std::path::{PathBuf, Path};
use std::fs::File;
use std::io::{ErrorKind, Read};
use log::info;
use sha1::{Digest, Sha1};
use crate::url;

pub enum CorruptedReason {
    /// Ok status is internal.
    Ok,
    Missing,
    Corrupted,
}

fn hash_file(fd: &mut File) -> [u8; 20] {
    let mut hasher = Sha1::new();
    let mut buf: [u8; 4096] = [0; 4096];
    loop {
        match fd.read(&mut buf) {
            Ok(len) => {
                if len == 0 {
                    break;
                };
                hasher.update(&buf[..len])
            },
            Err(e) => panic!("Cannot read file: {}", e),
        };
    };
    return hasher.finalize().into();
}

fn verify_file(file: &Path, hash: &[u8; 20]) -> CorruptedReason {
    let mut fd = {
        match File::open(file) {
            Ok(f) => f,
            Err(e) => {
                match e.kind() {
                    ErrorKind::NotFound => return CorruptedReason::Missing,
                    _ => panic!("Cannot access file '{}': {:?}", file.display(), e)
                }
            }
        }
    };

    return if &hash_file(&mut fd) != hash {
        CorruptedReason::Corrupted
    } else {
        CorruptedReason::Ok
    };
}

/// Verify files.
///
/// items is map<path, hash_sha1>.
///
/// Returns map<path, reason> of failed files (either corrupted or missing).
pub fn verify(files: &HashMap<PathBuf, [u8; 20]>) -> HashMap<PathBuf, CorruptedReason> {
    let mut corrupted = HashMap::<PathBuf, CorruptedReason>::new();
    for (path, hash)  in files.iter() {
        match verify_file(path, hash) {
            CorruptedReason::Ok => continue,
            c => {corrupted.insert(path.clone(), c)}
        };
    }
    return corrupted;
}

/// Verify files.
///
/// This is the same as [verify], but automatically transforms urls into paths.
pub fn verify_url(urls: &HashMap<String, [u8; 20]>) -> HashMap<PathBuf, CorruptedReason> {
    let files = {
        let mut ret = HashMap::<PathBuf, [u8; 20]>::new();
        for (url, hash) in urls.iter() {
            ret.insert(url::path(url), *hash);
        }
        ret
    };
    verify(&files)
}