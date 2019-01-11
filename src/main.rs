extern crate bitcoin;

use crate::op_return::OpReturn;
use crate::segwit::Segwit;
use crate::blocks::Blocks;
use crate::stats::Stats;
use crate::parse::TxOrBlock;
use std::io;
use std::thread;
use std::error::Error;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::channel;
use std::time::Instant;
use std::time::Duration;

mod parse;
mod op_return;
mod segwit;
mod blocks;
mod stats;

trait Startable {
    fn start(&self);
}

fn main() -> Result<(), Box<Error>> {
    let (line_sender, line_receiver) = sync_channel(1000);
    let (parsed_sender, parsed_receiver) = channel();

    let op_return = OpReturn::new();
    let stats = Stats::new();
    let segwit = Segwit::new();
    let blocks = Blocks::new();

    let line_sender_clone = line_sender.clone();
    let stdin_handle = thread::spawn( move ||  {
        let mut i = 0u64;
        loop {
            let mut buffer = String::new();
            match io::stdin().read_line(&mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        println!("Received 0 as read_line after {} lines", i);
                        break;
                    }
                    line_sender_clone.send(Some(buffer)).expect("failed to send line");
                    i += 1;
                }
                Err(error) => {
                    eprintln!("Error: {}", error);
                    break;
                }
            }
        }
        println!("ending stdin reader, {} lines read", i);
    });

    let parsed_sender_clone = parsed_sender.clone();
    let parse_handle = thread::spawn(move || {
        let mut i = 0u64;
        let mut wait_time = Duration::from_secs(0);
        loop {
            let instant = Instant::now();
            let received : Option<String> = line_receiver.recv().expect("failed to receive from tx_receiver");
            wait_time += instant.elapsed();
            match received {
                Some(value) => {
                    match parse::line(&value) {
                        Ok(result) => {
                            parsed_sender_clone.send(result).expect("failed to send tx to dispatcher");
                        },
                        Err(e) => {
                            eprintln!("parse line error {:?} ({})", e, value);
                            break;
                        },
                    };
                    i += 1;
                },
                None => break,
            }
        }
        println!("ending line parser, line parsed {}, wait time {:?}", i, wait_time);
    });

    let op_return_sender = op_return.get_sender();
    let stats_sender = stats.get_sender();
    let blocks_sender = blocks.get_sender();
    let segwit_sender = segwit.get_sender();
    let dispatcher_handle = thread::spawn( move || {
        let mut wait_time = Duration::from_secs(0);
        loop {
            let instant = Instant::now();
            let tx_or_block : TxOrBlock = parsed_receiver.recv().expect("failed to receive from tx_receiver");
            wait_time += instant.elapsed();
            op_return_sender.send(tx_or_block.clone()).expect("failed to send tx_or_block to op_return");
            stats_sender.send(tx_or_block.clone()).expect("failed to send tx_or_block to stats");
            segwit_sender.send(tx_or_block.clone()).expect("failed to send tx_or_block to segwit");
            match tx_or_block {
                TxOrBlock::Block(block) => blocks_sender.send(Some(block)).expect("failed to send block to blocks"),
                TxOrBlock::End => {
                    blocks_sender.send(None).expect("failed to send block to blocks");
                    break;
                },
                _ => continue,
            }
        }
        println!("ending dispatcher wait time {:?}", wait_time);
    });

    let mut startable : Vec<Box<Startable + Send>> = vec![Box::new(op_return),
                                                          Box::new(stats),
                                                          Box::new(segwit),
                                                          Box::new(blocks)];
    let mut processer = vec![];
    while let Some(el) = startable.pop() {
        let handle = thread::spawn(move|| {
            el.start();
        });
        processer.push(handle);
    }

    stdin_handle.join().expect("stdin_handle failed to join");
    println!("stdin_handle joined");
    line_sender.send(None).expect("error sending None on line_sender");

    parse_handle.join().expect("parse_handle failed to join");
    println!("parse_handle joined");
    parsed_sender.send(TxOrBlock::End).expect("error sending End on parse_handle");

    dispatcher_handle.join().expect("dispatcher failed to join");
    println!("dispatcher_handle joined");

    while let Some(handle) = processer.pop() {
        println!("processer {:?} joining", handle);
        handle.join().expect("processer failed to join");
    }

    println!("end");
    Ok(())
}

/*
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
*/