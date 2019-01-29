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
use bitcoin::BitcoinHash;
use bitcoin::Block;

pub struct Fee {
    sender : SyncSender<Option<Block>>,
    receiver : Receiver<Option<Block>>,
    db : DB,
}

impl Fee {
    pub fn new(db : DB) -> Fee {
        let (sender, receiver) = sync_channel(1000);
        Fee {
            sender,
            receiver,
            db,
        }
    }
    pub fn get_sender(&self) -> SyncSender<Option<Block>> {
        self.sender.clone()
    }
}

impl Startable for Fee {
    fn start(&self) {
        println!("starting fee processer");

        let mut total_tx = 0u64;
        loop {
            let received = self.receiver.recv().expect("cannot get segwit");
            match received {
                Some(block) => total_tx += block.txdata.len() as u64,
                None => break,
            }
        }
        println!("ending fee processer total tx {}", total_tx);
    }

}