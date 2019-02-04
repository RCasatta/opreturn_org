use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use bitcoin::util::hash::Sha256dHash;
use bitcoin::BitcoinHash;
use std::collections::HashMap;
use crate::BlockExtra;

pub struct Reorder {
    receiver : Receiver<Option<BlockExtra>>,
    sender : SyncSender<Option<BlockExtra>>,
    height: u32,
    next: Sha256dHash,
    out_of_order_blocks: HashMap<Sha256dHash, BlockExtra>,
}

impl Reorder {
    pub fn new(receiver : Receiver<Option<BlockExtra>>, sender : SyncSender<Option<BlockExtra>> ) -> Reorder {
        Reorder {
            sender,
            receiver,
            height: 0,
            next: Sha256dHash::default(),
            out_of_order_blocks: HashMap::new(),
        }
    }

    fn send(&mut self, mut block_extra : BlockExtra) {
        self.next = block_extra.block.bitcoin_hash();
        block_extra.height = self.height;
        self.sender.send(Some(block_extra)).expect("reorder: cannot send block");
        self.height += 1;
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
