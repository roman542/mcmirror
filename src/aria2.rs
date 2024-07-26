use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;

use std::process::{Command, Stdio};

type Items = HashMap<String, PathBuf>;

fn gen_list_item(url: &str, path: &Path) -> String {
    format!("{url}\n\tout={}\n", path.display())
}

fn gen_list(items: Items) -> String {
    let mut ret = String::new();
    for (url, path) in items.iter() {
        ret.push_str(&gen_list_item(url, path));
    };
    ret
}



/// Download all items.
///
/// items is map<url, dest>.
/// You should verify that all the files are correct.
pub fn download(items: Items) {
    let list = gen_list(items).into_bytes();
    let mut aria2 = Command::new("aria2c")
        .args(vec!["-d", ".", "-s", "1", "--allow-overwrite", "true", "-i", "-"])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Couldn't launch child aria2");
    let mut stdin = aria2.stdin.take().expect("Open stdin of aria2c");
    let writer = thread::spawn(move || {
        stdin.write_all(&list[..]).expect("Send list to aria2c");
    });
    aria2.wait().expect("aria3c didn't run");
    writer.join().expect("Couldn't join writer thread");
}