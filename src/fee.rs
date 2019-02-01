use std::time::Instant;
use std::time::Duration;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::collections::HashMap;
use rocksdb::DB;
use rocksdb::WriteBatch;
use rocksdb::WriteOptions;
use bitcoin::util::hash::Sha256dHash;
use bitcoin::consensus::serialize;
use bitcoin::consensus::deserialize;
use bitcoin::VarInt;
use bitcoin::Block;
use bitcoin::BitcoinHash;
use bitcoin::OutPoint;
use bitcoin::Transaction;
use crate::Startable;
use crate::parse::BlockSize;
use crate::reorder::BlockSizeHeight;

pub struct BlockSizeHeightValues {
    pub block : Block,
    pub size: u32,
    pub height: u32,
    pub outpoint_values: HashMap<OutPoint,u64>,
}

pub struct Fee {
    receiver : Receiver<Option<BlockSizeHeight>>,
    sender : SyncSender<Option<BlockSizeHeightValues>>,
    db : DB,
}

impl Fee {
    pub fn new(receiver : Receiver<Option<BlockSizeHeight>>,  sender : SyncSender<Option<BlockSizeHeightValues>>, db : DB) -> Fee {
        Fee {
            sender,
            receiver,
            db,
        }
    }
}

impl Startable for Fee {
    fn start(&self) {
        println!("starting fee processer");

        let mut total_txs = 0u64;
        let mut found_values = 0u32;
        loop {
            let received = self.receiver.recv().expect("cannot get segwit");
            match received {
                Some(block_size_height) => {
                    let (block,size,height) = (block_size_height.block, block_size_height.size, block_size_height.height);
                    total_txs += block.txdata.len() as u64;
                    let outpoint_values_bytes = self.db.get(&block_outpoint_values_key(block.bitcoin_hash())).expect("operational problem encountered");
                    let mut outpoint_values_vec = match outpoint_values_bytes {
                        Some(block_outpoint_values_bytes) => {
                            found_values += 1;
                            deserialize(&block_outpoint_values_bytes).expect("cannot deserialize block fees")
                        },
                        None => self.compute_outpoint_values(&block),
                    };
                    let mut outpoint_values = HashMap::new();
                    for tx in block.txdata.iter() {
                        for input in tx.input.iter() {
                            outpoint_values.insert(input.previous_output, outpoint_values_vec.pop().expect("can't pop").0);
                        }
                    }
                    let b = BlockSizeHeightValues {
                        block,
                        size,
                        height,
                        outpoint_values,
                    };
                    println!("#{:>6} {} size:{:>6}, txs:{:>4} fee:{:>9} found:{:>6}",
                             b.height,
                             b.block.bitcoin_hash(),
                             b.size,
                             b.block.txdata.len(), block_fee(&b),
                             found_values
                    );
                },
                None => break,
            }
        }
        println!("ending fee processer total tx {}, output values found: {}", total_txs, found_values);
    }

}

impl Fee {
    fn compute_outpoint_values(&self, block : &Block) -> Vec<VarInt> {

        // saving all outputs value in the block in write batch
        let mut batch = WriteBatch::default();
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for (i,output) in tx.output.iter().enumerate()  {
                let key = output_key(txid, i as u64);
                let value = serialize(&VarInt(output.value));
                batch.put(&key[..], &value).expect("can't put value in batch");
                //println!("putting {:?} hex {}", txid, hex::encode(key));
            }
        }
        self.db.write(batch).expect("error writing batch writes");

        // getting all inputs keys and outpoints, prepare for deletion
        let mut keys_outpoint = vec![];
        let mut index = 1u32;
        for tx in block.txdata.iter().skip(1) {
            for input in tx.input.iter() {
                let key = output_key(input.previous_output.txid, input.previous_output.vout as u64);
                keys_outpoint.push((key,index));
                index += 1;
            }
        }

        // getting value from db in ordered fashion (because of paging is faster)
        keys_outpoint.sort_by(|a, b| a.0.cmp(&b.0));
        let mut values = vec![];
        values.push( (0, VarInt( block.txdata[0].output.iter().map(|el| el.value).sum() ) ) );  //coinbase
        for (key, index) in keys_outpoint {
            let value = self.db.get(&key).expect("operational problem in get").unwrap();
            values.push( (index, deserialize::<VarInt>( &value ).unwrap() ) );
        }

        // reordering in block order (reversed)
        values.sort_by(|a,b| a.0.cmp(&b.0));
        let mut values : Vec<VarInt> = values.iter().map(|el| el.1.clone() ).collect();
        values.reverse();

        self.db.put(&block_outpoint_values_key(block.bitcoin_hash()), &serialize(&values));

        values
    }
}

pub fn block_fee(block_value: &BlockSizeHeightValues) -> u64 {
    let mut total = 0u64;
    for tx in block_value.block.txdata.iter() {
        total += tx_fee(tx, &block_value.outpoint_values);
    }
    total
}

pub fn tx_fee(tx : &Transaction, outpoint_values : &HashMap<OutPoint, u64>) -> u64 {
    let output_total : u64 = tx.output.iter().map(|el| el.value).sum();
    let mut input_total = 0u64;
    for input in tx.input.iter() {
        match outpoint_values.get(&input.previous_output) {
            Some(val) => input_total += val,
            None => panic!("can't find tx fee {}", tx.txid()),
        }
    }
    input_total - output_total
}

fn output_key(txid : Sha256dHash, i : u64) -> Vec<u8> {
    let mut v = vec![];
    v.push('o' as u8);
    v.extend(serialize(&txid.into_hash64()));
    v.extend(serialize(&VarInt(i)) );
    v
}

fn block_outpoint_values_key(hash : Sha256dHash) -> Vec<u8> {
    let mut v = vec![];
    v.push('v' as u8);
    v.extend(serialize(&hash));
    v
}