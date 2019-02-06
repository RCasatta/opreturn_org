use std::sync::mpsc::Receiver;
use crate::BlockExtra;
use std::collections::BTreeMap;
use std::collections::HashMap;
use chrono::{Utc, TimeZone, Datelike};
use time::Duration;
use bitcoin::Script;
use std::fs;
use crate::fee::tx_fee;

pub struct Process {
    receiver : Receiver<Option<BlockExtra>>,
    op_return_data: OpReturnData,
}

impl Process {
    pub fn new(receiver : Receiver<Option<BlockExtra>> ) -> Process {
        Process {
            receiver,
            op_return_data: OpReturnData::new(),
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

        let toml = self.op_return_data.to_toml();
        println!("{}", toml);
        fs::write("outputs/op_return.toml", toml).expect("Unable to write file");

        println!("ending processer");
    }

    fn process_block(&mut self, block : BlockExtra) {
        for tx in block.block.txdata {
            let txid = tx.txid();
            for output in tx.output.iter() {
                if output.script_pubkey.is_op_return() {
                    self.process_script( &output.script_pubkey, block.block.header.time, tx_fee(&tx,&block.outpoint_values));
                    if output.script_pubkey.len() > 100 {
                        println!("len {} greater than 100 {}", output.script_pubkey.len(), txid );
                    }
                }
            }
        }
    }

    fn process_script(&mut self, op_return_script : &Script, time : u32, fee : u64) {
        let script_bytes = op_return_script.as_bytes();
        let script_hex = hex::encode(script_bytes);
        let script_len = script_bytes.len();
        let date = Utc.timestamp(time as i64, 0);
        let ym = format!("{}{:02}", date.year(), date.month());
        let data = &mut self.op_return_data;

        *data.op_ret_size.entry(format!("{:>3}",script_len)).or_insert(0)+=1;
        *data.op_ret_per_month.entry(ym.clone()).or_insert(0)+=1;
        *data.op_ret_fee_per_month.entry(ym.clone()).or_insert(0)+=fee;

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
            *data.op_ret_per_proto.entry(op_ret_proto.clone()).or_insert(0)+=1;
            if op_ret_proto.starts_with("0004") {
                *data.veriblock_per_month.entry(ym.clone()).or_insert(0) += 1;
            }
        }
    }

}

struct OpReturnData {
    op_ret_per_month: BTreeMap<String, u32>,
    op_ret_size: BTreeMap<String, u32>,  //pad with spaces usize of len up to 3
    veriblock_per_month : BTreeMap<String,u32>,
    op_ret_fee_per_month: BTreeMap<String, u64>,

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
        s.push_str( &toml_section_u64("op_ret_fee_per_month", &self.op_ret_fee_per_month) );

        s
    }
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


fn toml_section_u64(title : &str, map : &BTreeMap<String, u64>) -> String {
    let mut s = String::new();
    s.push_str(&format!("\n[{}]\n", title ));
    let labels : Vec<String> = map.keys().cloned().collect();
    s.push_str(&format!("labels={:?}\n", labels) );
    let values : Vec<u64> = map.values().cloned().collect();
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

