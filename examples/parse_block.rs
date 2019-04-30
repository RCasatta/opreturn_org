extern crate bitcoin;

use std::io;
use std::io::{Read, Write};
use bitcoin::{Block, OutPoint, Transaction, Script};
use bitcoin::consensus::{deserialize, serialize};
use bitcoin::util::hash::BitcoinHash;
use std::collections::{HashSet, HashMap};
use bitcoin_hashes::sha256d;

fn main() {
    let mut buffer = [0u8; 200_000];
    let mut all: Vec<u8> = vec![];
    loop {
        let bytes_read = io::stdin().read(&mut buffer).unwrap();
        eprintln!("bytes read {}", bytes_read);
        if bytes_read == 0 {
            break;
        }
        all.extend(&buffer[0..bytes_read]);
    }
    eprintln!("all size {}", all.len());
    let mut block: Block = deserialize(&all).unwrap();
    eprintln!("block hash {:?}", block.header.bitcoin_hash());

    let txs: HashMap<sha256d::Hash, Transaction> = block.txdata.iter().map(|tx| (tx.txid(), tx.clone()) ).collect();
    let mut scripts: HashSet<Script> = HashSet::new();
    let mut counter = 0usize;
    eprintln!("block tx hashes {:?}", txs.len());
    for tx in block.txdata.iter_mut() {
        let hash = tx.txid();
        for input in tx.input.iter_mut() {
            if let Some(previous_tx) = txs.get(&input.previous_output.txid) {
                eprintln!("tx {} has prevout hash {} in current block ", hash, &input.previous_output.txid);
                counter += 1;
                input.previous_output.txid = Default::default();
                eprintln!("script_sig {:?}",input.script_sig );
                let previous_output = previous_tx.output.get(input.previous_output.vout as usize).unwrap();
                eprintln!("script_pubkey {:?}",previous_output.script_pubkey);
                scripts.insert(previous_output.script_pubkey.clone());
                eprintln!("script_pubkey_hex {:?}", hex::encode(previous_output.script_pubkey.as_bytes() ));
            }
        }
    }
    let mut count_p2pkh = 0;
    for tx in block.txdata.iter_mut() {
        for output in tx.output.iter_mut() {
            if scripts.contains( &output.script_pubkey) {
                if output.script_pubkey.is_p2pkh() {
                    count_p2pkh += 1;
                    let mut new_script = vec![];
                    new_script.extend_from_slice(output.script_pubkey.as_bytes());
                    for i in 3..23 {
                        new_script[i] = 0;
                    }
                    eprintln!("rep script_pubkey_hex {:?} {:?}", hex::encode(output.script_pubkey.as_bytes() ), hex::encode(&new_script ));
                    output.script_pubkey = Script::from( new_script);
                }
            }
        }
    }

    eprintln!("block: {} txs: {} total: {} p2pkh: {}", block.header.bitcoin_hash(), txs.len(), counter, count_p2pkh);

    let result = serialize(&block);
    io::stdout().write(&result);
}
