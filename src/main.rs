extern crate bitcoin;
extern crate rocksdb;

//use crate::fee::Fee;
use std::path::PathBuf;
use std::fs;
use bitcoin::consensus::encode::Decodable;
use bitcoin::consensus::deserialize;
use bitcoin::Block;
use std::env;
use std::io::Cursor;
use std::io::SeekFrom;
use std::io::Seek;
use bitcoin::network::constants::Network;
use std::sync::mpsc::sync_channel;
use std::thread;
use std::sync::Mutex;
use std::sync::Arc;
use rocksdb::DB;
use crate::read::Read;
use crate::parse::Parse;
use crate::fee::Fee;

mod fee;
mod parse;
mod read;

trait Startable {
    fn start(&self);
}

fn main() {
    let mut path = PathBuf::from(env::var("BITCOIN_DIR").unwrap_or("~/.bitcoin/".to_string()));
    let blob_size = env::var("BLOB_CHANNEL_SIZE").unwrap_or("1".to_string()).parse::<usize>().unwrap_or(1);
    let blocks_size = env::var("BLOCKS_CHANNEL_SIZE").unwrap_or("1000".to_string()).parse::<usize>().unwrap_or(1000);
    let db = DB::open_default(env::var("DB").unwrap_or("db".to_string())).unwrap();

    let (send_blobs, receive_blobs) = sync_channel(blob_size);
    let mut read = Read::new(path, send_blobs);
    let read_handle = thread::spawn( move || { read.start(); });

    let (send_blocks, receive_blocks) = sync_channel(blocks_size);
    let mut parse = Parse::new(receive_blobs, send_blocks);
    let parse_handle = thread::spawn( move || { parse.start(); });

    let (send_blocks_and_fee, receive_blocks_and_fee) = sync_channel(blocks_size);
    let mut fee = Fee::new(receive_blocks, send_blocks_and_fee, db);
    let fee_handle = thread::spawn( move || { fee.start(); });

    /*
    let process = Process::new(receive_blocks_and_fee);
    let process_handle = thread::spawn( move || { process.start(); });
*/
    read_handle.join().unwrap();
    parse_handle.join().unwrap();
    /*
    let fee = Fee::new(db);
    let fee_sender = fee.get_sender();
    let fee_handle = thread::spawn( move || {
        fee.start();
    });

    let (send_blocks, receive_blocks) = sync_channel(channel_size);


    let block_counter_clone = block_counter.clone();
    let receive_clone = receive_blocks.clone();
    let fee_sender_clone = fee_sender.clone();
    let handle = thread::spawn( move || {
        loop {
            let result = receive_clone.lock().unwrap().recv();
            match result {
                Ok(blob) => {
                    match blob {
                        Some(blob) => {
                            println!("#{} thread received blob", i);
                            let blocks = parse_blocks(blob, Network::Bitcoin.magic());
                            let blocks_len = blocks.len();
                            let mut block_counter = block_counter_clone.lock().unwrap();
                            *block_counter += blocks_len;
                            println!("#{} thread received {} blocks, total {}", i, blocks_len, block_counter);
                            for block_and_size in blocks {
                                fee_sender_clone.send(Some(block_and_size)).unwrap();
                            }
                        },

                        None => {
                            println!("#{} received None, finishing", i);
                            break;
                        }
                    }

                },
                Err(e) => {
                    eprintln!("erro {:?}", e);
                    break;
                },
            }
        }
    });
    handles.push(handle);


    let handle = thread::spawn( move || {
        let mut i = 0usize;
        for path in paths.iter() {
            let blob = fs::read(path).expect(&format!("failed to read {:?}", path));
            let len = blob.len();
            println!("read {:?}", len);
            send_blocks.send(Some(blob)).expect("cannot send");
            i=i+1;
        }
        for _ in 0..thread {
            send_blocks.send(None).expect("cannot send None");
        }

    });

    handle.join().unwrap();
    for handle in handles {
        handle.join().unwrap();
    }
    fee_sender.send(None).unwrap();
    fee_handle.join().unwrap();*/
}


