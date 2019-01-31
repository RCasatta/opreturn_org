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
use crate::parse::BlockSize;
use std::collections::HashMap;

pub struct BlockSizeHeight {
    pub block: Block,
    pub size: u32,
    pub height: u32,
}

pub struct Reorder {
    receiver : Receiver<Option<BlockSize>>,
    sender : SyncSender<Option<BlockSizeHeight>>,
    height: u32,
    next: Sha256dHash,
    out_of_order_blocks: HashMap<Sha256dHash, BlockSize>,
}

impl Reorder {
    pub fn new(receiver : Receiver<Option<BlockSize>>, sender : SyncSender<Option<BlockSizeHeight>> ) -> Reorder {
        Reorder {
            sender,
            receiver,
            height: 0,
            next: Sha256dHash::default(),
            out_of_order_blocks: HashMap::new(),
        }
    }

    fn send(&mut self, block_size : BlockSize) {
        self.next = block_size.block.bitcoin_hash();
        self.sender.send(Some(BlockSizeHeight { block: block_size.block, size: block_size.size, height: self.height}));
        self.height += 1;
        if self.height % 1000 == 0 {
            println!("out_of_order_size: {}", self.out_of_order_blocks.len());
        }
    }

    pub fn start(&mut self) {
        loop {
            let received = self.receiver.recv().expect("cannot receive blob");
            match received {
                Some(block_size) => {
                    let prev_blockhash = block_size.block.header.prev_blockhash;
                    if prev_blockhash == self.next {
                        self.send(block_size);
                        loop {
                            match self.out_of_order_blocks.remove(&self.next) {
                                Some(value) => self.send(value),
                                None => break,
                            }
                        }
                    } else {
                        self.out_of_order_blocks.insert(prev_blockhash, block_size);
                    }
                },
                None => break,
            }
        }
        self.sender.send(None);
    }
}
