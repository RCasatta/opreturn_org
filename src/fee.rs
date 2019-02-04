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
    delete_after: HashMap<u32, Vec<Vec<u8>>>,
}

impl Fee {
    pub fn new(receiver : Receiver<Option<BlockSizeHeight>>,  sender : SyncSender<Option<BlockSizeHeightValues>>, db : DB) -> Fee {
        Fee {
            sender,
            receiver,
            db,
            delete_after: HashMap::new(),
        }
    }
}

impl Fee {
    pub fn start(&mut self) {
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
                        None => self.compute_outpoint_values(&block, height),
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
                    println!("#{:>6} {} size:{:>7} txs:{:>4} total_txs:{:>9} fee:{:>9} found:{:>6} reorder_cache:{:>4}",
                             b.height,
                             b.block.bitcoin_hash(),
                             b.size,
                             b.block.txdata.len(),
                             total_txs,
                             block_fee(&b),
                             found_values,
                             block_size_height.out_of_order_size,
                    );
                    self.sender.send(Some(b)).expect("fee: cannot send");
                },
                None => break,
            }
        }
        self.sender.send(None).expect("fee: cannot send none");
        println!("ending fee processer total tx {}, output values found: {}", total_txs, found_values);
    }

}

impl Fee {
    fn compute_outpoint_values(&mut self, block : &Block, height : u32) -> Vec<VarInt> {

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
        let mut keys_index = vec![];
        let mut index = 1u32;
        for tx in block.txdata.iter().skip(1) {
            for input in tx.input.iter() {
                let key = output_key(input.previous_output.txid, input.previous_output.vout as u64);
                keys_index.push((key,index));
                index += 1;
            }
        }

        // getting value from db in ordered fashion (because of paging is faster)
        keys_index.sort_by(|a, b| a.0.cmp(&b.0));
        let mut values_index = vec![];
        let coin_base_output_value = block.txdata[0].output.iter().map(|el| el.value).sum();
        values_index.push( ( VarInt(coin_base_output_value), 0 ) );  //coinbase
        for (key, index) in keys_index.iter() {
            let value = self.db.get(key).expect("operational problem in get").expect("unexpected None in db");
            values_index.push( (deserialize::<VarInt>( &value ).unwrap(), *index ) );
        }
        let to_delete = keys_index.into_iter().map(|el| el.0).collect();
        self.delete_after(height,to_delete);

        // reordering in block order (reversed)
        values_index.sort_by(|a,b| b.1.cmp(&a.1));
        let values : Vec<VarInt> = values_index.into_iter().map(|el| el.0 ).collect();

        self.db.put(&block_outpoint_values_key(block.bitcoin_hash()),
                    &serialize(&values)).expect("fee: cannot put value in db");

        values
    }

    fn delete_after(&mut self, height : u32, to_delete : Vec<Vec<u8>>) {
        self.delete_after.insert(height, to_delete);
        if height>6 {
            if let Some(val) = self.delete_after.remove(&(height - 6) ) {
                let mut batch = WriteBatch::default();
                for el in val {
                    batch.delete(&el).expect("cannot insert deletion in batch");
                }
                let mut opt = WriteOptions::default();
                opt.set_sync(false);
                opt.disable_wal(true);
                self.db.write_opt(batch, &opt).expect("cannot delete batch");
            }
        }
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