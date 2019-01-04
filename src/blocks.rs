use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use crate::{Start, Parsed};
use std::collections::BTreeMap;
use bitcoin::BlockHeader;
use bitcoin::util::hash::BitcoinHash;

pub struct Blocks {
    sender : Sender<Option<Parsed>>,
    receiver : Receiver<Option<Parsed>>,
}

impl Blocks {
    pub fn new() -> Blocks {
        let (sender, receiver) = channel();
        Blocks {
            sender,
            receiver,
        }
    }
}

impl Start for Blocks {
    fn start(&self) {
        println!("starting blocks processer");

        let mut headers_sizes : BTreeMap<u32, (BlockHeader, u32)> = BTreeMap::new();
        loop {
            let received = self.receiver.recv().unwrap();
            match received {
                Some(received) => {
                    headers_sizes.entry(received.height).or_insert((received.block_header, received.size));
                },
                None => break,
            }
        }

        let mut sum_size = 0u64;
        let mut max_size = 0;
        let mut min_hash = "Z".to_string();
        for (k, v) in headers_sizes {
            let (header, size) = v;

            let cur_hash = header.bitcoin_hash().to_string();
            if min_hash > cur_hash {
                min_hash = cur_hash;
                println!("min hash: {} at height {}", min_hash, k );
            }
            if max_size < size {
                max_size = size;
                println!("max size: {} at height {}", max_size, k );
            }

            sum_size += size as u64;
        }

        println!("sum = {}", sum_size);
        println!("ending blocks processer");
    }

    fn get_sender(&self) -> Sender<Option<Parsed>> {
        self.sender.clone()
    }
}
