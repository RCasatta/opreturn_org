use crate::counter::Counter;
use crate::process::{block_index, parse_multisig, parse_pubkeys_in_tx};
use blocks_iterator::bitcoin::Script;
use blocks_iterator::log::{debug, info};
use blocks_iterator::BlockExtra;
use blocks_iterator::PeriodCounter;
use chrono::Duration;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::time::Instant;

const OP_RETURN_BUCKETS: [usize; 47] = [
    0, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 200, 300, 400, 500, 600, 700, 800, 900, 1000, 2000,
    3000, 4000, 5000, 6000, 7000, 8000, 9000, 10000, 20000, 30000, 40000, 50000, 60000, 70000,
    80000, 90000, 100000, 200000, 300000, 400000, 500000, 600000, 700000, 800000, 900000, 1000000,
];

pub struct ProcessOpRet {
    receiver: Receiver<Arc<Option<BlockExtra>>>,
    pub op_return_data: OpReturnData,
    pub script_type: ScriptType,
    pub opret_json_file: File,
    pub parse_pubkeys: bool,
}

#[derive(Default, Serialize, Deserialize)]
pub struct OpReturnData {
    pub op_ret_per_period: Counter,
    pub op_ret_size: BTreeMap<String, u64>,
    pub op_ret_fee_per_period: Counter,
    pub op_ret_per_proto: HashMap<String, u64>,
    pub op_ret_per_proto_last_month: HashMap<String, u64>,
    pub op_ret_per_proto_last_year: HashMap<String, u64>,
    pub month_ago: u32,
    pub year_ago: u32,

    pub compressed_starts_with: Counter,
    pub uncompressed_starts_with: Counter,
}

#[derive(Default)]
pub struct ScriptType {
    pub all: Counter,
    pub p2pkh: Counter,
    pub p2pk: Counter,
    pub v0_p2wpkh: Counter,
    pub v0_p2wsh: Counter,
    pub p2sh: Counter,
    pub p2tr: Counter,
    pub other: Counter,
    pub multisig: HashMap<String, u64>,
    pub multisig_tx: HashMap<String, String>,
}

impl ProcessOpRet {
    pub fn new(
        receiver: Receiver<Arc<Option<BlockExtra>>>,
        target_dir: &PathBuf,
        parse_pubkeys: bool,
    ) -> ProcessOpRet {
        let opret_json_file =
            File::create(format!("{}/raw/opret.json", target_dir.display())).unwrap();
        ProcessOpRet {
            receiver,
            op_return_data: OpReturnData::new(),
            script_type: ScriptType::new(),
            opret_json_file,
            parse_pubkeys,
        }
    }

    pub fn start(mut self) -> (OpReturnData, ScriptType) {
        let mut busy_time = 0u128;
        let mut now = Instant::now();
        let mut period = PeriodCounter::new(std::time::Duration::from_secs(10));
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

        debug!("{:?}", self.script_type.multisig_tx);

        let opret_json = serde_json::to_string(&self.op_return_data).unwrap();
        self.opret_json_file
            .write_all(opret_json.as_bytes())
            .unwrap();

        busy_time += now.elapsed().as_nanos();
        info!(
            "ending processer, busy time: {}s",
            (busy_time / 1_000_000_000)
        );

        (self.op_return_data, self.script_type)
    }

    fn process_block(&mut self, block_extra: &BlockExtra) {
        let time = block_extra.block().header.time;
        let index = block_index(block_extra.height());

        for (txid, tx) in block_extra.iter_tx() {
            for output in tx.output.iter() {
                if output.script_pubkey.is_op_return() {
                    self.process_op_return_script(
                        &output.script_pubkey,
                        time,
                        index,
                        block_extra.tx_fee(&tx).unwrap(),
                    );
                }
                self.process_output_script(&output.script_pubkey, index);
            }
            for input in tx.input.iter() {
                if let Some(witness_script) = input.witness.last() {
                    if let Some(key) = parse_multisig(witness_script) {
                        if self.script_type.multisig_tx.get(&key).is_none() {
                            self.script_type
                                .multisig_tx
                                .insert(key.clone(), format!("{}", txid));
                        }
                        *self.script_type.multisig.entry(key).or_insert(0) += 1;
                    }
                }
            }

            if self.parse_pubkeys {
                for p in parse_pubkeys_in_tx(tx) {
                    if p.compressed {
                        self.op_return_data
                            .compressed_starts_with
                            .increment(p.to_bytes()[0] as usize);
                    } else {
                        self.op_return_data
                            .uncompressed_starts_with
                            .increment(p.to_bytes()[0] as usize);
                    }
                }
            }
        }
    }

    fn process_output_script(&mut self, script: &Script, index: usize) {
        self.script_type.all.increment(index);
        if script.is_p2pkh() {
            self.script_type.p2pkh.increment(index);
        } else if script.is_p2pk() {
            self.script_type.p2pk.increment(index);
        } else if script.is_p2wpkh() {
            self.script_type.v0_p2wpkh.increment(index);
        } else if script.is_p2wsh() {
            self.script_type.v0_p2wsh.increment(index);
        } else if script.is_p2sh() {
            self.script_type.p2sh.increment(index);
        } else if script.is_p2tr() {
            self.script_type.p2tr.increment(index);
        } else {
            self.script_type.other.increment(index);
        }
    }

    fn process_op_return_script(
        &mut self,
        op_return_script: &Script,
        time: u32,
        index: usize,
        fee: u64,
    ) {
        let script_bytes = op_return_script.as_bytes();
        let script_hex = hex::encode(script_bytes);
        let script_len = script_bytes.len();
        let data = &mut self.op_return_data;

        // Find the appropriate bucket for this script length
        let bucket_key = if let Some(bucket_idx) = OP_RETURN_BUCKETS
            .windows(2)
            .position(|w| script_len >= w[0] && script_len < w[1])
        {
            format!(
                "{:>6}-{}",
                OP_RETURN_BUCKETS[bucket_idx],
                OP_RETURN_BUCKETS[bucket_idx + 1]
            )
        } else if script_len >= *OP_RETURN_BUCKETS.last().unwrap() {
            format!("{:>6}+", OP_RETURN_BUCKETS.last().unwrap())
        } else {
            // Shouldn't happen if buckets start at 0
            format!("{}", script_len)
        };

        *data.op_ret_size.entry(bucket_key).or_insert(0) += 1;
        data.op_ret_per_period.increment(index);
        data.op_ret_fee_per_period.add(index, fee);

        if script_len > 4 {
            let op_ret_proto = if script_hex.starts_with("6a4c") && script_len > 5 {
                // 4c = OP_PUSHDATA1
                String::from(&script_hex[6..12])
            } else {
                String::from(&script_hex[4..10])
            };
            if time > data.year_ago {
                *data
                    .op_ret_per_proto_last_year
                    .entry(op_ret_proto.clone())
                    .or_insert(0) += 1;

                if time > data.month_ago {
                    *data
                        .op_ret_per_proto_last_month
                        .entry(op_ret_proto.clone())
                        .or_insert(0) += 1;
                }
            }
            *data
                .op_ret_per_proto
                .entry(op_ret_proto.clone())
                .or_insert(0) += 1;
        }
    }
}

impl ScriptType {
    fn new() -> Self {
        ScriptType {
            ..Default::default()
        }
    }
}

impl OpReturnData {
    fn new() -> OpReturnData {
        let month_ago = (Utc::now() - Duration::days(30)).timestamp() as u32; // 1 month ago
        let year_ago = (Utc::now() - Duration::days(365)).timestamp() as u32; // 1 year ago
        OpReturnData {
            month_ago,
            year_ago,
            ..Default::default()
        }
    }
}

/*
#[cfg(test)]
mod test {
    use crate::process::cumulative;
    use crate::process::decompress_amount;
    use crate::process::index_month;
    use crate::process::parse_multisig;
    use crate::process::{compress_amount, SignatureHash};
    use crate::process::{date_index, encoded_length_7bit_varint, month_date};
    use bitcoin::consensus::{deserialize, Decodable};
    use bitcoin::SigHashType;
    use chrono::offset::TimeZone;
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test0() {
        let date = Utc.timestamp(1230768000i64, 0);
        assert_eq!(0, date_index(date));
        assert_eq!("200901", index_month(0));
        let date = Utc.timestamp(1262304000i64, 0);
        assert_eq!(12, date_index(date));
        assert_eq!("200912", index_month(11));
        assert_eq!("201001", index_month(12));
        for i in 0..2000 {
            assert_eq!(i, date_index(month_date(index_month(i))));
        }
    }

    #[test]
    fn test1() {
        assert_eq!(encoded_length_7bit_varint(127), 1);
        assert_eq!(encoded_length_7bit_varint(128), 2);
        assert_eq!(encoded_length_7bit_varint(1_270), 2);
        assert_eq!(encoded_length_7bit_varint(111_270), 3);
        assert_eq!(encoded_length_7bit_varint(2_097_151), 3);
        assert_eq!(encoded_length_7bit_varint(2_097_152), 4);
    }

    #[test]
    fn test2() {
        let tuples = vec![("one", 1), ("two", 2), ("three", 3)];
        let m: HashMap<_, _> = tuples.into_iter().collect();
        println!("{:?}", m);
    }

    #[test]
    fn test3() {
        let mut b: bool = true;
        let u = b as u32;
        assert_eq!(u, 1u32);
        b = false;
        let u = b as u32;
        assert_eq!(u, 0u32);
    }

    #[test]
    fn test4() {
        let i = 10_000_000_000;
        let compressed = compress_amount(i);
        println!("compressed: {}", compressed);
        assert_eq!(i, decompress_amount(compressed));

        for i in 0..std::u64::MAX {
            assert_eq!(i, decompress_amount(compress_amount(i)));
        }
    }

    #[test]
    fn test5() {
        let vec = vec![1, 1, 1];
        assert_eq!(cumulative(&vec), vec![1, 2, 3]);
    }

    #[test]
    fn test_parse_multisig() {
        let script = hex::decode("52210293de2378b245e0c4a8325d2beb2e537041a3b9b12c96052a9f30954700e56ef3210230d013baf38205252c298625a7c7799e1f11a016d3738198410bcf8bcc1fecab52ae").unwrap();
        assert_eq!(Some("02of02".to_string()), parse_multisig(&script));
    }

    #[test]
    fn test_decode_signature() {
        let der_signature = hex::decode("3045022100bd3688bbeefe67dbaf34b7e7d250bcbcf99c8a5cf7cb680393f5025b03dac912022057dbf2317c3413b57eeaf712f1599b74213f1a4ea4e3f5091db6f7fe8d02465a01").unwrap();

        let signatureHash: SignatureHash = deserialize(&der_signature).unwrap();
        assert_eq!(signatureHash.0, SigHashType::All);

        let der_signature = hex::decode("3045022100bd3688bbeefe67dbaf34b7e7d250bcbcf99c8a5cf7cb680393f5025b03dac912022057dbf2317c3413b57eeaf712f1599b74213f1a4ea4e3f5091db6f7fe8d02465a83").unwrap();

        let signatureHash: SignatureHash = deserialize(&der_signature).unwrap();
        assert_eq!(signatureHash.0, SigHashType::SinglePlusAnyoneCanPay);
    }
}

*/
