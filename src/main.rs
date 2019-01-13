extern crate bitcoin;

use crate::op_return::OpReturn;
use crate::blocks::Blocks;
use crate::stats::Stats;
use crate::parse::TxOrBlock;
use std::io;
use std::thread;
use std::error::Error;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

mod parse;
mod op_return;
mod blocks;
mod stats;

trait Startable {
    fn start(&self);
}

fn main() -> Result<(), Box<Error>> {
    let op_return = OpReturn::new();
    let stats = Stats::new();
    //let segwit = Segwit::new();
    let blocks = Blocks::new();

    let op_return_sender = op_return.get_sender();
    let stats_sender = stats.get_sender();
    let blocks_sender = blocks.get_sender();
    //let segwit_sender = segwit.get_sender();

    let handle = thread::spawn( move ||  {
        let mut i = 0u64;
        loop {
            let mut buffer = String::new();
            match io::stdin().read_line(&mut buffer) {
                Ok(n) => {
                    if n == 0 {
                        println!("Received 0 as read_line after {} lines", i);
                        op_return_sender.send(TxOrBlock::End).expect("error sending End on op_return");
                        stats_sender.send(TxOrBlock::End).expect("error sending End on stats_sender");
                        //segwit_sender.send(TxOrBlock::End).expect("error sending End on segwit");
                        blocks_sender.send(None).expect("error sending None to blocks_sender");

                        break;
                    }
                    match parse::line(&buffer) {
                        Ok(tx_or_block) => {
                            op_return_sender.send(tx_or_block.clone()).expect("failed to send tx_or_block to op_return");
                            stats_sender.send(tx_or_block.clone()).expect("failed to send tx_or_block to stats");
                            //segwit_sender.send(tx_or_block.clone()).expect("failed to send tx_or_block to segwit");
                            match tx_or_block {
                                TxOrBlock::Block(block) => blocks_sender.send(Some(block)).expect("failed to send block to blocks"),
                                TxOrBlock::End => {
                                    blocks_sender.send(None).expect("failed to send block to blocks");
                                    break;
                                },
                                _ => continue,
                            }
                        },
                        Err(e) => {
                            eprintln!("parse line error {:?} ({})", e, buffer);
                        },
                    };
                    i += 1;
                }
                Err(error) => {
                    eprintln!("Error: {}", error);
                }
            }
        }
        println!("ending stdin and line parser reader, {} lines read", i);
    });

    let mut startable : Vec<Box<Startable + Send>> = vec![Box::new(op_return),
                                                          Box::new(stats),
                                                          //Box::new(segwit),
                                                          Box::new(blocks)];
    let mut processer = vec![];
    while let Some(el) = startable.pop() {
        let handle = thread::spawn(move|| {
            el.start();
        });
        processer.push(handle);
    }

    handle.join().expect("parse_handle failed to join");
    println!("parse_handle joined");

    while let Some(handle) = processer.pop() {
        println!("processer {:?} joining", handle);
        handle.join().expect("processer failed to join");
    }

    println!("end");
    Ok(())
}

fn print_map_by_usize_key(map : &HashMap<usize,u32>, file_name: &str) {
    let mut file = BufWriter::new( File::create(&file_name).expect(&format!("error opening file {}", file_name)));
    let mut map_keys : Vec<_> = map.keys().collect();
    map_keys.sort();
    for key in map_keys {
        let value = map.get(key).unwrap();
        file.write(format!("{} {}\n",key,value).as_bytes()).expect("can't write");
    }
    println!("file {} written", file_name);
}

fn print_map_by_key(map : &HashMap<String,u32>, file_name: &str){
    let mut file = BufWriter::new( File::create(&file_name).expect(&format!("error opening file {}", file_name)));
    let mut map_keys : Vec<_> = map.keys().collect();
    map_keys.sort();
    for key in map_keys {
        let value = map.get(key).unwrap();
        file.write(format!("{} {}\n",key,value).as_bytes()).expect("can't write");
    }
    println!("file {} written", file_name);
}

fn print_map_by_value(map : &HashMap<String,u32>, file_name: &str) {
    let mut file = BufWriter::new(File::create(&file_name).expect(&format!("error opening file {}", file_name)));
    let mut count_vec: Vec<(&String, &u32)> = map.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (key,value) in count_vec.iter().take(10) {
        file.write(format!("{} {}\n",key,value).as_bytes()).expect("can't write");
    }
    let other = count_vec.iter().skip(10).fold(0, |acc, x| acc + x.1);
    file.write(format!("other {}\n",other).as_bytes()).expect("can't write");
    println!("file {} written", file_name);
}