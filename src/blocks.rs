use crate::parse::BlockParsed;
use crate::{Startable};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use bitcoin::util::hash::BitcoinHash;
use std::time::Instant;
use std::time::Duration;

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
        let mut sum_size = 0u64;
        let mut max_size = 0;
        let mut min_hash = "Z".to_string();
        let mut wait_time = Duration::from_secs(0);

        loop {
            let instant = Instant::now();
            let received = self.receiver.recv().expect("can't receive in blocks");
            wait_time += instant.elapsed();
            match received {
                Some(block) => {
                    let header = block.block_header;
                    let size = block.size;

                    let cur_hash = header.bitcoin_hash().to_string();
                    if min_hash > cur_hash {
                        min_hash = cur_hash;
                        println!("min hash: {} at height {}", min_hash, block.height );
                    }
                    if max_size < size {
                        max_size = size;
                        println!("max size: {} at height {}", max_size, block.height );
                    }

                    sum_size += size as u64;
                },
                None => break,
            }
        }

        println!("sum = {}", sum_size);
        println!("ending blocks processer, wait time: {:?}", wait_time );
    }
}
