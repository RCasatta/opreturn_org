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
    blocks: HashMap<Sha256dHash, BlockExtra>,  // hash, block
}

impl OutOfOrderBlocks {
    fn new() -> Self {
        OutOfOrderBlocks {
            blocks : HashMap::new(),
        }
    }

    fn add(&mut self, mut block_extra: BlockExtra) {
        let prev_hash = block_extra.block.header.prev_blockhash;
        let hash = block_extra.block.header.bitcoin_hash();
        for (key,value) in self.blocks.iter() {
            if value.block.header.prev_blockhash == hash {
                block_extra.next = Some(*key);
            }
        }
        if let Some(mut prev_block) = self.blocks.get_mut(&prev_hash) {
            prev_block.next = Some(hash);
        }
        self.blocks.insert(hash, block_extra);
    }

    fn exist_and_has_next(&self, hash: &Sha256dHash) -> bool {
        if let Some(block) = self.blocks.get(hash)  {
            if let Some(next) = block.next {
                return self.blocks.get(&next).is_some();
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
            next: Sha256dHash::from_hex("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f").unwrap(),
            blocks: OutOfOrderBlocks::new(),
        }
    }

    fn send(&mut self, mut block_extra : BlockExtra) {
        self.next = block_extra.next.unwrap();
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
