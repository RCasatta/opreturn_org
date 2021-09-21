mod process_bip158;

use crate::fee::Fee;
use crate::parse::Parse;
use crate::process::Process;
use crate::process_bip158::ProcessBip158Stats;
use crate::process_stats::ProcessStats;
use crate::read::Read;
use crate::reorder::Reorder;
use bitcoin::{Block, BlockHash, OutPoint, Transaction, TxOut, Txid};
use rocksdb::{DB, Env};
use std::collections::{HashMap, HashSet};
use std::env;
use std::path::PathBuf;
use std::sync::mpsc::sync_channel;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

mod fee;
mod parse;
mod process;
mod process_stats;
mod read;
mod reorder;

fn main() {

    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    info!("start");

    let config = Config::from_args();
    let (send, recv) = sync_channel(100);
    let handle = blocks_iterator::iterate(config, send);

    let (send_1, receive_1) = sync_channel(blocks_size);
    let (send_2, receive_2) = sync_channel(blocks_size);
    let (send_3, receive_3) = sync_channel(blocks_size);
    let senders = [send_1, send_2, send_3];

    let mut process = Process::new(receive_1);
    let process_handle = thread::spawn(move || {
        process.start();
    });

    let mut process_stats = ProcessStats::new(receive_2);
    let process_stats_handle = thread::spawn(move || {
        process_stats.start();
    });

    let mut process_bip158 = ProcessBip158Stats::new(receive_3, db);
    let process_bip158_handle = thread::spawn(move || {
        process_bip158.start();
    });

    while let Some(block_extra) = recv.recv()? {
        let block_extra = Arc::new(Some(block_extra));
        for sender in senders.iter() {
            sender.send(block_extra.clone())
        }
        log!(
            periodic_log_level(block_extra.height),
            "# {:7} {} {:?}",
            block_extra.height,
            block_extra.block_hash,
            block_extra.fee()
        );

    }
    handle.join().expect("couldn't join");
    info!("end");

}
