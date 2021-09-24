use crate::process::*;
use bitcoin::util::bip158::BlockFilter;
use bitcoin::util::bip158::Error;
use bitcoin::Script;
use blocks_iterator::BlockExtra;
use chrono::{TimeZone, Utc};
use std::collections::HashSet;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Instant;
use std::{env, fs};
use std::fs::File;
use std::io::Read;
use std::convert::TryInto;

pub struct ProcessBip158Stats {
    receiver: Receiver<Arc<Option<BlockExtra>>>,
    stats: Bip158Stats,
    scripts_1m: HashSet<Script>, // counter of elements
    scripts_1m_heights: Vec<u32>,
    scripts_10m: HashSet<Script>, // counter of elements
    scripts_10m_heights: Vec<u32>,
    cache: Vec<u32>,
}

struct Bip158Stats {
    bip158_filter_size_per_month: Vec<u64>,
}

impl ProcessBip158Stats {
    pub fn new(receiver: Receiver<Arc<Option<BlockExtra>>>) -> Self {
        let cache = match File::open("bip138_size_cache") {
            Ok(mut file) => {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer).unwrap();
                let mut cache = Vec::new();
                for chunk in buffer.chunks(4) {
                    cache.push(u32::from_be_bytes(chunk.try_into().unwrap()))
                }
                cache
            }
            Err(_) => Vec::new(),
        };

        Self {
            receiver,
            cache,
            stats: Bip158Stats::new(),
            scripts_1m: HashSet::new(),
            scripts_1m_heights: vec![],
            scripts_10m: HashSet::new(),
            scripts_10m_heights: vec![],
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
        println!("scripts_1M: {:?}", self.scripts_1m_heights);
        println!("scripts_1M: {}", self.scripts_1m_heights.len());
        println!("scripts_10M: {:?}", self.scripts_10m_heights);
        println!("scripts_10M: {}", self.scripts_10m_heights.len());
    }

    fn process_block(&mut self, block: &BlockExtra) {
        let time = block.block.header.time;
        let date = Utc.timestamp(i64::from(time), 0);
        let index = date_index(date);

        let filter_len = match self.cache.get(block.height as usize) {
            Some(val) => *val,
            None =>{
                let filter = BlockFilter::new_script_filter(&block.block, |o| {
                    if let Some(s) = &block.outpoint_values.get(o) {
                        Ok(s.script_pubkey.clone())
                    } else {
                        Err(Error::UtxoMissing(o.clone()))
                    }
                })
                    .unwrap();
                let filter_len = filter.content.len() as u32;
                if let Ok(dir) = env::var("BIP158_DIR") {
                    let p = PathBuf::from_str(&format!("{}/{}.bin", dir, block.height)).unwrap();
                    fs::write(p, filter.content).unwrap();
                }
                filter_len
            }
        };
        self.cache[block.height as usize] = filter_len;

        self.stats.bip158_filter_size_per_month[index] += filter_len as u64;

        for tx in block.block.txdata.iter() {
            for input in tx.input.iter() {
                self.add_script(
                    &block
                        .outpoint_values
                        .get(&input.previous_output)
                        .unwrap()
                        .script_pubkey,
                    block.height,
                );
            }
            for output in tx.output.iter() {
                self.add_script(&output.script_pubkey, block.height)
            }
        }
    }

    fn add_script(&mut self, script: &Script, height: u32) {
        self.scripts_1m.insert(script.clone());
        if self.scripts_1m.len() >= 1_000_000 {
            self.scripts_1m.clear();
            self.scripts_1m_heights.push(height);
        }
        self.scripts_10m.insert(script.clone());
        if self.scripts_10m.len() >= 10_000_000 {
            self.scripts_10m.clear();
            self.scripts_10m_heights.push(height);
        }
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
