use std::sync::mpsc::Receiver;
use crate::BlockExtra;
use std::collections::BTreeMap;
use std::collections::HashMap;
use chrono::{Utc, TimeZone, Datelike};
use time::Duration;
use bitcoin::Script;
use std::fs;
use crate::fee::tx_fee;
use bitcoin::Transaction;
use bitcoin::util::hash::BitcoinHash;
use bitcoin_hashes::sha256d;
use bitcoin_hashes::hex::FromHex;
use std::collections::HashSet;
use bitcoin::VarInt;

pub struct Process {
    receiver : Receiver<Option<BlockExtra>>,
    op_return_data: OpReturnData,
    stats: Stats,
    script_type: ScriptType,
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
        loop {
            let received = self.receiver.recv().expect("cannot receive fee block");
            match received {
                Some(block) => {
                    self.process_block(block);
                },
                None => break,
            }
        }

        //remove current month
        let now = Utc::now();
        let current_ym = format!("{}{:02}", now.year(), now.month());
        self.op_return_data.op_ret_per_month.remove(&current_ym);
        self.op_return_data.veriblock_per_month.remove(&current_ym);
        self.op_return_data.veriblock_fee_per_month.remove(&current_ym);
        self.op_return_data.op_ret_fee_per_month.remove(&current_ym);

        let toml = self.op_return_data.to_toml();
        println!("{}", toml);
        fs::write("site/_data/op_return.toml", toml).expect("Unable to write file");

        let toml = self.stats.to_toml();
        println!("{}", toml);
        fs::write("site/_data/stats.toml", toml).expect("Unable to w rite file");

        self.script_type.all.remove(&current_ym);
        self.script_type.p2pkh.remove(&current_ym);
        self.script_type.p2pk.remove(&current_ym);
        self.script_type.p2sh.remove(&current_ym);
        self.script_type.v0_p2wpkh.remove(&current_ym);
        self.script_type.v0_p2wsh.remove(&current_ym);
        align(&mut self.script_type.all, &mut self.script_type.p2pkh);
        align(&mut self.script_type.all, &mut self.script_type.p2pk);
        align(&mut self.script_type.all, &mut self.script_type.p2sh);
        align(&mut self.script_type.all, &mut self.script_type.v0_p2wpkh);
        align(&mut self.script_type.all, &mut self.script_type.v0_p2wsh);
        let toml = self.script_type.to_toml();
        println!("{}", toml);
        fs::write("site/_data/script_type.toml", toml).expect("Unable to write file");

        println!("ending processer");
    }

    fn process_block(&mut self, block: BlockExtra) {
        let tx_hashes: HashSet<sha256d::Hash> = block.block.txdata.iter().map(|tx| tx.txid() ).collect();
        for tx in block.block.txdata {
            for output in tx.output.iter() {
                if output.script_pubkey.is_op_return() {
                    self.process_op_return_script(&output.script_pubkey, block.block.header.time, tx_fee(&tx, &block.outpoint_values));
                }
                self.process_output_script(&output.script_pubkey, block.block.header.time);

                self.stats.total_bytes_output_value_bitcoin_varint += VarInt(output.value).encoded_length();
                self.stats.total_bytes_output_value_varint += encoded_length_7bit_varint(output.value);
            }
            for input in tx.input.iter() {
                if tx_hashes.contains(&input.previous_output.txid) {
                    self.stats.total_spent_in_block += 1;
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

    fn process_output_script(&mut self, script: &Script, time: u32) {
        let date = Utc.timestamp(i64::from(time), 0);
        let ym = format!("{}{:02}", date.year(), date.month());
        *self.script_type.all.entry(ym.clone()).or_insert(0) += 1;
        if script.is_p2pkh() {
            *self.script_type.p2pkh.entry(ym).or_insert(0) += 1;
        } else if script.is_p2pk() {
            *self.script_type.p2pk.entry(ym).or_insert(0) += 1;
        } else if script.is_v0_p2wpkh() {
            *self.script_type.v0_p2wpkh.entry(ym).or_insert(0) += 1;
        } else if script.is_v0_p2wsh() {
            *self.script_type.v0_p2wsh.entry(ym).or_insert(0) += 1;
        } else if script.is_p2sh() {
            *self.script_type.p2sh.entry(ym).or_insert(0) += 1;
        }
    }

    fn process_op_return_script(&mut self, op_return_script: &Script, time: u32, fee: u64) {
        let script_bytes = op_return_script.as_bytes();
        let script_hex = hex::encode(script_bytes);
        let script_len = script_bytes.len();
        let date = Utc.timestamp(i64::from(time), 0);
        let ym = format!("{}{:02}", date.year(), date.month());
        let data = &mut self.op_return_data;

        *data.op_ret_size.entry(format!("{:>3}", script_len)).or_insert(0) += 1;
        *data.op_ret_per_month.entry(ym.clone()).or_insert(0) += 1;
        *data.op_ret_fee_per_month.entry(ym.clone()).or_insert(0) += fee;

        if script_len > 4 {
            let op_ret_proto = if script_hex.starts_with("6a4c") && script_hex.len() > 5 {  // 4c = OP_PUSHDATA1
                String::from(&script_hex[6..12])
            } else {
                String::from(&script_hex[4..10])
            };
            if time > data.year_ago {
                *data.op_ret_per_proto_last_year.entry(op_ret_proto.clone()).or_insert(0) += 1;

                if time > data.month_ago {
                    *data.op_ret_per_proto_last_month.entry(op_ret_proto.clone()).or_insert(0) += 1;
                }
            }
            *data.op_ret_per_proto.entry(op_ret_proto.clone()).or_insert(0) += 1;
            if op_ret_proto.starts_with("00") && script_len >= 82 {
                *data.veriblock_per_month.entry(ym.clone()).or_insert(0) += 1;
                *data.veriblock_fee_per_month.entry(ym.clone()).or_insert(0) += fee;
            }
        }
    }

    fn process_stats(&mut self, tx: &Transaction) {
        let weight = tx.get_weight();
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
        let over_32 = tx.output.iter().filter(|o| o.value > 0xffff_ffff).count();
        if over_32 > 0 {
            self.stats.amount_over_32 += over_32;
        }
    }
}

struct ScriptType {
    all: BTreeMap<String, u64>,
    p2pkh: BTreeMap<String, u64>,
    p2pk: BTreeMap<String, u64>,
    v0_p2wpkh: BTreeMap<String, u64>,
    v0_p2wsh: BTreeMap<String, u64>,
    p2sh: BTreeMap<String, u64>,
}

impl ScriptType {
    fn new() -> Self {
        ScriptType {
            all : BTreeMap::new(),
            p2pkh : BTreeMap::new(),
            p2pk : BTreeMap::new(),
            v0_p2wpkh : BTreeMap::new(),
            v0_p2wsh : BTreeMap::new(),
            p2sh : BTreeMap::new(),
        }
    }

    fn to_toml(&self) -> String {
        let mut s = String::new();

        s.push_str( &toml_section("all", &self.all));
        s.push_str( &toml_section("p2pkh", &self.p2pkh));
        s.push_str( &toml_section("p2pk", &self.p2pk));
        s.push_str( &toml_section("v0_p2wpkh", &self.v0_p2wpkh));
        s.push_str( &toml_section("v0_p2wsh", &self.v0_p2wsh));
        s.push_str( &toml_section("p2sh", &self.p2sh));

        s

    }
}

struct OpReturnData {
    op_ret_per_month: BTreeMap<String, u64>,
    op_ret_size: BTreeMap<String, u64>,  //pad with spaces usize of len up to 3
    veriblock_per_month : BTreeMap<String,u64>,
    op_ret_fee_per_month: BTreeMap<String, u64>,
    veriblock_fee_per_month: BTreeMap<String, u64>,

    op_ret_per_proto: HashMap<String, u64>,
    op_ret_per_proto_last_month: HashMap<String, u64>,
    op_ret_per_proto_last_year: HashMap<String, u64>,

    month_ago: u32,
    year_ago: u32,
}

impl OpReturnData {
    fn new() -> OpReturnData {
        let month_ago = (Utc::now() - Duration::days(30)).timestamp() as u32; // 1 month ago
        let year_ago = (Utc::now() - Duration::days(365)).timestamp() as u32; // 1 year ago
        OpReturnData {
            op_ret_per_month : BTreeMap::new(),
            op_ret_size : BTreeMap::new(),
            op_ret_fee_per_month : BTreeMap::new(),
            op_ret_per_proto : HashMap::new(),
            op_ret_per_proto_last_month : HashMap::new(),
            op_ret_per_proto_last_year : HashMap::new(),
            veriblock_per_month : BTreeMap::new(),
            veriblock_fee_per_month : BTreeMap::new(),

            month_ago,
            year_ago,
        }
    }

    fn to_toml(&self) -> String {
        let mut s = String::new();

        s.push_str( &toml_section("op_ret_per_month", &self.op_ret_per_month) );
        s.push_str( &toml_section("op_ret_size", &self.op_ret_size) );
        s.push_str( &toml_section("op_ret_per_proto", &map_by_value(&self.op_ret_per_proto)) );
        s.push_str( &toml_section("op_ret_per_proto_last_month", &map_by_value(&self.op_ret_per_proto_last_month)) );
        s.push_str( &toml_section("op_ret_per_proto_last_year", &map_by_value(&self.op_ret_per_proto_last_year)) );

        s.push_str( &toml_section_f64("op_ret_fee_per_month", &convert_sat_to_bitcoin(&self.op_ret_fee_per_month) ));

        s.push_str( &toml_section("veriblock_per_month", &keep_from("201801".to_string(),&self.veriblock_per_month) ) );
        s.push_str( &toml_section_f64("veriblock_fee_per_month", &convert_sat_to_bitcoin(&keep_from("201801".to_string(),&self.veriblock_fee_per_month) )) );

        s.push_str("\n[totals]\n");
        let op_ret_fee_total : u64 = self.op_ret_fee_per_month.iter().map(|(_k,v)| v).sum();
        s.push_str(&format!("op_ret_fee = {}\n", (op_ret_fee_total as f64 / 100_000_000f64)));
        let veriblock_fee_total : u64 = self.veriblock_fee_per_month.iter().map(|(_k,v)| v).sum();
        s.push_str(&format!("veriblock_fee = {}\n", (veriblock_fee_total as f64 / 100_000_000f64)));

        s
    }
}

fn keep_from(yyyymm : String, map : &BTreeMap<String, u64>) -> BTreeMap<String, u64>{
    map.clone().into_iter().skip_while(|(k,_)| *k < yyyymm).collect()
}

fn convert_sat_to_bitcoin( map : &BTreeMap<String, u64>) ->  BTreeMap<String, f64> {
    map.iter().map(|(k,v)| (k.to_string(), (*v as f64 / 100_000_000f64) )).collect()
}

fn toml_section(title : &str, map : &BTreeMap<String, u64>) -> String {
    let mut s = String::new();
    s.push_str(&format!("\n[{}]\n", title ));
    let labels : Vec<String> = map.keys().cloned().collect();
    s.push_str(&format!("labels={:?}\n", labels) );
    let values : Vec<u64> = map.values().cloned().collect();
    s.push_str(&format!("values={:?}\n\n", values ) );
    s
}


fn toml_section_f64(title : &str, map : &BTreeMap<String, f64>) -> String {
    let mut s = String::new();
    s.push_str(&format!("\n[{}]\n", title ));
    let labels : Vec<String> = map.keys().cloned().collect();
    s.push_str(&format!("labels={:?}\n", labels) );
    let values : Vec<f64> = map.values().cloned().collect();
    s.push_str(&format!("values={:?}\n\n", values ) );
    s
}

fn map_by_value(map : &HashMap<String,u64>) -> BTreeMap<String,u64> {
    let mut tree : BTreeMap<String, u64> = BTreeMap::new();
    let mut count_vec: Vec<(&String, &u64)> = map.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (key,value) in count_vec.iter().take(10) {
        tree.insert(key.to_string(),**value);
    }
    let other = count_vec.iter().skip(10).fold(0, |acc, x| acc + x.1);
    tree.insert("other".to_owned(), other);
    tree
}

fn align (map1 : &mut BTreeMap<String,u64>, map2 : &mut BTreeMap<String,u64>) {
    for key in map1.keys() {
        if map2.get(key).is_none() {
            map2.insert(key.to_owned(),0);
        }
    }

    for key in map2.keys() {
        if map1.get(key).is_none() {
            map1.insert(key.to_owned(),0);
        }
    }
}

struct Stats {
    max_outputs_per_tx : (u64, Option<sha256d::Hash>),
    min_weight_tx : (u64, Option<sha256d::Hash>),
    max_inputs_per_tx : (u64, Option<sha256d::Hash>),
    max_weight_tx : (u64, Option<sha256d::Hash>),
    total_outputs : u64,
    total_inputs : u64,
    total_spent_in_block : u64,
    amount_over_32 : usize,
    max_block_size: (u64, Option<sha256d::Hash>),
    min_hash : sha256d::Hash,
    total_bytes_output_value_varint : u64,
    total_bytes_output_value_bitcoin_varint : u64,
}

impl Stats {
    fn new() -> Self {
        Stats {
            max_outputs_per_tx: (100u64, None),
            max_inputs_per_tx: (100u64, None),
            min_weight_tx : (10000u64, None),
            max_weight_tx : (0u64, None),
            total_outputs: 0u64,
            total_inputs: 0u64,
            amount_over_32: 0usize,
            total_spent_in_block: 0u64,
            max_block_size : (0u64, None),
            min_hash: sha256d::Hash::from_hex("000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f").unwrap(),
            total_bytes_output_value_varint: 0u64,
            total_bytes_output_value_bitcoin_varint: 0u64,
        }
    }

    fn to_toml(&self) -> String {
        let mut s = String::new();

        s.push_str(&toml_section_hash("max_outputs_per_tx",&self.max_outputs_per_tx));
        s.push_str(&toml_section_hash("max_inputs_per_tx",&self.max_inputs_per_tx));
        s.push_str(&toml_section_hash("min_weight_tx",&self.min_weight_tx));
        s.push_str(&toml_section_hash("max_weight_tx",&self.max_weight_tx));
        s.push_str(&toml_section_hash("max_block_size",&self.max_block_size));

        s.push_str("\n[totals]\n");
        s.push_str(&format!("min_hash = \"{:?}\"\n", self.min_hash));
        s.push_str(&format!("outputs = {}\n", self.total_outputs));
        s.push_str(&format!("inputs = {}\n", self.total_inputs));
        s.push_str(&format!("amount_over_32 = {}\n", self.amount_over_32));
        s.push_str(&format!("total_spent_in_block = {}\n", self.total_spent_in_block));

        s.push_str(&format!("bytes_output_value = {}\n", self.total_outputs*8));
        s.push_str(&format!("bytes_output_value_bitcoin_varint = {}\n", self.total_bytes_output_value_bitcoin_varint));
        s.push_str(&format!("bytes_output_value_varint = {}\n", self.total_bytes_output_value_varint));

        s
    }

}

fn toml_section_hash(title : &str, value : &(u64,Option<sha256d::Hash>)) -> String {
    let mut s = String::new();
    s.push_str(&format!("\n[{}]\n", title ));
    s.push_str(&format!("hash=\"{:?}\"\n", value.1.unwrap() ) );
    s.push_str(&format!("value={:?}\n\n", value.0 ) );

    s
}

pub fn encoded_length_7bit_varint(mut value: u64) -> u64 {
    let mut bytes = 1;
    loop {
        if value <=  0x7F {
            return bytes;
        }
        bytes += 1;
        value >>= 7;
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use crate::process::encoded_length_7bit_varint;

    #[test]
    fn test1() {
        assert_eq!( encoded_length_7bit_varint(127), 1);
        assert_eq!( encoded_length_7bit_varint(128), 2);
        assert_eq!( encoded_length_7bit_varint(1_270), 2);
        assert_eq!( encoded_length_7bit_varint(111_270), 3);
        assert_eq!( encoded_length_7bit_varint(2_097_151), 3);
        assert_eq!( encoded_length_7bit_varint(2_097_152), 4);
    }

    #[test]
    fn test2() {
        let tuples = vec![("one", 1), ("two", 2), ("three", 3)];
        let m: HashMap<_, _> = tuples.into_iter().collect();
        println!("{:?}", m);
    }

    #[test]
    fn test3() {
        let mut b : bool = true;
        let u = b as u32;
        assert_eq!(u,1u32);
        b = false;
        let u = b as u32;
        assert_eq!(u,0u32);
    }
}
