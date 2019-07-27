use crate::fee::tx_fee;
use crate::BlockExtra;
use bitcoin::util::hash::BitcoinHash;
use bitcoin::Script;
use bitcoin::Transaction;
use bitcoin::VarInt;
use bitcoin::util::bip158::BlockFilter;
use bitcoin::util::bip158::Error;
use bitcoin_hashes::hex::FromHex;
use bitcoin_hashes::sha256d;
use chrono::DateTime;
use chrono::{Datelike, TimeZone, Utc};
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::sync::mpsc::Receiver;
use time::Duration;
use std::time::Instant;

pub struct Process {
    receiver: Receiver<Option<BlockExtra>>,
    op_return_data: OpReturnData,
    stats: Stats,
    script_type: ScriptType,
    // previous_hashes: VecDeque<HashSet<sha256d::Hash>>,
}

struct OpReturnData {
    op_ret_per_month: Vec<u64>,
    op_ret_size: BTreeMap<String, u64>, //pad with spaces usize of len up to 3
    veriblock_per_month: Vec<u64>,
    op_ret_fee_per_month: Vec<u64>,
    veriblock_fee_per_month: Vec<u64>,
    op_ret_per_proto: HashMap<String, u64>,
    op_ret_per_proto_last_month: HashMap<String, u64>,
    op_ret_per_proto_last_year: HashMap<String, u64>,
    month_ago: u32,
    year_ago: u32,
}

struct Stats {
    max_outputs_per_tx: (u64, Option<sha256d::Hash>),
    min_weight_tx: (u64, Option<sha256d::Hash>),
    max_inputs_per_tx: (u64, Option<sha256d::Hash>),
    max_weight_tx: (u64, Option<sha256d::Hash>),
    total_outputs: u64,
    total_inputs: u64,
    amount_over_32: usize,
    rounded_amount: u64,
    max_block_size: (u64, Option<sha256d::Hash>),
    min_hash: sha256d::Hash,
    total_spent_in_block: u64,
    total_spent_in_block_per_month: Vec<u64>,
    total_bytes_output_value_varint: u64,
    total_bytes_output_value_compressed_varint: u64,
    total_bytes_output_value_bitcoin_varint: u64,
    total_bytes_output_value_compressed_bitcoin_varint: u64,
    rounded_amount_per_month: Vec<u64>,
    bip158_filter_size_per_month: Vec<u64>,
}

struct ScriptType {
    all: Vec<u64>,
    p2pkh: Vec<u64>,
    p2pk: Vec<u64>,
    v0_p2wpkh: Vec<u64>,
    v0_p2wsh: Vec<u64>,
    p2sh: Vec<u64>,
    other: Vec<u64>,
}

impl Process {
    pub fn new(receiver: Receiver<Option<BlockExtra>>) -> Process {
        Process {
            receiver,
            op_return_data: OpReturnData::new(),
            stats: Stats::new(),
            script_type: ScriptType::new(),
        }
    }

    pub fn start(&mut self) {
        let mut busy_time = 0u128;
        loop {
            let received = self.receiver.recv().expect("cannot receive fee block");
            match received {
                Some(block) => {
                    let now = Instant::now();
                    self.process_block(block);
                    busy_time = busy_time + now.elapsed().as_nanos();
                }
                None => break,
            }
        }

        //remove current month and cut initial months if not significant
        self.op_return_data.op_ret_per_month.pop();
        self.op_return_data.op_ret_per_month = self.op_return_data.op_ret_per_month[month_index("201501".to_string())..].to_vec();
        self.op_return_data.op_ret_fee_per_month.pop();
        self.op_return_data.op_ret_fee_per_month = self.op_return_data.op_ret_fee_per_month[month_index("201501".to_string())..].to_vec();
        self.op_return_data.op_ret_fee_per_month.pop();
        self.op_return_data.veriblock_per_month.pop();
        self.op_return_data.veriblock_per_month = self.op_return_data.veriblock_per_month[month_index("201807".to_string())..].to_vec();
        self.op_return_data.veriblock_fee_per_month.pop();
        self.op_return_data.veriblock_fee_per_month = self.op_return_data.veriblock_fee_per_month[month_index("201807".to_string())..].to_vec();

        let toml = self.op_return_data.to_toml();
        println!("{}", toml);
        fs::write("site/_data/op_return.toml", toml).expect("Unable to write file");

        self.stats.total_spent_in_block_per_month.pop();
        self.stats.rounded_amount_per_month.pop();
        let toml = self.stats.to_toml();
        println!("{}", toml);
        fs::write("site/_data/stats.toml", toml).expect("Unable to w rite file");

        self.script_type.all.pop();
        self.script_type.p2pkh.pop();
        self.script_type.p2pk.pop();
        self.script_type.p2sh.pop();
        self.script_type.v0_p2wpkh.pop();
        self.script_type.v0_p2wsh.pop();
        self.script_type.other.pop();
        let toml = self.script_type.to_toml();
        println!("{}", toml);
        fs::write("site/_data/script_type.toml", toml).expect("Unable to write file");

        println!("ending processer, busy_time: {}", (busy_time / 1_000_000_000) );
    }

    fn process_block(&mut self, block: BlockExtra) {
        let time = block.block.header.time;
        let date = Utc.timestamp(i64::from(time), 0);
        let index = date_index(date);

        let filter = BlockFilter::new_script_filter(&block.block, |o| if let Some(s) = &block.outpoint_values.get(o) {
            Ok(s.script_pubkey.clone())
        } else {
            Err(Error::UtxoMissing(o.clone()))
        }).unwrap();

        self.stats.bip158_filter_size_per_month[index] += filter.content.len() as u64;

        for tx in block.block.txdata {
            for output in tx.output.iter() {
                if output.script_pubkey.is_op_return() {
                    self.process_op_return_script(
                        &output.script_pubkey,
                        time,
                        index,
                        tx_fee(&tx, &block.outpoint_values),
                    );
                }
                self.process_output_script(&output.script_pubkey, index);
                let len = VarInt(output.value).len() as u64;

                self.stats.total_bytes_output_value_bitcoin_varint += len;
                self.stats.total_bytes_output_value_varint +=
                    encoded_length_7bit_varint(output.value);
                let compressed = compress_amount(output.value);
                self.stats
                    .total_bytes_output_value_compressed_bitcoin_varint +=
                    VarInt(compressed).len() as u64;
                self.stats.total_bytes_output_value_compressed_varint +=
                    encoded_length_7bit_varint(compressed);
                if (output.value % 1000) == 0 {
                    self.stats.rounded_amount_per_month[index] += 1;
                    self.stats.rounded_amount += 1;
                }
            }
            for input in tx.input.iter() {
                if block.tx_hashes.contains(&input.previous_output.txid) {
                    self.stats.total_spent_in_block += 1;
                    self.stats.total_spent_in_block_per_month[index] += 1;
                }
            }
            self.process_stats(&tx);
        }
        let hash = block.block.header.bitcoin_hash();
        if self.stats.min_hash > hash {
            self.stats.min_hash = hash;
        }
        let size = u64::from(block.size);
        if self.stats.max_block_size.0 < size {
            self.stats.max_block_size = (size, Some(hash));
        }
    }

    fn process_output_script(&mut self, script: &Script, index: usize) {
        self.script_type.all[index] += 1;
        if script.is_p2pkh() {
            self.script_type.p2pkh[index] += 1;
        } else if script.is_p2pk() {
            self.script_type.p2pk[index] += 1;
        } else if script.is_v0_p2wpkh() {
            self.script_type.v0_p2wpkh[index] += 1;
        } else if script.is_v0_p2wsh() {
            self.script_type.v0_p2wsh[index] += 1;
        } else if script.is_p2sh() {
            self.script_type.p2sh[index] += 1;
        } else {
            self.script_type.other[index] += 1;
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

        *data
            .op_ret_size
            .entry(format!("{:>3}", script_len))
            .or_insert(0) += 1;
        data.op_ret_per_month[index] += 1;
        data.op_ret_fee_per_month[index] += fee;

        if script_len > 4 {
            let op_ret_proto = if script_hex.starts_with("6a4c") && script_hex.len() > 5 {
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
            if op_ret_proto.starts_with("00") && script_len >= 82 {
                data.veriblock_per_month[index] += 1;
                data.veriblock_fee_per_month[index] += fee;
            }
        }
    }

    fn process_stats(&mut self, tx: &Transaction) {
        let weight = tx.get_weight() as u64;
        let outputs = tx.output.len() as u64;
        let inputs = tx.input.len() as u64;
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

        self.stats.amount_over_32 += tx.output.iter().filter(|o| o.value > 0xffff_ffff).count();
    }
}

impl ScriptType {
    fn new() -> Self {
        ScriptType {
            all: vec![0; month_array_len()],
            p2pkh: vec![0; month_array_len()],
            p2pk: vec![0; month_array_len()],
            v0_p2wpkh: vec![0; month_array_len()],
            v0_p2wsh: vec![0; month_array_len()],
            p2sh: vec![0; month_array_len()],
            other: vec![0; month_array_len()],
        }
    }

    fn to_toml(&self) -> String {
        let mut s = String::new();

        s.push_str(&toml_section_vec("all", &self.all, None));
        s.push_str(&toml_section_vec("p2pkh", &self.p2pkh, None));
        s.push_str(&toml_section_vec("p2pk", &self.p2pk, None));
        s.push_str(&toml_section_vec("v0_p2wpkh", &self.v0_p2wpkh, None));
        s.push_str(&toml_section_vec("v0_p2wsh", &self.v0_p2wsh, None));
        s.push_str(&toml_section_vec("p2sh", &self.p2sh, None));
        s.push_str(&toml_section_vec("other", &self.other, None));

        s
    }
}

impl OpReturnData {
    fn new() -> OpReturnData {
        let month_ago = (Utc::now() - Duration::days(30)).timestamp() as u32; // 1 month ago
        let year_ago = (Utc::now() - Duration::days(365)).timestamp() as u32; // 1 year ago
        let len = month_array_len();
        OpReturnData {
            op_ret_per_month: vec![0; len],
            op_ret_size: BTreeMap::new(),
            op_ret_fee_per_month: vec![0; len],
            op_ret_per_proto: HashMap::new(),
            op_ret_per_proto_last_month: HashMap::new(),
            op_ret_per_proto_last_year: HashMap::new(),
            veriblock_per_month: vec![0; len],
            veriblock_fee_per_month: vec![0; len],

            month_ago,
            year_ago,
        }
    }

    fn to_toml(&self) -> String {
        let mut s = String::new();

        s.push_str(&toml_section_vec(
            "op_ret_per_month",
            &self.op_ret_per_month,
                 Some(month_index("201501".to_string()))
        ));
        s.push_str(&toml_section("op_ret_size", &self.op_ret_size));
        s.push_str(&toml_section(
            "op_ret_per_proto",
            &map_by_value(&self.op_ret_per_proto),
        ));
        s.push_str(&toml_section(
            "op_ret_per_proto_last_month",
            &map_by_value(&self.op_ret_per_proto_last_month),
        ));
        s.push_str(&toml_section(
            "op_ret_per_proto_last_year",
            &map_by_value(&self.op_ret_per_proto_last_year),
        ));

        s.push_str(&toml_section_vec_f64(
            "op_ret_fee_per_month",
            &convert_sat_to_bitcoin(&self.op_ret_fee_per_month),
            Some(month_index("201501".to_string()))

        ));

        s.push_str(&toml_section_vec(
            "veriblock_per_month",
            &self.veriblock_per_month.to_vec(),
            Some(month_index("201807".to_string()))

        ));
        s.push_str(&toml_section_vec_f64(
            "veriblock_fee_per_month",
            &convert_sat_to_bitcoin(&self.veriblock_fee_per_month),
            Some(month_index("201807".to_string()))

        ));

        s.push_str("\n[totals]\n");
        let op_ret_fee_total: u64 = self.op_ret_fee_per_month.iter().sum();
        s.push_str(&format!(
            "op_ret_fee = {}\n",
            (op_ret_fee_total as f64 / 100_000_000f64)
        ));
        let veriblock_fee_total: u64 = self.veriblock_fee_per_month.iter().sum();
        s.push_str(&format!(
            "veriblock_fee = {}\n",
            (veriblock_fee_total as f64 / 100_000_000f64)
        ));

        s
    }
}

fn convert_sat_to_bitcoin(map: &Vec<u64>) -> Vec<f64> {
    map.iter().map(|v| *v as f64 / 100_000_000f64).collect()
}

fn toml_section_vec_f64(title: &str, vec: &Vec<f64>, shift: Option<usize>) -> String {
    let mut s = String::new();
    s.push_str(&format!("\n[{}]\n", title));
    let labels: Vec<String> = vec.iter().enumerate().map(|el| index_month(el.0+ shift.unwrap_or(0))).collect();
    s.push_str(&format!("labels={:?}\n", labels));
    s.push_str(&format!("values={:?}\n\n", vec));
    s
}

fn toml_section_vec(title: &str, vec: &Vec<u64>, shift: Option<usize>) -> String {
    let mut s = String::new();
    s.push_str(&format!("\n[{}]\n", title));
    let labels: Vec<String> = vec.iter().enumerate().map(|el| index_month(el.0+ shift.unwrap_or(0) ) ).collect();
    s.push_str(&format!("labels={:?}\n", labels));
    s.push_str(&format!("values={:?}\n\n", vec));
    s
}

fn toml_section(title: &str, map: &BTreeMap<String, u64>) -> String {
    let mut s = String::new();
    s.push_str(&format!("\n[{}]\n", title));
    let labels: Vec<String> = map.keys().cloned().collect();
    s.push_str(&format!("labels={:?}\n", labels));
    let values: Vec<u64> = map.values().cloned().collect();
    s.push_str(&format!("values={:?}\n\n", values));
    s
}

fn map_by_value(map: &HashMap<String, u64>) -> BTreeMap<String, u64> {
    let mut tree: BTreeMap<String, u64> = BTreeMap::new();
    let mut count_vec: Vec<(&String, &u64)> = map.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (key, value) in count_vec.iter().take(10) {
        tree.insert(key.to_string(), **value);
    }
    let other = count_vec.iter().skip(10).fold(0, |acc, x| acc + x.1);
    tree.insert("other".to_owned(), other);
    tree
}

impl Stats {
    fn new() -> Self {
        Stats {
            max_outputs_per_tx: (100u64, None),
            max_inputs_per_tx: (100u64, None),
            min_weight_tx: (10000u64, None),
            max_weight_tx: (0u64, None),
            total_outputs: 0u64,
            total_inputs: 0u64,
            amount_over_32: 0usize,
            rounded_amount: 0u64,
            total_spent_in_block: 0u64,
            max_block_size: (0u64, None),
            min_hash: sha256d::Hash::from_hex(
                "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f",
            )
            .unwrap(),
            total_bytes_output_value_varint: 0u64,
            total_bytes_output_value_compressed_varint: 0u64,
            total_bytes_output_value_bitcoin_varint: 0u64,
            total_bytes_output_value_compressed_bitcoin_varint: 0u64,
            total_spent_in_block_per_month: vec![0; month_array_len()],
            rounded_amount_per_month: vec![0; month_array_len()],
            bip158_filter_size_per_month: vec![0; month_array_len()],
        }
    }

    fn to_toml(&self) -> String {
        let mut s = String::new();

        s.push_str(&toml_section_hash(
            "max_outputs_per_tx",
            &self.max_outputs_per_tx,
        ));
        s.push_str(&toml_section_hash(
            "max_inputs_per_tx",
            &self.max_inputs_per_tx,
        ));
        s.push_str(&toml_section_hash("min_weight_tx", &self.min_weight_tx));
        s.push_str(&toml_section_hash("max_weight_tx", &self.max_weight_tx));
        s.push_str(&toml_section_hash("max_block_size", &self.max_block_size));

        s.push_str("\n[totals]\n");
        s.push_str(&format!("min_hash = \"{:?}\"\n", self.min_hash));
        s.push_str(&format!("outputs = {}\n", self.total_outputs));
        s.push_str(&format!("inputs = {}\n", self.total_inputs));
        s.push_str(&format!("amount_over_32 = {}\n", self.amount_over_32));
        s.push_str(&format!("rounded_amount = {}\n", self.rounded_amount));
        s.push_str(&format!(
            "total_spent_in_block = {}\n",
            self.total_spent_in_block
        ));

        s.push_str(&format!(
            "bytes_output_value = {}\n",
            self.total_outputs * 8
        ));
        s.push_str(&format!(
            "bytes_output_value_bitcoin_varint = {}\n",
            self.total_bytes_output_value_bitcoin_varint
        ));
        s.push_str(&format!(
            "bytes_output_value_varint = {}\n",
            self.total_bytes_output_value_varint
        ));
        s.push_str(&format!(
            "bytes_output_value_compressed_bitcoin_varint = {}\n",
            self.total_bytes_output_value_compressed_bitcoin_varint
        ));
        s.push_str(&format!(
            "bytes_output_value_compressed_varint = {}\n",
            self.total_bytes_output_value_compressed_varint
        ));

        s.push_str("\n\n");
        s.push_str(&toml_section_vec(
            "total_spent_in_block_per_month",
            &self.total_spent_in_block_per_month,
                 None,
        ));

        s.push_str("\n\n");
        s.push_str(&toml_section_vec(
            "rounded_amount_per_month",
            &self.rounded_amount_per_month,
                 None,
        ));

        s.push_str("\n\n");
        s.push_str(&toml_section_vec(
            "bip158_filter_size_per_month",
            &self.bip158_filter_size_per_month,
            None,
        ));

        s
    }
}

fn toml_section_hash(title: &str, value: &(u64, Option<sha256d::Hash>)) -> String {
    let mut s = String::new();
    s.push_str(&format!("\n[{}]\n", title));
    s.push_str(&format!("hash=\"{:?}\"\n", value.1.unwrap()));
    s.push_str(&format!("value={:?}\n\n", value.0));

    s
}

pub fn encoded_length_7bit_varint(mut value: u64) -> u64 {
    let mut bytes = 1;
    loop {
        if value <= 0x7F {
            return bytes;
        }
        bytes += 1;
        value >>= 7;
    }
}

pub fn compress_amount(n: u64) -> u64 {
    let mut n = n;
    if n == 0 {
        return 0;
    }
    let mut e = 0u64;
    loop {
        if (n % 10) != 0 || e >= 9 {
            break;
        }
        n /= 10;
        e += 1;
    }
    if e < 9 {
        let d = n % 10;
        assert!(d >= 1 && d <= 9);
        n /= 10;
        1 + (n * 9 + d - 1) * 10 + e
    } else {
        1 + ((n - 1) * 10) + 9
    }
}

#[cfg(test)]
pub fn decompress_amount(x: u64) -> u64 {
    if x == 0 {
        return 0;
    }
    let mut x = x;
    x -= 1;
    let mut e = x % 10;
    x /= 10;
    let mut n;
    if e < 9 {
        let d = (x % 9) + 1;
        x /= 9;
        n = x * 10 + d;
    } else {
        n = x + 1;
    }
    loop {
        if e == 0 {
            break;
        }
        n *= 10;
        e -= 1;
    }
    n
}

fn date_index(date: DateTime<Utc>) -> usize {
    return (date.year() as usize - 2009) * 12 + (date.month() as usize - 1);
}

fn index_month(index: usize) -> String {
    let year = 2009 + index / 12;
    let month = (index % 12) + 1;
    format!("{:04}{:02}", year, month)
}

fn month_date(yyyymm: String) -> DateTime<Utc> {
    let year: i32 = yyyymm[0..4].parse().unwrap();
    let month: u32 = yyyymm[4..6].parse().unwrap();
    Utc.ymd(year, month, 1).and_hms(0, 0, 0)
}

fn month_index(yyyymm: String) -> usize {
    date_index(month_date(yyyymm))
}

fn month_array_len() -> usize {
    date_index(Utc::now()) + 1
}

#[cfg(test)]
mod test {
    use crate::process::compress_amount;
    use crate::process::decompress_amount;
    use crate::process::index_month;
    use crate::process::{date_index, encoded_length_7bit_varint, month_date};
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
}
