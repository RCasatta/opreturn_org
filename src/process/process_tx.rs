use crate::counter::Counter;
use crate::pages::bip69::{has_more_than_one_input_output, is_bip69};
use crate::process::{block_index, compress_amount, encoded_length_7bit_varint};
use blocks_iterator::bitcoin::consensus::{encode, Decodable};
use blocks_iterator::bitcoin::{EcdsaSighashType, Transaction, Txid, VarInt};
use blocks_iterator::log::info;
use blocks_iterator::BlockExtra;
use blocks_iterator::PeriodCounter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub struct ProcessTxStats {
    receiver: Receiver<Arc<Option<BlockExtra>>>,
    pub stats: TxStats,
    pub tx_stats_json_file: File,
}

#[derive(Default, Serialize, Deserialize)]
pub struct TxStats {
    pub min_weight_tx: (u64, Option<Txid>),
    pub max_inputs_per_tx: (u64, Option<Txid>),
    pub max_weight_tx: (u64, Option<Txid>),
    pub max_outputs_per_tx: (u64, Option<Txid>),
    pub total_outputs: u64,
    pub total_inputs: u64,
    pub total_outputs_per_period: Counter,
    pub total_inputs_per_period: Counter,
    pub script_pubkey_size_per_period: Counter,
    pub total_tx_per_period: Counter,
    pub in_out: HashMap<String, u64>,
    pub amount_over_32: usize,
    pub many_inputs_tx: Vec<Txid>,

    pub total_bytes_output_value_varint: u64,
    pub total_bytes_output_value_compressed_varint: u64,
    pub total_bytes_output_value_bitcoin_varint: u64,
    pub total_bytes_output_value_compressed_bitcoin_varint: u64,
    pub rounded_amount_per_period: Counter,
    pub rounded_amount: u64,

    pub is_bip69: [Counter; 2],

    /// number of txs with a nlocktime higher than confirmed block height minus 6
    pub non_deeply_reorgable: Counter,
}

//TODO split again this one slower together with read
impl ProcessTxStats {
    pub fn new(
        receiver: Receiver<Arc<Option<BlockExtra>>>,
        target_dir: &PathBuf,
    ) -> ProcessTxStats {
        let tx_stats_json_file =
            File::create(format!("{}/tx_stats.json", target_dir.display())).unwrap();
        ProcessTxStats {
            receiver,
            stats: TxStats::new(),
            tx_stats_json_file,
        }
    }

    pub fn start(mut self) -> TxStats {
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

        let tx_stats_json = serde_json::to_string(&self.stats).unwrap();
        self.tx_stats_json_file
            .write_all(tx_stats_json.as_bytes())
            .unwrap();

        busy_time += now.elapsed().as_nanos();
        info!(
            "ending stats processer, busy time: {}s",
            (busy_time / 1_000_000_000)
        );

        self.stats
    }

    fn process_block(&mut self, block_extra: &BlockExtra) {
        let index = block_index(block_extra.height);

        for (txid, tx) in block_extra.iter_tx() {
            self.process_tx(*txid, &tx, index, block_extra.height);
        }
    }

    fn process_tx(&mut self, txid: Txid, tx: &Transaction, index: usize, block_height: u32) {
        let weight = tx.weight() as u64;
        let outputs = tx.output.len() as u64;
        let inputs = tx.input.len() as u64;
        self.stats.total_outputs_per_period.add(index, outputs);
        self.stats.total_inputs_per_period.add(index, inputs);
        self.stats.total_tx_per_period.increment(index);
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
        if inputs > 500 {
            self.stats.many_inputs_tx.push(txid);
        }

        let in_out_key = if inputs > 9 || outputs > 9 {
            "10+10".to_string()
        } else {
            format!("{:02}-{:02}", inputs, outputs)
        };

        *self.stats.in_out.entry(in_out_key).or_insert(0) += 1;
        self.stats.amount_over_32 += tx.output.iter().filter(|o| o.value > 0xffff_ffff).count();

        for output in tx.output.iter() {
            let len = VarInt(output.value).len() as u64;

            self.stats.total_bytes_output_value_bitcoin_varint += len;
            self.stats.total_bytes_output_value_varint += encoded_length_7bit_varint(output.value);
            let compressed = compress_amount(output.value);
            self.stats
                .total_bytes_output_value_compressed_bitcoin_varint +=
                VarInt(compressed).len() as u64;
            self.stats.total_bytes_output_value_compressed_varint +=
                encoded_length_7bit_varint(compressed);
            if (output.value % 1000) == 0 {
                self.stats.rounded_amount_per_period.increment(index);
                self.stats.rounded_amount += 1;
            }

            self.stats
                .script_pubkey_size_per_period
                .add(index, output.script_pubkey.len() as u64);
        }

        if has_more_than_one_input_output(&tx) {
            self.stats
                .is_bip69
                .get_mut(is_bip69(&tx) as usize)
                .expect("all keys inserted during init")
                .increment(index);
        }

        if tx.lock_time > block_height.saturating_sub(6) {
            self.stats.non_deeply_reorgable.increment(index);
        }
    }
}

impl TxStats {
    pub fn new() -> Self {
        TxStats {
            max_outputs_per_tx: (u64::MIN, None),
            max_inputs_per_tx: (u64::MIN, None),
            min_weight_tx: (u64::MAX, None),
            max_weight_tx: (u64::MIN, None),
            ..Default::default()
        }
    }
}

struct SignatureHash(pub EcdsaSighashType);

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
        let sighash = EcdsaSighashType::from_consensus(sighash_u8 as u32);

        Ok(SignatureHash(sighash))
    }
}
