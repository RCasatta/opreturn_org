use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use crate::{Start, Parsed};
use std::collections::HashMap;
use time::Duration;
use chrono::{Utc, TimeZone, Datelike, DateTime};
use bitcoin::Script;

struct OpReturnData {
    op_ret_per_month: HashMap<String, u32>,
    op_ret_per_proto: HashMap<String, u32>,
    op_ret_per_proto_last_month: HashMap<String, u32>,
    op_ret_per_proto_last_year: HashMap<String, u32>,
    op_ret_size: HashMap<usize, u32>,
    month_ago: u32,
    year_ago: u32,
}


impl OpReturnData {
    fn new() -> OpReturnData {
        let month_ago = (Utc::now() - Duration::days(30)).timestamp() as u32; // 1 month ago
        let year_ago = (Utc::now() - Duration::days(365)).timestamp() as u32; // 1 year ago
        OpReturnData {
            op_ret_per_month : HashMap::new(),
            op_ret_per_proto : HashMap::new(),
            op_ret_per_proto_last_month : HashMap::new(),
            op_ret_per_proto_last_year : HashMap::new(),
            op_ret_size : HashMap::new(),
            month_ago,
            year_ago,
        }
    }
}

pub struct OpReturn {
    sender : Sender<Option<Parsed>>,
    receiver : Receiver<Option<Parsed>>,
}

impl OpReturn {
    pub fn new() -> OpReturn {
        let (sender, receiver) = channel();
        OpReturn {
            sender,
            receiver,
        }
    }

    fn process(&self, op_return_script : &Script, time : u32, data : &mut OpReturnData) {
        let script_hex = op_return_script.to_string();
        if script_hex.starts_with("6a") && script_hex.len() > 9 { // 6a = OP_RETURN
            let op_ret_proto = if script_hex.starts_with("6a4c") && script_hex.len() > 11 {  // 4c = OP_PUSHDATA1
                String::from(&script_hex[6..12])
            } else {
                String::from(&script_hex[4..10])
            };
            if time > data.month_ago {
                *data.op_ret_per_proto_last_month.entry(op_ret_proto.clone()).or_insert(0) += 1;
            }
        }
        /*if let Some(op_ret_proto) = parsed.op_ret_proto {
                        if parsed.is_last_month {
                            *maps.op_ret_per_proto_last_month.entry(op_ret_proto.clone()).or_insert(0) += 1;
                        }
                        if parsed.is_last_year {
                            *maps.op_ret_per_proto_last_year.entry(op_ret_proto.clone()).or_insert(0) += 1;
                        }
                        *maps.op_ret_per_month.entry(parsed.ym.clone()).or_insert(0)+=1;
                        *maps.op_ret_per_proto.entry(op_ret_proto).or_insert(0)+=1;
                        *maps.op_ret_size.entry(parsed.script_size).or_insert(0)+=1;
                    }*/
    }
}

impl Start for OpReturn {
    fn start(&self) {
        println!("starting op_return processer");
        let mut data = OpReturnData::new();
        loop {
            let received = self.receiver.recv().unwrap();
            match received {
                Some(received) => {
                    for output in received.tx.output {
                        if output.script_pubkey.is_op_return() {
                            self.process(&output.script_pubkey, received.block_header.time, &mut data);
                        }
                    }
                },
                None => break,
            }
        }
        println!("ending op_return processer");
    }

    fn get_sender(&self) -> Sender<Option<Parsed>> {
        self.sender.clone()
    }
}