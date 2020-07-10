extern crate bitcoin;

use bitcoin::consensus::{deserialize, serialize};
use bitcoin::hashes::sha256d;
use bitcoin::util::hash::BitcoinHash;
use bitcoin::{Block, Script, Transaction, VarInt};
use std::collections::{HashMap, HashSet};
use std::io;
use std::io::{Read, Write};

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

    let txs: HashMap<sha256d::Hash, (usize, Transaction)> = block
        .txdata
        .iter()
        .enumerate()
        .map(|(i, tx)| (tx.txid(), (i, tx.clone())))
        .collect();
    let inputs_per_tx: Vec<usize> = block.txdata.iter().map(|tx| tx.input.len()).collect();
    let mut scripts: HashSet<Script> = HashSet::new();
    let mut counter = 0usize;
    let mut distances = vec![];
    eprintln!("block tx hashes {:?}", txs.len());
    for (j, tx) in block.txdata.iter_mut().enumerate() {
        let hash = tx.txid();
        for (k, input) in tx.input.iter_mut().enumerate() {
            if let Some((i, previous_tx)) = txs.get(&input.previous_output.txid) {
                let inputs_distance: usize = inputs_per_tx[*i + 1..j].iter().sum::<usize>() + k;
                distances.push(VarInt(inputs_distance as u64));
                eprintln!(
                    "tx {} #{} has prevout hash {} #{} in current block, inputs_distance: {}",
                    hash, j, &input.previous_output.txid, i, inputs_distance
                );
                assert!(j > *i);
                counter += 1;
                input.previous_output.txid = Default::default();
                eprintln!("script_sig {:?}", input.script_sig);
                let previous_output = previous_tx
                    .output
                    .get(input.previous_output.vout as usize)
                    .unwrap();
                eprintln!("script_pubkey {:?}", previous_output.script_pubkey);
                scripts.insert(previous_output.script_pubkey.clone());
                eprintln!(
                    "script_pubkey_hex {:?}",
                    hex::encode(previous_output.script_pubkey.as_bytes())
                );
            }
        }
    }
    let mut count_p2pkh = 0;
    for tx in block.txdata.iter_mut() {
        for output in tx.output.iter_mut() {
            if scripts.contains(&output.script_pubkey) {
                if output.script_pubkey.is_p2pkh() {
                    count_p2pkh += 1;
                    let mut new_script = vec![];
                    new_script.extend_from_slice(output.script_pubkey.as_bytes());
                    for i in 3..23 {
                        new_script[i] = 0;
                    }
                    eprintln!(
                        "rep script_pubkey_hex {:?} {:?}",
                        hex::encode(output.script_pubkey.as_bytes()),
                        hex::encode(&new_script)
                    );
                    output.script_pubkey = Script::from(new_script);
                }
            }
        }
    }

    eprintln!("{:?}", inputs_per_tx);

    eprintln!(
        "block: {} txs: {} total: {}",
        block.header.bitcoin_hash(),
        txs.len(),
        counter
    );

    let result = serialize(&block);
    io::stdout().write(&result).expect("cannot print to stdout");
    let result = serialize(&distances);
    io::stdout().write(&result).expect("cannot print to stdout");
}
