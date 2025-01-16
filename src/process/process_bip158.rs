use crate::counter::Counter;
use crate::process::block_index;
use bitcoin::bip158::BlockFilter;
use bitcoin::ScriptBuf;
use blocks_iterator::log::{debug, info};
use blocks_iterator::BlockExtra;
use blocks_iterator::PeriodCounter;
use std::collections::HashSet;
use std::convert::TryInto;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use std::{env, fs};

pub struct ProcessBip158Stats {
    receiver: Receiver<Arc<Option<BlockExtra>>>,
    pub stats: Bip158Stats,

    /// counter of different scripts, when reach 1M elements, it resets and insert the height in `scripts_1m_heights`
    scripts_1m: HashSet<ScriptBuf>,
    scripts_1m_heights: Vec<u32>,

    /// cache the value of the BIP158 filter
    cache: Vec<u32>,
    cache_path: PathBuf,
}

pub struct Bip158Stats {
    pub bip158_filter_size_per_period: Counter,
}

impl ProcessBip158Stats {
    pub fn new(receiver: Receiver<Arc<Option<BlockExtra>>>, target_dir: &PathBuf) -> Self {
        let mut cache_path = target_dir.clone();
        cache_path.push("raw");
        cache_path.push("bip138_size_cache");

        let cache = match File::open(&cache_path) {
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
            cache_path,
            stats: Bip158Stats::new(),
            scripts_1m: HashSet::new(),
            scripts_1m_heights: vec![],
        }
    }

    pub fn start(mut self) -> Bip158Stats {
        let mut busy_time = 0u128;
        let mut now = Instant::now();
        let mut period = PeriodCounter::new(Duration::from_secs(10));
        loop {
            busy_time += now.elapsed().as_nanos();
            let received = self.receiver.recv().expect("cannot receive fee block");
            now = Instant::now();
            match *received {
                Some(ref block) => {
                    self.process_block(&block);
                    if period.period_elapsed().is_some() {
                        info!("busy_time:{}", (busy_time / 1_000_000_000));
                    }
                }
                None => break,
            }
        }
        let mut file = File::create(self.cache_path).unwrap();
        for size in self.cache.iter() {
            file.write(&size.to_be_bytes()).unwrap();
        }

        busy_time += now.elapsed().as_nanos();
        info!(
            "ending bip158 stats processer, busy time: {}s",
            (busy_time / 1_000_000_000)
        );
        debug!("scripts_1M: {:?}", self.scripts_1m_heights);
        info!("scripts_1M: {}", self.scripts_1m_heights.len());

        self.stats
    }

    fn process_block(&mut self, block: &BlockExtra) {
        let index = block_index(block.height());

        let (filter_len, insert) = match self.cache.get(block.height() as usize) {
            Some(val) => (*val, false),
            None => {
                let filter = BlockFilter::new_script_filter(&block.block(), |o| {
                    if let Some(s) = &block.outpoint_values().get(o) {
                        Ok(s.script_pubkey.clone())
                    } else {
                        Err(bitcoin::bip158::Error::UtxoMissing(o.clone()))
                    }
                })
                .unwrap();
                let filter_len = filter.content.len() as u32;
                if let Ok(dir) = env::var("BIP158_DIR") {
                    let p = PathBuf::from_str(&format!("{}/{}.bin", dir, block.height())).unwrap();
                    fs::write(p, filter.content).unwrap();
                }
                (filter_len, true)
            }
        };
        if insert {
            self.cache.push(filter_len);
        }

        self.stats
            .bip158_filter_size_per_period
            .add(index, filter_len as u64);

        for tx in block.block().txdata.iter() {
            for input in tx.input.iter() {
                self.add_script(
                    &block
                        .outpoint_values()
                        .get(&input.previous_output)
                        .unwrap()
                        .script_pubkey,
                    block.height(),
                );
            }
            for output in tx.output.iter() {
                self.add_script(&output.script_pubkey, block.height())
            }
        }
    }

    fn add_script(&mut self, script: &ScriptBuf, height: u32) {
        self.scripts_1m.insert(script.clone());
        if self.scripts_1m.len() >= 1_000_000 {
            self.scripts_1m.clear();
            self.scripts_1m_heights.push(height);
        }
    }
}

impl Bip158Stats {
    fn new() -> Self {
        Self {
            bip158_filter_size_per_period: Counter::new(),
        }
    }
}
