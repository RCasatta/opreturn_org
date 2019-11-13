use crate::BlockExtra;
use bitcoin::BitcoinHash;
use bitcoin_hashes::hex::FromHex;
use bitcoin_hashes::sha256d;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;

pub struct Reorder {
    receiver: Receiver<Option<BlockExtra>>,
    sender: SyncSender<Option<BlockExtra>>,
    height: u32,
    next: sha256d::Hash,
    blocks: OutOfOrderBlocks,
}

struct OutOfOrderBlocks {
    blocks: HashMap<sha256d::Hash, BlockExtra>, // hash, block
}

impl OutOfOrderBlocks {
    fn new() -> Self {
        OutOfOrderBlocks {
            blocks: HashMap::new(),
        }
    }

    fn add(&mut self, mut block_extra: BlockExtra) {
        let prev_hash = block_extra.block.header.prev_blockhash;
        let hash = block_extra.block.header.bitcoin_hash();
        for (key, value) in self.blocks.iter() {
            if value.block.header.prev_blockhash == hash {
                block_extra.next.push(*key);
            }
        }
        if let Some(prev_block) = self.blocks.get_mut(&prev_hash) {
            prev_block.next.push(hash);
        }

        self.blocks.insert(hash, block_extra);
    }

    fn exist_and_has_two_following(&mut self, hash: &sha256d::Hash) -> Option<sha256d::Hash> {
        if let Some(block1) = self.blocks.get(hash) {
            for next1 in block1.next.iter() {
                if let Some(block2) = self.blocks.get(next1) {
                    for next2 in block2.next.iter() {
                        if self.blocks.get(next2).is_some() {
                            return Some(*next1);
                        }
                    }
                }
            }
        }
        None
    }

    fn remove(&mut self, hash: &sha256d::Hash) -> Option<BlockExtra> {
        if let Some(next) = self.exist_and_has_two_following(hash) {
            let mut value = self.blocks.remove(hash).unwrap();
            if value.next.len() > 1 {
                value.next = vec![next];
            }
            Some(value)
        } else {
            None
        }
    }

    fn print(&self) {
        for (key, value) in self.blocks.iter() {
            println!(
                "hash {} prev {} next {:?}",
                key, value.block.header.prev_blockhash, value.next
            );
        }
    }
}

impl Reorder {
    pub fn new(
        receiver: Receiver<Option<BlockExtra>>,
        sender: SyncSender<Option<BlockExtra>>,
    ) -> Reorder {
        Reorder {
            sender,
            receiver,
            height: 0,
            next: sha256d::Hash::from_hex(
                "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
            )
            .unwrap(),
            blocks: OutOfOrderBlocks::new(),
        }
    }

    fn send(&mut self, mut block_extra: BlockExtra) {
        self.next = block_extra.next[0];
        block_extra.height = self.height;
        self.sender
            .send(Some(block_extra))
            .expect("reorder: cannot send block");
        self.height += 1;
    }

    pub fn start(&mut self) {
        loop {
            let received = self.receiver.recv().expect("cannot receive blob");
            match received {
                Some(block_extra) => {
                    self.blocks.add(block_extra);
                    while let Some(mut block_to_send) = self.blocks.remove(&self.next) {
                        block_to_send.out_of_order_size = self.blocks.blocks.len();
                        self.send(block_to_send);
                    }
                }
                None => break,
            }
        }
        self.blocks.print();
        self.sender.send(None).expect("reorder cannot send none");
        println!("ending reorder");
    }
}
