use std::path::PathBuf;
use std::env;
use std::sync::mpsc::sync_channel;
use std::thread;
use rocksdb::DB;
use crate::read::Read;
use crate::parse::Parse;
use crate::fee::Fee;
use crate::reorder::Reorder;
use crate::process::Process;
use std::collections::HashMap;
use bitcoin::Block;
use bitcoin::OutPoint;
use bitcoin_hashes::sha256d;

mod fee;
mod parse;
mod read;
mod reorder;
mod process;

#[derive(Debug)]
pub struct BlockExtra {
    pub block: Block,
    pub next: Vec<sha256d::Hash>, // reorg
    pub size: u32,
    pub height: u32,
    pub out_of_order_size: usize,
    pub outpoint_values: HashMap<OutPoint, u64>,
}

fn main() {
    let path = PathBuf::from(env::var("BITCOIN_DIR").unwrap_or("~/.bitcoin/".to_string()));
    let blob_size = env::var("BLOB_CHANNEL_SIZE").unwrap_or("1".to_string()).parse::<usize>().unwrap_or(2);
    let blocks_size = env::var("BLOCKS_CHANNEL_SIZE").unwrap_or("100".to_string()).parse::<usize>().unwrap_or(200);
    let mut db_opts = rocksdb::Options::default();
    db_opts.increase_parallelism(4);
    db_opts.create_if_missing(true);
    let db = DB::open(&db_opts, env::var("DB").unwrap_or("db".to_string())).unwrap();

    let (send_blobs, receive_blobs) = sync_channel(blob_size);
    let mut read = Read::new(path, send_blobs);
    let read_handle = thread::spawn( move || { read.start(); });

    let (send_blocks, receive_blocks) = sync_channel(blocks_size);
    let mut parse = Parse::new(receive_blobs, send_blocks);
    let parse_handle = thread::spawn( move || { parse.start(); });

    let (send_ordered_blocks, receive_ordered_blocks) = sync_channel(blocks_size);
    let mut reorder = Reorder::new(receive_blocks, send_ordered_blocks);
    let orderer_handle = thread::spawn( move || { reorder.start(); });

    let (send_blocks_and_fee, receive_blocks_and_fee) = sync_channel(blocks_size);
    let mut fee = Fee::new(receive_ordered_blocks, send_blocks_and_fee, db);
    let fee_handle = thread::spawn( move || { fee.start(); });

    let mut process = Process::new(receive_blocks_and_fee);
    let process_handle = thread::spawn( move || { process.start(); });

    read_handle.join().unwrap();
    parse_handle.join().unwrap();
    orderer_handle.join().unwrap();
    fee_handle.join().unwrap();
    process_handle.join().unwrap();

}


