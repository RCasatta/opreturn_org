extern crate bitcoin;

use crate::op_return::OpReturn;
use crate::segwit::Segwit;
use crate::blocks::Blocks;
use crate::stats::Stats;
use std::io;
use std::thread;
use bitcoin::{BlockHeader, Transaction};
use std::error::Error;
use std::sync::mpsc::Sender;
use std::sync::mpsc::sync_channel;
use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use bitcoin::util::hash::BitcoinHash;
use bitcoin::OutPoint;
use bitcoin::util::hash::Sha256dHash;

mod parse;
mod op_return;
mod segwit;
mod blocks;
mod stats;

pub trait Start {
    fn start(&self);
    fn get_sender(&self) -> Sender<Option<Parsed>>;
}

#[derive(Debug, Clone)]
pub struct Parsed {
    height : u32,
    size : u32,
    tx_count : u32,
    block_header: BlockHeader,
    tx: Transaction,
}

fn main() -> Result<(), Box<Error>> {

    let mut vec : Vec<Box<Start + Send>> = vec![/*Box::new(Segwit::new()),*/
                                                Box::new(Blocks::new()),
                                                Box::new(Stats::new()),
                                                Box::new(OpReturn::new())];

    let vec_senders : Vec<Sender<Option<Parsed>>> = vec.iter().map(|el| el.get_sender()).collect();

    let mut line_senders = vec![];
    let mut line_parsers = vec![];
    let mut processer = vec![];

    let amounts : Arc<Mutex<HashMap<Sha256dHash, Vec<u64>>>> = Arc::new(Mutex::new(HashMap::new()));

    let parsers = 4;
    for i in 0..parsers {
        let (line_sender, line_receiver) = sync_channel(1000);
        line_senders.push(line_sender);
        let vec_senders = vec_senders.clone();
        let amounts = amounts.clone();
        let handle = thread::spawn(move || {
            loop {
                let received = line_receiver.recv().expect("failed to receive from line_receiver");
                match received {
                    Some(value) => {
                        //println!("{}", value);
                        let result = parse::line(value).expect("failed to parse line");
                        //println!("{:?}", result)
                        for el in vec_senders.iter() {
                            el.send(Some(result.clone())).expect("failed to send parsed");
                        }

                        let txid = result.tx.txid();
                        let values = result.tx.output.iter().map(|o| o.value).collect();
                        amounts.lock().unwrap().insert(txid, values);
                    },
                    None => break,
                }
            }
            println!("ending line parser {}",i);
        });
        line_parsers.push(handle);
    }

    while let Some(el) = vec.pop() {
        let handle = thread::spawn(move|| {
            el.start();
        });
        processer.push(handle);
    }

    let mut i = 0usize;
    loop {
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    println!("Received 0 as read_line after {} lines", i);
                    break;
                }
                line_senders[i % parsers].send(Some(buffer)).expect("failed to send line");
                i=i+1;
            }
            Err(error) => {
                println!("Error: {}", error);
                break;
            }
        }
    }

    for i in 0..parsers {
        println!("sending None to line_senders[{}]", i);
        line_senders[i].send(None).expect("failed to send to line_sender");
    }

    while let Some(handle) = line_parsers.pop() {
        handle.join().expect("line_parser failed to join");
    }

    for (i,el) in vec_senders.iter().enumerate() {
        println!("sending None to parsed_senders[{}]",i);
        el.send(None).expect("failed to send to parsed");
    }

    while let Some(handle) = processer.pop() {
        handle.join().expect("processer failed to join");
    }

    println!("{:?}",amounts.lock().unwrap().get(&Sha256dHash::default()));

    Ok(())
}


fn align (map1 : &mut HashMap<String,u32>, map2 : &mut HashMap<String,u32>) {
    for key in map1.keys() {
        if let None = map2.get(key) {
            map2.insert(key.to_owned(),0);
        }
    }

    for key in map2.keys() {
        if let None = map1.get(key) {
            map1.insert(key.to_owned(),0);
        }
    }
}
