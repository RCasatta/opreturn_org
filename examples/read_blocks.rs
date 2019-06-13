use glob::glob;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let mut path = PathBuf::from(env::var("BITCOIN_DIR").unwrap_or("~/.bitcoin/".to_string()));
    path.push("blocks");
    path.push("blk*.dat");
    println!("listing block files at {:?}", path);
    let mut paths: Vec<PathBuf> = glob::glob(path.to_str().unwrap())
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort();
    println!("block files {:?}", paths);
    for path in paths.iter() {
        let blob = fs::read(path).expect(&format!("failed to read {:?}", path));
        println!("read {:?}", blob.len());
    }
}
