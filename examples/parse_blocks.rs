extern crate bitcoin;

use std::path::PathBuf;
use glob::glob;
use std::fs;
use bitcoin::consensus::encode::Decodable;
use bitcoin::consensus::deserialize;
use bitcoin::Block;
use std::env;
use std::io::Cursor;
use std::io::SeekFrom;
use std::io::Seek;
use bitcoin::network::constants::Network;

fn main() {
    let mut path = PathBuf::from(env::var("BITCOIN_DIR").unwrap_or("~/.bitcoin/".to_string()));
    path.push("blocks");
    path.push("blk*.dat");
    println!("listing block files at {:?}", path);
    let mut paths: Vec<PathBuf> = glob::glob(path.to_str().unwrap()).unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort();
    println!("block files {:?}", paths);
    for path in paths.iter() {
        let blob = fs::read(path).expect(&format!("failed to read {:?}", path));
        let vec = parse_blocks(blob, Network::Bitcoin.magic());
        println!("read {:?} blocks {:?}", blob.len(), vec.len());
    }
}

fn parse_blocks(blob: Vec<u8>, magic: u32) -> Vec<Block> {
    let mut cursor = Cursor::new(&blob);
    let mut blocks = vec![];
    let max_pos = blob.len() as u64;
    while cursor.position() < max_pos {
        match u32::consensus_decode(&mut cursor) {
            Ok(value) => {
                if magic != value {
                    cursor
                        .seek(SeekFrom::Current(-3))
                        .expect("failed to seek back");
                    eprintln!("seek back");
                    continue;
                }
            }
            Err(_) => break, // EOF
        };
        let block_size = u32::consensus_decode(&mut cursor).expect("a");
        let start = cursor.position() as usize;
        cursor
            .seek(SeekFrom::Current(block_size as i64));
        let end = cursor.position() as usize;

        match deserialize(&blob[start..end]) {
            Ok(block) => blocks.push(block),
            Err(e) => eprintln!("error block parsing {:?}", e ),
        }
    }
    blocks
}