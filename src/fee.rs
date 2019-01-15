use crate::parse::TxOrBlock;
use crate::parse::BlockParsed;
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

pub struct Fee {
    sender : SyncSender<TxOrBlock>,
    receiver : Receiver<TxOrBlock>,
}

impl Fee {
    pub fn new() -> Fee {
        let (sender, receiver) = sync_channel(1000);
        Fee {
            sender,
            receiver,
        }
    }
    pub fn get_sender(&self) -> SyncSender<TxOrBlock> {
        self.sender.clone()
    }
}

impl Startable for Fee {
    fn start(&self) {
        println!("starting fee processer");
        let db = DB::open_default("db").unwrap();

        let mut wait_time =  Duration::from_secs(0);
        let mut block_fee = 0u64;
        let mut current_block :Option<BlockParsed>= None;
        loop {
            let instant = Instant::now();
            let received = self.receiver.recv().expect("cannot get segwit");
            wait_time += instant.elapsed();
            match received {
                TxOrBlock::Block(block) => {
                    match current_block {
                        Some(current_block) => {
                            println!("block {} fee {}", current_block.height, block_fee);
                            db.put(&block_fee_key(current_block.block_header.bitcoin_hash()), &serialize(&VarInt(block_fee)) ).expect("put error");
                        },
                        None => (),
                    }
                    current_block = Some(block);
                    block_fee = 0u64;
                },
                TxOrBlock::Tx(tx) => {
                    let tx = tx.tx;
                    let txid = tx.txid();
                    let tx_fee_key = tx_fee_key(txid);
                    match db.get(&tx_fee_key).expect("operational problem encountered") {
                        Some(value) => {
                            let value : VarInt = deserialize(&value).expect("error while deserializing varing");
                            block_fee += value.0;
                            continue;
                        },
                        None => (),
                    }
                    let mut output_sum = 0u64;
                    let mut batch = WriteBatch::default();
                    for (i,output) in tx.output.iter().enumerate()  {
                        let key = output_key(txid, i as u64);
                        let value = serialize(&VarInt(output.value));
                        batch.put(&key[..], &value).expect("can't put value in batch");
                        output_sum += output.value;
                    }
                    db.write(batch).expect("error writing batch writes");
                    if tx.is_coin_base() {
                        continue;
                    }
                    let mut keys = vec![];
                    for input in tx.input {
                        let key = output_key(input.previous_output.txid, input.previous_output.vout as u64);
                        keys.push(key);
                    }
                    keys.sort();
                    let mut input_sum = 0u64;
                    for key in keys {
                        match db.get(&key).expect("operational problem encountered") {
                            Some(value) => {
                                let value : VarInt = deserialize(&value).expect("error while deserializing varing");
                                input_sum += value.0;
                            },
                            None => println!("value not found for key {}", hex::encode(&key)),
                        }
                    }
                    if input_sum > output_sum {
                        let fee = input_sum - output_sum;
                        db.put(&tx_fee_key, &serialize(&VarInt(fee))).expect("can't write fee");
                        block_fee += fee;
                    }
                },
                TxOrBlock::End => {
                    println!("fee: received {:?}", received);
                    break;
                },
            }
        }
        println!("ending fee processer, wait time: {:?}", wait_time );
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

fn block_fee_key(hash : Sha256dHash) -> Vec<u8> {
    let mut v = vec![];
    v.push('b' as u8);
    v.extend(serialize(&hash.into_hash64()) );
    v
}