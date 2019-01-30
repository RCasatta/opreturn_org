use crate::Startable;
use std::time::Instant;
use std::time::Duration;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use rocksdb::DB;
use rocksdb::WriteBatch;
use bitcoin::util::hash::Sha256dHash;
use bitcoin::consensus::serialize;
use bitcoin::consensus::deserialize;
use bitcoin::VarInt;
use bitcoin::Block;
use bitcoin::BitcoinHash;
use std::io::Cursor;
use std::io::SeekFrom;
use bitcoin::network::constants::Network;
use bitcoin::consensus::Decodable;
use std::io::Seek;

pub struct Parse {
    receiver : Receiver<Option<Vec<u8>>>,
    sender : SyncSender<Option<BlockSize>>,
}

impl Parse {
    pub fn new(receiver : Receiver<Option<Vec<u8>>>, sender : SyncSender<Option<BlockSize>> ) -> Parse {
        Parse {
            sender,
            receiver,
        }
    }

    pub fn start(&mut self) {
        loop {
            let received = self.receiver.recv().expect("cannot receive blob");
            match received {
                Some(blob) => {
                    let blocks_vec = parse_blocks(blob);
                    println!("received {}", blocks_vec.len());
                },
                None => break,
            }
        }
    }
}

pub struct BlockSize {
    block: Block,
    size: u32,
}

fn parse_blocks(blob: Vec<u8>) -> Vec<BlockSize> {
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
            Ok(block) => blocks.push(BlockSize{block, size}),
            Err(e) => eprintln!("error block parsing {:?}", e ),
        }
    }
    blocks
}