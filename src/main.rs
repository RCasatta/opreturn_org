extern crate bitcoin;

use std::path::PathBuf;
use std::fs;
use bitcoin::consensus::encode::Decodable;
use bitcoin::consensus::deserialize;
use bitcoin::Block;
use std::env;
use std::io::Cursor;
use std::io::SeekFrom;
use std::io::Seek;
use bitcoin::network::constants::Network;
use std::sync::mpsc::sync_channel;
use std::thread;
use std::sync::mpsc::SyncSender;

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
    let thread = 4usize;

    let mut handles = vec![];
    let mut senders : Vec<SyncSender<Vec<u8>>> = vec![];
    for i in 0..thread {
        let (send_blocks, receive_blocks) = sync_channel(0);
        senders.push(send_blocks);
        let handle = thread::spawn( move || {
            loop {
                match receive_blocks.recv() {
                    Ok(blob) => {
                        let blocks = parse_blocks(blob, Network::Bitcoin.magic());
                        println!("{} thread received {} blocks", i, blocks.len());
                    },
                    Err(e) => {
                        eprintln!("erro {:?}", e);
                        break;
                    },
                }
            }
        });
        handles.push(handle);
    }

    let handle = thread::spawn( move || {
        let mut i = 0usize;
        for path in paths.iter() {
            let blob = fs::read(path).expect(&format!("failed to read {:?}", path));
            let len = blob.len();
            println!("read {:?}", len);
            senders[i%thread].send(blob).expect("cannot send");
            i=i+1;
        }
    });

    handle.join().unwrap();
    for handle in handles {
        handle.join().unwrap();
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
                    cursor.seek(SeekFrom::Current(-3)).expect("failed to seek back");
                    eprintln!("seek back");
                    continue;
                }
            }
            Err(_) => break, // EOF
        };
        let block_size = u32::consensus_decode(&mut cursor).expect("a");
        let start = cursor.position() as usize;
        cursor.seek(SeekFrom::Current(block_size as i64)).expect("failed to seek forward");
        let end = cursor.position() as usize;

        match deserialize(&blob[start..end]) {
            Ok(block) => blocks.push(block),
            Err(e) => eprintln!("error block parsing {:?}", e ),
        }
    }
    blocks
}