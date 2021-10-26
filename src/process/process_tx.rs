use crate::process::{date_index, month_array_len};
use blocks_iterator::bitcoin::consensus::{encode, Decodable};
use blocks_iterator::bitcoin::{SigHashType, Transaction, Txid};
use blocks_iterator::log::{info, log};
use blocks_iterator::periodic_log_level;
use blocks_iterator::BlockExtra;
use chrono::{TimeZone, Utc};
use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Instant;

pub struct ProcessTxStats {
    receiver: Receiver<Arc<Option<BlockExtra>>>,
    pub stats: TxStats,
}

pub struct TxStats {
    pub min_weight_tx: (u64, Option<Txid>),
    pub max_inputs_per_tx: (u64, Option<Txid>),
    pub max_weight_tx: (u64, Option<Txid>),
    pub max_outputs_per_tx: (u64, Option<Txid>),
    pub total_outputs: u64,
    pub total_inputs: u64,
    pub total_outputs_per_month: Vec<u64>,
    pub total_inputs_per_month: Vec<u64>,
    pub total_tx_per_month: Vec<u64>,
    pub in_out: HashMap<String, u64>,
    pub amount_over_32: usize,
}

//TODO split again this one slower together with read
impl ProcessTxStats {
    pub fn new(receiver: Receiver<Arc<Option<BlockExtra>>>) -> ProcessTxStats {
        ProcessTxStats {
            receiver,
            stats: TxStats::new(),
        }
    }

    pub fn start(mut self) -> TxStats {
        let mut busy_time = 0u128;
        let mut now = Instant::now();
        loop {
            busy_time += now.elapsed().as_nanos();
            let received = self.receiver.recv().expect("cannot receive fee block");
            now = Instant::now();
            match *received {
                Some(ref block) => {
                    self.process_block(&block);
                    log!(
                        periodic_log_level(block.height, 10_000),
                        "busy_time:{}",
                        (busy_time / 1_000_000_000)
                    );
                }
                None => break,
            }
        }

        self.stats.total_inputs_per_month.pop();
        self.stats.total_outputs_per_month.pop();
        self.stats.total_tx_per_month.pop();

        busy_time += now.elapsed().as_nanos();
        info!(
            "ending stats processer, busy time: {}s",
            (busy_time / 1_000_000_000)
        );

        self.stats
    }

    fn process_block(&mut self, block: &BlockExtra) {
        let time = block.block.header.time;
        let date = Utc.timestamp(i64::from(time), 0);
        let index = date_index(date);

        for tx in block.block.txdata.iter() {
            self.process_tx(&tx, index);
        }
    }

    fn process_tx(&mut self, tx: &Transaction, index: usize) {
        let weight = tx.get_weight() as u64;
        let outputs = tx.output.len() as u64;
        let inputs = tx.input.len() as u64;
        self.stats.total_outputs_per_month[index] += outputs;
        self.stats.total_inputs_per_month[index] += inputs;
        self.stats.total_tx_per_month[index] += 1;
        let txid = tx.txid();
        self.stats.total_outputs += outputs as u64;
        self.stats.total_inputs += inputs as u64;
        if self.stats.max_outputs_per_tx.0 < outputs {
            self.stats.max_outputs_per_tx = (outputs, Some(txid));
        }
        if self.stats.max_inputs_per_tx.0 < inputs {
            self.stats.max_inputs_per_tx = (inputs, Some(txid));
        }
        if self.stats.max_weight_tx.0 < weight {
            self.stats.max_weight_tx = (weight, Some(txid));
        }
        if self.stats.min_weight_tx.0 > weight {
            self.stats.min_weight_tx = (weight, Some(txid));
        }

        let in_out_key = format!("{:02}-{:02}", inputs, outputs);
        *self.stats.in_out.entry(in_out_key).or_insert(0) += 1;
        self.stats.amount_over_32 += tx.output.iter().filter(|o| o.value > 0xffff_ffff).count();
    }
}

impl TxStats {
    pub fn new() -> Self {
        TxStats {
            max_outputs_per_tx: (100u64, None),
            max_inputs_per_tx: (100u64, None),
            min_weight_tx: (10000u64, None),
            max_weight_tx: (0u64, None),
            total_outputs: 0u64,
            total_inputs: 0u64,
            amount_over_32: 0usize,
            in_out: HashMap::new(),
            total_inputs_per_month: vec![0u64; month_array_len()],
            total_outputs_per_month: vec![0u64; month_array_len()],
            total_tx_per_month: vec![0u64; month_array_len()],
        }
    }
}

struct SignatureHash(pub SigHashType);

impl Decodable for SignatureHash {
    fn consensus_decode<D: std::io::Read>(mut d: D) -> Result<Self, encode::Error> {
        let first = u8::consensus_decode(&mut d)?;
        if first != 0x30 {
            return Err(encode::Error::ParseFailed("Signature must start with 0x30"));
        }
        let _ = u8::consensus_decode(&mut d)?;
        let integer_header = u8::consensus_decode(&mut d)?;
        if integer_header != 0x02 {
            return Err(encode::Error::ParseFailed("No integer header"));
        }
        let length_r = u8::consensus_decode(&mut d)?;
        for _ in 0..length_r {
            let _ = u8::consensus_decode(&mut d)?;
        }
        let integer_header = u8::consensus_decode(&mut d)?;
        if integer_header != 0x02 {
            return Err(encode::Error::ParseFailed("No integer header"));
        }
        let length_s = u8::consensus_decode(&mut d)?;
        for _ in 0..length_s {
            let _ = u8::consensus_decode(&mut d)?;
        }

        let sighash_u8 = u8::consensus_decode(&mut d)?;
        let sighash = SigHashType::from_u32_consensus(sighash_u8 as u32);

        Ok(SignatureHash(sighash))
    }
}
