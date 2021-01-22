use crate::BlockExtra;
use bitcoin::consensus::{deserialize, serialize};
use bitcoin::consensus::Decodable;
use bitcoin::network::constants::Network;
use bitcoin::{BitcoinHash, Block, Txid};
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
        println!("ending parser, busy time: {}s", (busy_time / 1_000_000_000));
    }
}

fn parse_blocks(blob: Vec<u8>) -> Vec<BlockExtra> {
    let bytes_to_search = vec![
        hex::decode("8de2fdb04edce612738eb51e14ecc426381f8ed8").unwrap(),  // sha1(bitcoin.pdf)
        hex::decode("b1674191a88ec5cdd733e4240a81803105dc412d6c6708d53ab94fc248f4f553").unwrap(), //sha256(bitcoin.pdf)
        hex::decode("9c1185a5c5e9fc54612808977ee8f548b2258d31").unwrap(),  // ripemd("")
        hex::decode("20a4da2752a61bd6866e8080144fbb560e0b8f0d").unwrap(),  // ripemd(sha256(bitcoin.pdf))
    ];
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
                for (i, hash) in bytes_to_search.iter().enumerate() {
                    if let Some(pos) = find_subsequence(&blob[start..end], &hash[..]) {
                        println!("Found hash#{} at {} in {}", i, pos, block.bitcoin_hash());
                        for tx in block.txdata.iter() {
                            let tx_bytes = serialize(tx);
                            if let Some(pos) = find_subsequence(&tx_bytes[..], &hash[..]) {
                                println!("Found hash#{} at {} in tx {}", i, pos, tx.txid());
                            }
                        }
                    }
                }
                let tx_hashes: HashSet<Txid> = block.txdata.iter().map(|tx| tx.txid()).collect();
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

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
