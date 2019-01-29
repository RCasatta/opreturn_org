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
    sender : SyncSender<Option<Vec<Block>>>,
    receiver : Receiver<Option<Vec<Block>>>,
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
    pub fn get_sender(&self) -> SyncSender<Option<Vec<Block>>> {
        self.sender.clone()
    }
}

impl Startable for Fee {
    fn start(&self) {
        println!("starting fee processer");

        loop {
            let received = self.receiver.recv().expect("cannot get segwit");
            match received {
                Some(blocks) => println!("fee received {}", blocks.len()),
                None => break,
            }
        }
        println!("ending fee processer" );
    }

}