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
use bitcoin::OutPoint;
use std::collections::HashMap;
use bitcoin::Transaction;
use crate::parse::BlockSize;


pub struct Fee {
    sender : SyncSender<Option<BlockSizeValues>>,
    receiver : Receiver<Option<BlockSize>>,
    db : DB,
}

impl Fee {
    pub fn new(receiver : Receiver<Option<BlockSize>>,  sender : SyncSender<Option<BlockSizeValues>>, db : DB) -> Fee {
        Fee {
            sender,
            receiver,
            db,
        }
    }
}

pub struct BlockSizeValues {
    pub block: Block,
    pub size: u32,
    pub outpoint_values: HashMap<OutPoint,u64>,
}

impl Startable for Fee {
    fn start(&self) {
        println!("starting fee processer");

        let mut total_txs = 0u64;
        loop {
            let received = self.receiver.recv().expect("cannot get segwit");
            match received {
                Some(block_and_size) => {
                    let (block,size) = (block_and_size.block, block_and_size.size);
                    total_txs += block.txdata.len() as u64;
                    let outpoint_values_bytes = self.db.get(&block_outpoint_values_key(block.bitcoin_hash())).expect("operational problem encountered");
                    let mut outpoint_values_vec = match outpoint_values_bytes {
                        Some(block_outpoint_values_bytes) => deserialize(&block_outpoint_values_bytes).expect("cannot deserialize block fees"),
                        None => self.compute_outpoint_values(&block),
                    };
                    let mut outpoint_values = HashMap::new();
                    for tx in block.txdata.iter() {
                        for input in tx.input.iter() {
                            outpoint_values.insert(input.previous_output, outpoint_values_vec.pop().expect("can't pop").0);
                        }
                    }
                    let b = BlockSizeValues {
                        block,
                        size,
                        outpoint_values,
                    };
                    println!("# {} prev {} block size: {}, block txs: {} block fee:{:?}", b.block.bitcoin_hash(), b.block.header.prev_blockhash, b.size, b.block.txdata.len(), block_fee(&b));
                },
                None => break,
            }
        }
        println!("ending fee processer total tx {}", total_txs);
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

        // getting all inputs values
        let mut values : Vec<VarInt> = vec![];
        for tx in block.txdata.iter() {
            if tx.is_coin_base() {
                values.push(VarInt( tx.output.iter().map(|el| el.value).sum() ) );
                continue;
            }
            for input in tx.input.iter() {
                let key = output_key(input.previous_output.txid, input.previous_output.vout as u64);
                match self.db.get(&key).expect("operational problem in get") {
                    Some(val) => {
                        values.push(deserialize(&val).expect("err in deser val"))
                    },
                    None => {
                        //println!("tx {} value not found for prevout {:?} : {} hex {}", tx.txid(), input.previous_output.txid.be_hex_string(), input.previous_output.vout, hex::encode(key));
                        values.push(VarInt(0));
                    },
                }
            }
        }
        values.reverse();

        self.db.put(&block_outpoint_values_key(block.bitcoin_hash()), &serialize(&values));

        values
    }
}

fn block_fee(block_value: &BlockSizeValues) -> Option<u64> {
    let mut total = 0u64;
    for tx in block_value.block.txdata.iter() {
        match tx_fee(tx, &block_value.outpoint_values) {
            Some(val) => {
                total += val;
                //println!("txfee {} {}", tx.txid(), val);
            },
            None => return None,
        }
    }
    Some(total)
}

fn tx_fee(tx : &Transaction, outpoint_values : &HashMap<OutPoint, u64>) -> Option<u64> {
    let output_total : u64 = tx.output.iter().map(|el| el.value).sum();
    let mut input_total = 0u64;
    for input in tx.input.iter() {
        match outpoint_values.get(&input.previous_output) {
            Some(val) => input_total += val,
            None => return None,
        }
    }
    Some(input_total - output_total)
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