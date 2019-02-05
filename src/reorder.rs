use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use bitcoin::util::hash::Sha256dHash;
use bitcoin::BitcoinHash;
use std::collections::HashMap;
use crate::BlockExtra;

pub struct Reorder {
    receiver: Receiver<Option<BlockExtra>>,
    sender: SyncSender<Option<BlockExtra>>,
    height: u32,
    next: Sha256dHash,
    blocks : OutOfOrderBlocks,
}

struct OutOfOrderBlocks {
    blocks: HashMap<Sha256dHash, BlockExtra>
}

impl OutOfOrderBlocks {
    fn new() -> Self {
        OutOfOrderBlocks {
            blocks : HashMap::new(),
        }
    }
    fn add(&mut self, block_extra: BlockExtra) {
        let hash = block_extra.block.bitcoin_hash();
        if let Some(mut prev_block) = self.blocks.get_mut(&hash) {
            prev_block.next = Some(hash);
        }
        self.blocks.insert(block_extra.block.bitcoin_hash(), block_extra);
    }

    fn exist_and_has_next(&self, hash: &Sha256dHash) -> bool {
        if let Some(block) = self.blocks.get(hash)  {
            if let Some(next) = block.next {
                if let Some(_) = self.blocks.get(&next) {
                    return true
                }
            }
        }
        false
    }

    fn remove(&mut self, hash: &Sha256dHash) -> Option<BlockExtra> {
        if self.exist_and_has_next(hash) {
            self.blocks.remove(hash)
        } else {
            None
        }
    }
}



impl Reorder {
    pub fn new(receiver : Receiver<Option<BlockExtra>>, sender : SyncSender<Option<BlockExtra>> ) -> Reorder {
        Reorder {
            sender,
            receiver,
            height: 0,
            next: Sha256dHash::default(),
            blocks: OutOfOrderBlocks::new(),
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
                Some(block_extra) => {
                    self.blocks.add(block_extra);
                    while let Some(block_to_send) = self.blocks.remove(&self.next) {
                        self.send(block_to_send);
                    }
                },
                None => break,
            }
        }
        self.sender.send(None).expect("reorder cannot send none");
        println!("ending reorder");
    }
}
