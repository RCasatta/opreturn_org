use crate::parse::BlockParsed;
use crate::{Startable};
use std::sync::mpsc::{Receiver};
use bitcoin::util::hash::BitcoinHash;
use std::time::Instant;
use std::time::Duration;
use std::sync::mpsc::SyncSender;
use std::sync::mpsc::sync_channel;

pub struct Blocks {
    sender : SyncSender<Option<BlockParsed>>,
    receiver : Receiver<Option<BlockParsed>>,
}

impl Blocks {
    pub fn new() -> Blocks {
        let (sender, receiver) = sync_channel(1000);
        Blocks {
            sender,
            receiver,
        }
    }

    pub fn get_sender(&self) -> SyncSender<Option<BlockParsed>> {
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
        let mut nonce_points = vec![];

        loop {
            let instant = Instant::now();
            let received = self.receiver.recv().expect("can't receive in blocks");
            wait_time += instant.elapsed();
            match received {
                Some(block) => {
                    let header = block.block_header;
                    let size = block.size;
                    let height = block.height;

                    let cur_hash = header.bitcoin_hash().to_string();
                    if min_hash > cur_hash {
                        min_hash = cur_hash;
                        println!("min hash: {} at height {}", min_hash, height );
                    }
                    if max_size < size {
                        max_size = size;
                        println!("max size: {} at height {}", max_size, height );
                    }
                    if block.height % 20000 == 0 {
                        println!("height: {} ", height);
                    }
                    sum_size += size as u64;
                    nonce_points.push((height as f64,block.block_header.nonce as f64));
                },
                None => break,
            }
        }

        println!("sum = {}", sum_size);
        println!("ending blocks processer, wait time: {:?}", wait_time );
    }
}
