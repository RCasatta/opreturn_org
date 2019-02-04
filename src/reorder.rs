use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use bitcoin::util::hash::Sha256dHash;
use bitcoin::Block;
use bitcoin::BitcoinHash;
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
        let b = BlockSizeHeight { block: block_size.block, size: block_size.size, height: self.height };
        self.sender.send(Some(b)).expect("reorder: cannot send block");
        self.height += 1;
        if self.height % 1000 == 0 || self.out_of_order_blocks.len() > 200 {
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
        self.sender.send(None).expect("reorder cannot send none");
    }
}
