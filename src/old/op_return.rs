use crate::parse::TxOrBlock;
use crate::Startable;
use std::collections::HashMap;
use time::Duration;
use chrono::{Utc, TimeZone, Datelike};
use bitcoin::Script;
use std::time::Instant;
use std::time::Duration as StdDur;
use std::sync::mpsc::sync_channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::collections::BTreeMap;
use std::fs;


struct OpReturnData {
    op_ret_per_month: BTreeMap<String, u32>,
    op_ret_size: BTreeMap<String, u32>,  //pad with spaces usize of len up to 3

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
            op_ret_per_proto : HashMap::new(),
            op_ret_per_proto_last_month : HashMap::new(),
            op_ret_per_proto_last_year : HashMap::new(),
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


pub struct OpReturn {
    sender : SyncSender<TxOrBlock>,
    receiver : Receiver<TxOrBlock>,
}

impl OpReturn {
    pub fn new() -> OpReturn {
        let (sender, receiver) = sync_channel(1000);
        OpReturn {
            sender,
            receiver,
        }
    }

    fn process(&self, op_return_script : &Script, time : u32, data : &mut OpReturnData) {
        let script_bytes = op_return_script.as_bytes();
        let script_hex = hex::encode(script_bytes);
        let script_len = script_bytes.len();
        let date = Utc.timestamp(time as i64, 0);
        let ym = format!("{}{:02}", date.year(), date.month());

        *data.op_ret_size.entry(format!("{:>3}",script_len)).or_insert(0)+=1;
        *data.op_ret_per_month.entry(ym.clone()).or_insert(0)+=1;

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
            *data.op_ret_per_proto.entry(op_ret_proto).or_insert(0)+=1;

        }
    }

    pub fn get_sender(&self) -> SyncSender<TxOrBlock> {
        self.sender.clone()
    }

}

impl Startable for OpReturn {
    fn start(&self) {
        println!("starting op_return processer");
        let mut data = OpReturnData::new();
        let mut current_time = 0u32;
        let mut wait_time =  StdDur::from_secs(0);
        loop {
            let instant = Instant::now();
            let received = self.receiver.recv().expect("can't receive in op_return");
            wait_time += instant.elapsed();
            match received {
                TxOrBlock::Block(block) => {
                    current_time = block.block_header.time;
                },
                TxOrBlock::Tx(tx) => {
                    let txid = tx.tx.txid();
                    for output in tx.tx.output {
                        if output.script_pubkey.is_op_return() {
                            self.process( &output.script_pubkey, current_time, &mut data);
                            if output.script_pubkey.len() > 100 {
                                println!("len {} greater than 100 {}", output.script_pubkey.len(), txid );
                            }
                        }
                    }
                },
                _ => {
                    println!("op_return: received {:?}", received);
                    break;
                },
            }
        }

        //remove current month
        let now = Utc::now();
        let current_ym = format!("{}{:02}", now.year(), now.month());
        data.op_ret_per_month.remove(&current_ym);

        let toml = data.to_toml();
        println!("{}", toml);
        fs::write("op_return.toml", toml).expect("Unable to write file");

        println!("ending op_return processer, wait time: {:?}", wait_time );
    }
}



#[cfg(test)]
mod test {
    use toml;
    use std::collections::HashMap;

    #[test]
    fn test2() {
        let tuples = vec![("one", 1), ("two", 2), ("three", 3)];
        let m: HashMap<_, _> = tuples.into_iter().collect();
        println!("{:?}", m);
    }

    #[test]
    fn test() {
        let mut c = HashMap::new();

        let mut b = HashMap::new();
        let a = vec![2,3];
        b.insert("a", a);
        c.insert("c", b);
        println!("{}",toml::to_string(&c).unwrap() );
    }
}