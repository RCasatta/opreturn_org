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
use crate::BlockAndSize;

pub struct Fee {
    sender : SyncSender<Option<BlockAndSize>>,
    receiver : Receiver<Option<BlockAndSize>>,
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
    pub fn get_sender(&self) -> SyncSender<Option<BlockAndSize>> {
        self.sender.clone()
    }
}

pub struct BlockSizeFees {
    block: Block,
    size: u32,
    fees: Vec<u64>,
}

impl Startable for Fee {
    fn start(&self) {
        println!("starting fee processer");

        let mut total_txs = 0u64;
        loop {
            let received = self.receiver.recv().expect("cannot get segwit");
            match received {
                Some(block_and_size) => {
                    let (block,size) = (block_and_size.0, block_and_size.1);
                    total_txs += block.txdata.len() as u64;
                    let block_fees_bytes = self.db.get(&block_fees_key(block.bitcoin_hash())).expect("operational problem encountered");
                    let block_fees : Vec<VarInt> = match block_fees_bytes {
                        Some(block_fees_bytes) => {
                            deserialize(&block_fees_bytes).unwrap()
                        },
                        None => self.compute_block_fees(&block),
                    };
                    let b = BlockSizeFees {
                        block,
                        size,
                        fees: block_fees.iter().map(|el| el.0).collect(),
                    };
                    println!("block size: {}, block txs: {}, block fees: {}", b.size, b.block.txdata.len(), b.fees.iter().sum::<u64>());

                },
                None => break,
            }
        }
        println!("ending fee processer total tx {}", total_txs);
    }

}

impl Fee {
    fn compute_block_fees(&self, block : &Block) -> Vec<VarInt> {

        // saving all outputs value in the block in write batch
        let mut batch = WriteBatch::default();
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for (i,output) in tx.output.iter().enumerate()  {
                let key = output_key(txid, i as u64);
                let value = serialize(&VarInt(output.value));
                batch.put(&key[..], &value).expect("can't put value in batch");
            }
        }
        self.db.write(batch).expect("error writing batch writes");

        // getting all inputs value in the block
        let mut keys = vec![];
        for tx in block.txdata.iter() {
            for input in tx.input.iter() {
                let key = output_key(input.previous_output.txid, input.previous_output.vout as u64);
                keys.push(key);
            }
        }
        keys.sort();


        vec![]
    }
}



fn output_key(txid : Sha256dHash, i : u64) -> Vec<u8> {
    let mut v = vec![];
    v.push('o' as u8);
    v.extend(serialize(&txid.into_hash64()) );
    v.extend(serialize(&VarInt(i)) );
    v
}

fn tx_fee_key(txid : Sha256dHash) -> Vec<u8> {
    let mut v = vec![];
    v.push('f' as u8);
    v.extend(serialize(&txid.into_hash64()) );
    v
}

fn block_fees_key(hash : Sha256dHash) -> Vec<u8> {
    let mut v = vec![];
    v.push('b' as u8);
    v.extend(serialize(&hash.into_hash64()) );
    v
}