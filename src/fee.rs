use std::time::Instant;
use std::time::Duration;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::collections::HashMap;
use rocksdb::DB;
use rocksdb::WriteBatch;
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
                    println!("#{} {} prev {} size: {}, txs: {} fee:{:?} found:{}",
                             b.height,
                             b.block.bitcoin_hash(),
                             b.block.header.prev_blockhash,
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
        let mut batch_delete = WriteBatch::default();
        for tx in block.txdata.iter() {
            if tx.is_coin_base() {
                continue;
            }
            for input in tx.input.iter() {
                let key = output_key(input.previous_output.txid, input.previous_output.vout as u64);
                batch_delete.delete(&key);
                keys_outpoint.push((key,input.previous_output.clone()));

            }
        }

        // getting value from db in ordered fashion (because of paging is faster)
        keys_outpoint.sort_by(|a, b| a.0.cmp(&b.0));
        let mut map = HashMap::new();
        for (key, outpoint) in keys_outpoint {
            match self.db.get(&key).expect("operational problem in get") {
                Some(val) => {
                    map.insert(outpoint, deserialize(&val).expect("err in deser val"));
                    ()
                } ,
                None => println!("value not found for key {} outpoint: {:?}", hex::encode(key), outpoint ),
            };
        }

        // creating array of prevout values
        let mut values : Vec<VarInt> = vec![];
        for tx in block.txdata.iter() {
            if tx.is_coin_base() {
                values.push(VarInt( tx.output.iter().map(|el| el.value).sum() ) );
            } else {
                for input in tx.input.iter() {
                    values.push(map.remove(&input.previous_output).expect("value not found"));
                }
            }

        }
        values.reverse();  // we will use them in reverse order

        self.db.put(&block_outpoint_values_key(block.bitcoin_hash()), &serialize(&values));
        self.db.write(batch_delete);  // since every output could be spent exactly once, we can remove it from db (we can rebuild the db in bad cases like reorgs)

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