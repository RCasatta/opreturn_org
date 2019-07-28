use crate::BlockExtra;
use bitcoin::consensus::deserialize;
use bitcoin::consensus::Decodable;
use bitcoin::network::constants::Network;
use bitcoin::Block;
use bitcoin_hashes::sha256d;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Cursor;
use std::io::Seek;
use std::io::SeekFrom;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::time::Instant;

pub struct Parse {
    receiver: Receiver<Option<Vec<u8>>>,
    sender: SyncSender<Option<BlockExtra>>,
}

impl Parse {
    pub fn new(
        receiver: Receiver<Option<Vec<u8>>>,
        sender: SyncSender<Option<BlockExtra>>,
    ) -> Parse {
        Parse { sender, receiver }
    }

    pub fn start(&mut self) {
        let mut total_blocks = 0usize;

        let mut busy_time = 0u128;
        loop {
            let received = self.receiver.recv().expect("cannot receive blob");
            let now = Instant::now();
            match received {
                Some(blob) => {
                    let blocks_vec = parse_blocks(blob);
                    total_blocks += blocks_vec.len();
                    println!("received {} total {}", blocks_vec.len(), total_blocks);
                    busy_time = busy_time + now.elapsed().as_nanos();
                    for block in blocks_vec {
                        self.sender
                            .send(Some(block))
                            .expect("parse: cannot send block");
                    }
                }
                None => break,
            }
        }
        self.sender.send(None).expect("parse: cannot send None");
        println!("ending parser, busy time: {}s", (busy_time / 1_000_000_000));
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
                    cursor
                        .seek(SeekFrom::Current(-3))
                        .expect("failed to seek back");
                    continue;
                }
            }
            Err(_) => break, // EOF
        };
        let size = u32::consensus_decode(&mut cursor).expect("a");
        let start = cursor.position() as usize;
        cursor
            .seek(SeekFrom::Current(i64::from(size)))
            .expect("failed to seek forward");
        let end = cursor.position() as usize;

        match deserialize::<Block>(&blob[start..end]) {
            Ok(block) => {
                let tx_hashes: HashSet<sha256d::Hash> =
                    block.txdata.iter().map(|tx| tx.txid()).collect();
                blocks.push(BlockExtra {
                    block,
                    size,
                    height: 0,
                    next: vec![],
                    outpoint_values: HashMap::new(),
                    out_of_order_size: 0,
                    tx_hashes,
                })
            }
            Err(e) => eprintln!("error block parsing {:?}", e),
        }
    }
    blocks
}
