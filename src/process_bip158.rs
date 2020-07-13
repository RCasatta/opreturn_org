use crate::process::*;
use crate::BlockExtra;
use bitcoin::consensus::{deserialize, serialize};
use bitcoin::util::bip158::BlockFilter;
use bitcoin::util::bip158::Error;
use bitcoin::util::hash::BitcoinHash;
use chrono::{TimeZone, Utc};
use rocksdb::DB;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Instant;
use std::{env, fs};

pub struct ProcessBip158Stats {
    receiver: Receiver<Arc<Option<BlockExtra>>>,
    stats: Bip158Stats,
    db: Arc<DB>, // previous_hashes: VecDeque<HashSet<sha256d::Hash>>,
}

// TODO slowest, bring bip158 out/remove?
struct Bip158Stats {
    bip158_filter_size_per_month: Vec<u64>,
}

impl ProcessBip158Stats {
    pub fn new(receiver: Receiver<Arc<Option<BlockExtra>>>, db: Arc<DB>) -> Self {
        Self {
            receiver,
            stats: Bip158Stats::new(),
            db,
        }
    }

    pub fn start(&mut self) {
        let mut busy_time = 0u128;
        loop {
            let received = self.receiver.recv().expect("cannot receive fee block");
            match *received {
                Some(ref block) => {
                    let now = Instant::now();
                    self.process_block(&block);
                    busy_time = busy_time + now.elapsed().as_nanos();
                }
                None => break,
            }
        }

        self.stats.bip158_filter_size_per_month.pop();
        let toml = self.stats.to_toml();
        //println!("{}", toml);
        fs::write("site/_data/bip158_stats.toml", toml).expect("Unable to w rite file");

        println!(
            "ending bip158 stats processer, busy time: {}s",
            (busy_time / 1_000_000_000)
        );
    }

    fn process_block(&mut self, block: &BlockExtra) {
        let time = block.block.header.time;
        let date = Utc.timestamp(i64::from(time), 0);
        let index = date_index(date);
        let key = filter_key(block.block.bitcoin_hash());

        let filter_len = match self.db.get(&key).expect("operational problem encountered") {
            Some(value) => deserialize::<u64>(&value).expect("cant deserialize u64"),
            None => {
                let filter = BlockFilter::new_script_filter(&block.block, |o| {
                    if let Some(s) = &block.outpoint_values.get(o) {
                        Ok(s.script_pubkey.clone())
                    } else {
                        Err(Error::UtxoMissing(o.clone()))
                    }
                })
                .unwrap();
                let filter_len = filter.content.len() as u64;
                if let Ok(dir) = env::var("BIP158_DIR") {
                    let p = PathBuf::from_str(&format!("{}/{}.bin", dir, block.height)).unwrap();
                    fs::write(p, filter.content).unwrap();
                }
                self.db
                    .put(&key, &serialize(&filter_len))
                    .expect("error in write");
                filter_len
            }
        };
        self.stats.bip158_filter_size_per_month[index] += filter_len;
    }
}

impl Bip158Stats {
    fn new() -> Self {
        Self {
            bip158_filter_size_per_month: vec![0u64; month_array_len()],
        }
    }

    fn to_toml(&self) -> String {
        let mut s = String::new();

        s.push_str("\n\n");
        s.push_str(&toml_section_vec(
            "bip158_filter_size_per_month",
            &self.bip158_filter_size_per_month,
            None,
        ));

        s.push_str("\n\n");
        s.push_str(&toml_section_vec(
            "bip158_filter_size_per_month_cum",
            &cumulative(&self.bip158_filter_size_per_month),
            None,
        ));

        s
    }
}
