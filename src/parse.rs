use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use bitcoin::consensus::deserialize;
use std::io::Cursor;
use std::io::SeekFrom;
use bitcoin::network::constants::Network;
use bitcoin::consensus::Decodable;
use std::io::Seek;
use crate::BlockExtra;
use std::collections::HashMap;

pub struct Parse {
    receiver : Receiver<Option<Vec<u8>>>,
    sender : SyncSender<Option<BlockExtra>>,
}

impl Parse {
    pub fn new(receiver : Receiver<Option<Vec<u8>>>, sender : SyncSender<Option<BlockExtra>> ) -> Parse {
        Parse {
            sender,
            receiver,
        }
    }

    pub fn start(&mut self) {
        let mut total_blocks = 0usize;
        loop {
            let received = self.receiver.recv().expect("cannot receive blob");
            match received {
                Some(blob) => {
                    let blocks_vec = parse_blocks(blob);
                    total_blocks += blocks_vec.len();
                    println!("received {} total {}", blocks_vec.len(), total_blocks);
                    for block in blocks_vec {
                        self.sender.send(Some(block)).expect("parse: cannot send block");
                    }
                },
                None => break,
            }
        }
        self.sender.send(None).expect("parse: cannot send None");
        println!("ending parser");
    }
}

fn parse_blocks(blob: Vec<u8>) -> Vec<BlockExtra> {
    let magic = Network::Bitcoin.magic();
    let mut cursor = Cursor::new(&blob);
    let mut blocks = vec![];
    let max_pos = blob.len() as u64;
    while cursor.position() < max_pos {
        match u32::consensus_decode(&mut cursor) {
            Ok(value) => {
                if magic != value {
                    cursor.seek(SeekFrom::Current(-3)).expect("failed to seek back");
                    continue;
                }
            }
            Err(_) => break, // EOF
        };
        let size = u32::consensus_decode(&mut cursor).expect("a");
        let start = cursor.position() as usize;
        cursor.seek(SeekFrom::Current(size as i64)).expect("failed to seek forward");
        let end = cursor.position() as usize;

        match deserialize(&blob[start..end]) {
            Ok(block) => blocks.push(BlockExtra {block, size, height: 0, next: vec![], outpoint_values: HashMap::new()}),
            Err(e) => eprintln!("error block parsing {:?}", e ),
        }
    }
    blocks
}