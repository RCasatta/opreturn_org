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

pub struct Process {
    receiver : Receiver<Option<BlockExtra>>,
    op_return_data: OpReturnData,
    stats: Stats,
}

impl Process {
    pub fn new(receiver: Receiver<Option<BlockExtra>>) -> Process {
        Process {
            receiver,
            op_return_data: OpReturnData::new(),
            stats: Stats::new(),
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
        self.op_return_data.op_ret_fee_per_month.remove(&current_ym);

        let toml = self.op_return_data.to_toml();
        println!("{}", toml);
        fs::write("site/_data/op_return.toml", toml).expect("Unable to write file");

        let toml = self.stats.to_toml();
        println!("{}", toml);
        fs::write("site/_data/stats.toml", toml).expect("Unable to write file");

        println!("ending processer");
    }

    fn process_block(&mut self, block: BlockExtra) {
        for tx in block.block.txdata {
            let txid = tx.txid();
            for output in tx.output.iter() {
                if output.script_pubkey.is_op_return() {
                    self.process_script(&output.script_pubkey, block.block.header.time, tx_fee(&tx, &block.outpoint_values));
                    if output.script_pubkey.len() > 100 {
                        println!("len {} greater than 100 {}", output.script_pubkey.len(), txid);
                    }
                }
            }
            self.process_stats(&tx);
        }
    }

    fn process_script(&mut self, op_return_script: &Script, time: u32, fee: u64) {
        let script_bytes = op_return_script.as_bytes();
        let script_hex = hex::encode(script_bytes);
        let script_len = script_bytes.len();
        let date = Utc.timestamp(time as i64, 0);
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
            if op_ret_proto.starts_with("0004") {
                *data.veriblock_per_month.entry(ym.clone()).or_insert(0) += 1;
                *data.veriblock_fee_per_month.entry(ym.clone()).or_insert(0) += fee;
            }
        }
    }

    fn process_stats(&mut self, tx: &Transaction) {
        let weight = tx.get_weight();
        let outputs = tx.output.len() as u64;
        let inputs = tx.input.len() as u64;
        self.stats.total_outputs += outputs as u64;
        self.stats.total_inputs += inputs as u64;
        if self.stats.max_outputs_per_tx.0 < outputs {
            self.stats.max_outputs_per_tx = (outputs, Some(tx.clone()));
        }
        if self.stats.max_inputs_per_tx.0 < inputs {
            self.stats.max_inputs_per_tx = (inputs, Some(tx.clone()));
        }
        if self.stats.max_weight_tx.0 < weight {
            self.stats.max_weight_tx = (weight, Some(tx.clone()));
        }
        if self.stats.min_weight_tx.0 > weight {
            self.stats.min_weight_tx = (weight, Some(tx.clone()));
        }
        let over_32 = tx.output.iter().filter(|o| o.value > 0xffffffff).count();
        if over_32 > 0 {
            self.stats.amount_over_32 += over_32;
        }
    }
}

struct OpReturnData {
    op_ret_per_month: BTreeMap<String, u32>,
    op_ret_size: BTreeMap<String, u32>,  //pad with spaces usize of len up to 3
    veriblock_per_month : BTreeMap<String,u32>,
    op_ret_fee_per_month: BTreeMap<String, u64>,
    veriblock_fee_per_month: BTreeMap<String, u64>,

    op_ret_per_proto: HashMap<String, u32>,
    op_ret_per_proto_last_month: HashMap<String, u32>,
    op_ret_per_proto_last_year: HashMap<String, u32>,

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
        s.push_str( &toml_section("veriblock_per_month", &self.veriblock_per_month) );
        s.push_str( &toml_section_f64("op_ret_fee_per_month", &convert_sat_to_bitcoin(&self.op_ret_fee_per_month) ));
        s.push_str( &toml_section_f64("veriblock_fee_per_month", &convert_sat_to_bitcoin(&self.veriblock_fee_per_month) ));

        let op_ret_fee_total : u64 = self.op_ret_fee_per_month.iter().map(|(_k,v)| v).sum();
        s.push_str(&format!("op_ret_fee_per_month = {}", (op_ret_fee_total as f64 / 100_000_000f64)));

        let veriblock_fee_total : u64 = self.veriblock_fee_per_month.iter().map(|(_k,v)| v).sum();
        s.push_str(&format!("veriblock_fee_total = {}", (veriblock_fee_total as f64 / 100_000_000f64)));

        s
    }
}

fn convert_sat_to_bitcoin( map : &BTreeMap<String, u64>) ->  BTreeMap<String, f64> {
    map.iter().map(|(k,v)| (k.to_string(), (*v as f64 / 100_000_000f64) )).into_iter().collect()
}

fn toml_section(title : &str, map : &BTreeMap<String, u32>) -> String {
    let mut s = String::new();
    s.push_str(&format!("\n[{}]\n", title ));
    let labels : Vec<String> = map.keys().cloned().collect();
    s.push_str(&format!("labels={:?}\n", labels) );
    let values : Vec<u32> = map.values().cloned().collect();
    s.push_str(&format!("values={:?}\n", values ) );
    s
}


fn toml_section_f64(title : &str, map : &BTreeMap<String, f64>) -> String {
    let mut s = String::new();
    s.push_str(&format!("\n[{}]\n", title ));
    let labels : Vec<String> = map.keys().cloned().collect();
    s.push_str(&format!("labels={:?}\n", labels) );
    let values : Vec<f64> = map.values().cloned().collect();
    s.push_str(&format!("values={:?}\n", values ) );
    s
}

fn map_by_value(map : &HashMap<String,u32>) -> BTreeMap<String,u32> {
    let mut tree : BTreeMap<String, u32> = BTreeMap::new();
    let mut count_vec: Vec<(&String, &u32)> = map.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (key,value) in count_vec.iter().take(10) {
        tree.insert(key.to_string(),**value);
    }
    let other = count_vec.iter().skip(10).fold(0, |acc, x| acc + x.1);
    tree.insert("other".to_owned(), other);
    tree
}

struct Stats {
    max_outputs_per_tx : (u64, Option<Transaction>),
    min_weight_tx : (u64, Option<Transaction>),
    max_inputs_per_tx : (u64, Option<Transaction>),
    max_weight_tx : (u64, Option<Transaction>),
    total_outputs : u64,
    total_inputs : u64,
    amount_over_32 : usize,
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
        }
    }

    fn to_toml(&self) -> String {
        let mut s = String::new();

        s.push_str(&toml_section_tx("max_outputs_per_tx",&self.max_outputs_per_tx));
        s.push_str(&format!("total_outputs = {}", self.total_outputs));
        s.push_str(&format!("total_outputs = {}", self.total_inputs));
        s.push_str(&format!("total_outputs = {}", self.amount_over_32));

        s
    }

}

fn toml_section_tx(title : &str, value : &(u64,Option<Transaction>)) -> String {
    let mut s = String::new();
    s.push_str(&format!("\n[{}]\n", title ));
    s.push_str(&format!("hash={:?}\n", value.0 ) );
    s.push_str(&format!("value={:?}\n", value.1.clone().unwrap().txid() ) );
    s
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    #[test]
    fn test2() {
        let tuples = vec![("one", 1), ("two", 2), ("three", 3)];
        let m: HashMap<_, _> = tuples.into_iter().collect();
        println!("{:?}", m);
    }
}
