use crate::parse::BlockParsed;
use crate::{Startable};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::collections::BTreeMap;
use bitcoin::util::hash::BitcoinHash;

pub struct Blocks {
    sender : Sender<Option<BlockParsed>>,
    receiver : Receiver<Option<BlockParsed>>,
}

impl Blocks {
    pub fn new() -> Blocks {
        let (sender, receiver) = channel();
        Blocks {
            sender,
            receiver,
        }
    }

    pub fn get_sender(&self) -> Sender<Option<BlockParsed>> {
        self.sender.clone()
    }
}

impl Startable for Blocks {
    fn start(&self) {
        println!("starting blocks processer");

        let mut headers_sizes : BTreeMap<u32, BlockParsed> = BTreeMap::new();
        loop {
            let received = self.receiver.recv().expect("can't receive in blocks");
            match received {
                Some(received) => {
                    headers_sizes.entry(received.height).or_insert(received);
                },
                None => break,
            }
        }

        let mut sum_size = 0u64;
        let mut max_size = 0;
        let mut min_hash = "Z".to_string();
        for (k, v) in headers_sizes {
            let header = v.block_header;
            let size = v.size;

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
}
