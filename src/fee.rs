use crate::BlockExtra;
use bitcoin::consensus::{deserialize, serialize};
use bitcoin::{BitcoinHash, Block, OutPoint, Script, Transaction, TxOut, VarInt};
use bitcoin_hashes::sha256d;
use bitcoin_hashes::Hash;
use rocksdb::WriteBatch;
use rocksdb::WriteOptions;
use rocksdb::DB;
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::sync::Arc;
use std::time::Instant;

pub struct Fee {
    receiver: Receiver<Option<BlockExtra>>,
    sender: SyncSender<Option<BlockExtra>>,
    db: Arc<DB>,
    delete_after: HashMap<u32, Vec<Vec<u8>>>,
}

impl Fee {
    pub fn new(
        receiver: Receiver<Option<BlockExtra>>,
        sender: SyncSender<Option<BlockExtra>>,
        db: Arc<DB>,
    ) -> Fee {
        Fee {
            sender,
            receiver,
            db,
            delete_after: HashMap::new(),
        }
    }

    pub fn start(&mut self) {
        println!("starting fee processer");
        let mut busy_time = 0u128;
        let mut total_txs = 0u64;
        let mut found_values = 0u32;
        loop {
            let received = self.receiver.recv().expect("cannot get segwit");
            let now = Instant::now();
            match received {
                Some(mut block_extra) => {
                    total_txs += block_extra.block.txdata.len() as u64;
                    let outpoint_values_bytes = self
                        .db
                        .get(&block_outpoint_values_key(block_extra.block.bitcoin_hash()))
                        .expect("operational problem encountered");
                    let mut outpoint_values_vec = match outpoint_values_bytes {
                        Some(block_outpoint_values_bytes) => {
                            found_values += 1;
                            deserialize(&block_outpoint_values_bytes)
                                .expect("cannot deserialize block fees")
                        }
                        None => {
                            self.compute_previous_outpoint(&block_extra.block, block_extra.height)
                        }
                    };
                    for tx in block_extra.block.txdata.iter() {
                        for input in tx.input.iter() {
                            block_extra.outpoint_values.insert(
                                input.previous_output,
                                outpoint_values_vec.pop().expect("can't pop"),
                            );
                        }
                    }
                    println!("#{:>6} {} size:{:>7} txs:{:>4} total_txs:{:>9} fee:{:>9} found:{:>6} ooo_size:{:>4}" ,
                             block_extra.height,
                             block_extra.block.bitcoin_hash(),
                             block_extra.size,
                             block_extra.block.txdata.len(),
                             total_txs,
                             block_fee(&block_extra),
                             found_values,
                             block_extra.out_of_order_size,
                    );
                    busy_time = busy_time + now.elapsed().as_nanos();
                    self.sender
                        .send(Some(block_extra))
                        .expect("fee: cannot send");
                }
                None => break,
            }
        }
        self.sender.send(None).expect("fee: cannot send none");
        println!(
            "ending fee processer total tx {}, output values found: {}, busy time: {}s",
            total_txs,
            found_values,
            busy_time / 1_000_000_000
        );
    }

    fn compute_previous_outpoint(&mut self, block: &Block, height: u32) -> Vec<TxOut> {
        // saving all outputs value in the block in write batch
        let mut batch = WriteBatch::default();
        for tx in block.txdata.iter() {
            let txid = tx.txid();
            for (i, output) in tx.output.iter().enumerate() {
                let key = output_key(txid, i as u32);
                let value = serialize(output);
                batch
                    .put(&key[..], &value)
                    .expect("can't put value in batch");
                //println!("putting {:?} hex {}", txid, hex::encode(key));
            }
        }
        self.db.write(batch).expect("error writing batch writes");

        // getting all inputs keys and outpoints, prepare for deletion
        let mut keys = vec![];
        for tx in block.txdata.iter().skip(1) {
            for input in tx.input.iter() {
                keys.push(output_key(
                    input.previous_output.txid,
                    input.previous_output.vout,
                ));
            }
        }

        let coin_base_output_value = block.txdata[0].output.iter().map(|el| el.value).sum();
        let mut values = vec![];
        values.push(TxOut {
            value: coin_base_output_value,
            script_pubkey: Script::new(),
        }); //coinbase
        for key in keys.iter().rev() {
            let value = self
                .db
                .get(key)
                .expect("operational problem in get")
                .unwrap_or_else(|| panic!("unexpected None in db for key {}", hex::encode(key)));
            values.push(deserialize::<TxOut>(&value).unwrap());
        }

        self.db
            .put(
                &block_outpoint_values_key(block.bitcoin_hash()),
                &serialize(&values),
            )
            .expect("fee: cannot put value in db");

        self.delete_after(height, keys);

        values
    }

    fn delete_after(&mut self, height: u32, to_delete: Vec<Vec<u8>>) {
        self.delete_after.insert(height, to_delete);
        if height > 6 {
            if let Some(val) = self.delete_after.remove(&(height - 6)) {
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

pub fn block_fee(block_value: &BlockExtra) -> u64 {
    let mut total = 0u64;
    for tx in block_value.block.txdata.iter() {
        total += tx_fee(tx, &block_value.outpoint_values);
    }
    total
}

pub fn tx_fee(tx: &Transaction, outpoint_values: &HashMap<OutPoint, TxOut>) -> u64 {
    let output_total: u64 = tx.output.iter().map(|el| el.value).sum();
    let mut input_total = 0u64;
    for input in tx.input.iter() {
        match outpoint_values.get(&input.previous_output) {
            Some(txout) => input_total += txout.value,
            None => panic!("can't find tx fee {}", tx.txid()),
        }
    }
    input_total - output_total
}

fn output_key(txid: sha256d::Hash, i: u32) -> Vec<u8> {
    let mut v = vec![];
    v.push(b'o');
    v.extend(&txid.into_inner()[0..10]);
    //v.extend(serialize(&txid));
    v.extend(serialize(&VarInt(i as u64)));
    v
}

fn block_outpoint_values_key(hash: sha256d::Hash) -> Vec<u8> {
    let mut v = vec![];
    v.push(b'v');
    v.extend(serialize(&hash));
    v
}
