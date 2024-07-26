//! Url <-> path utils.
use std::ffi::OsStr;
use url::Url;
use urlencoding::{decode, encode};
use std::path::{Component, Path, PathBuf};
use std::fs::File;
use std::io::Read;
use log::debug;


/// Get local path from url.
pub fn path(url_str: &str) -> PathBuf {
    debug!("Decoding url: {url_str}");
    let url = Url::parse(url_str).expect(&format!("Couldn't parse: {url_str}"));
    Path::new(url.host_str().expect(&format!("No host: {url_str}")))
        .join(&decode(url.path()).expect(&format!("Undecodable path: {url_str}"))[1..])
}

/// Get url from local path.
pub fn url(path: &Path) -> String {
    let mut comp = path.components();
    let Component::Normal(host) = comp.next().expect(&format!("First component missing in '{}'", path.display()))
        else {
            panic!("Path must begin with component: {}", path.display());
        };
    let mut ret = String::from("https://");
    ret.push_str(host.to_str().expect(&format!("Convert '{:?}' to string", host)));
    for p in comp {
        ret.push('/');
        ret.push_str(&encode(&*p.as_os_str().to_string_lossy()));
    }
    ret
}
