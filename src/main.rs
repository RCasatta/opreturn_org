mod process_bip158;

use crate::fee::Fee;
use crate::parse::Parse;
use crate::process::Process;
use crate::process_bip158::ProcessBip158Stats;
use crate::process_stats::ProcessStats;
use crate::read::Read;
use crate::reorder::Reorder;
use bitcoin::{Block, BlockHash, OutPoint, TxOut, Txid};
use rocksdb::DB;
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;
use std::sync::mpsc::sync_channel;
use std::sync::Arc;
use std::thread;

mod fee;
mod parse;
mod process;
mod process_stats;
mod read;
mod reorder;

#[derive(Debug)]
pub struct BlockExtra {
    pub block: Block,
    pub next: Vec<BlockHash>, // vec cause in case of reorg could be more than one
    pub size: u32,
    pub height: u32,
    pub out_of_order_size: usize,
    pub outpoint_values: HashMap<OutPoint, TxOut>,
    pub tx_hashes: HashSet<Txid>,
}

fn main() {
    let path = PathBuf::from(env::var("BITCOIN_DIR").unwrap_or("~/.bitcoin/".to_string()));
    let blob_size = env::var("BLOB_CHANNEL_SIZE")
        .unwrap_or("1".to_string())
        .parse::<usize>()
        .unwrap_or(1);
    let blocks_size = env::var("BLOCKS_CHANNEL_SIZE")
        .unwrap_or("10".to_string())
        .parse::<usize>()
        .unwrap_or(10);
    let mut db_opts = rocksdb::Options::default();
    db_opts.increase_parallelism(4);
    db_opts.create_if_missing(true);
    let db = Arc::new(DB::open(&db_opts, env::var("DB").unwrap_or_else(|_| "db".into())).unwrap());

    let (send_blobs, receive_blobs) = sync_channel(blob_size);
    let mut read = Read::new(path, send_blobs);
    let read_handle = thread::spawn(move || {
        read.start();
    });

    let (send_blocks, receive_blocks) = sync_channel(blocks_size);
    let mut parse = Parse::new(receive_blobs, send_blocks);
    let parse_handle = thread::spawn(move || {
        parse.start();
    });

    let (send_ordered_blocks, receive_ordered_blocks) = sync_channel(blocks_size);
    let mut reorder = Reorder::new(receive_blocks, send_ordered_blocks);
    let orderer_handle = thread::spawn(move || {
        reorder.start();
    });

    let (send_blocks_and_fee_1, receive_blocks_and_fee_1) = sync_channel(blocks_size);
    let (send_blocks_and_fee_2, receive_blocks_and_fee_2) = sync_channel(blocks_size);
    let (send_blocks_and_fee_3, receive_blocks_and_fee_3) = sync_channel(blocks_size);

    let mut fee = Fee::new(
        receive_ordered_blocks,
        vec![
            send_blocks_and_fee_1,
            send_blocks_and_fee_2,
            send_blocks_and_fee_3,
        ],
        db.clone(),
    );
    let fee_handle = thread::spawn(move || {
        fee.start();
    });

    let mut process = Process::new(receive_blocks_and_fee_1);
    let process_handle = thread::spawn(move || {
        process.start();
    });

    let mut process_stats = ProcessStats::new(receive_blocks_and_fee_2);
    let process_stats_handle = thread::spawn(move || {
        process_stats.start();
    });

    let mut process_bip158 = ProcessBip158Stats::new(receive_blocks_and_fee_3, db);
    let process_bip158_handle = thread::spawn(move || {
        process_bip158.start();
    });

    read_handle.join().unwrap();
    parse_handle.join().unwrap();
    orderer_handle.join().unwrap();
    fee_handle.join().unwrap();
    process_handle.join().unwrap();
    process_stats_handle.join().unwrap();
    process_bip158_handle.join().unwrap();
}
