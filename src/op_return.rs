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
        let script_hex = op_return_script.to_string();
        let script_len = op_return_script.len();
        let date = Utc.timestamp(time as i64, 0);
        let ym = format!("{}{:02}", date.year(), date.month());

        *data.op_ret_size.entry(script_len).or_insert(0)+=1;
        *data.op_ret_per_month.entry(ym.clone()).or_insert(0)+=1;

        if script_len > 4 { // 6a = OP_RETURN
            let op_ret_proto = if script_hex.starts_with("6a4c") && script_hex.len() > 5 {  // 4c = OP_PUSHDATA1
                String::from(&script_hex[6..12])
            } else {
                String::from(&script_hex[4..10])
            };
            if time > data.month_ago {
                *data.op_ret_per_proto_last_month.entry(op_ret_proto.clone()).or_insert(0) += 1;
            }
            /*if op_ret_proto.starts_with("0040") {  //veriblock
                tx.fe
            }*/

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
            if time > data.year_ago {
                *data.op_ret_per_proto_last_year.entry(op_ret_proto.clone()).or_insert(0) += 1;
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
                    for output in tx.tx.output {
                        if output.script_pubkey.is_op_return() {
                            self.process( &output.script_pubkey, current_time, &mut data);
                        }
                    }
                },
                _ => {
                    println!("op_return: received {:?}", received);
                    break;
                },
            }
        }
        println!("{:?}", data.op_ret_per_proto_last_month);
        println!("ending op_return processer, wait time: {:?}", wait_time );

    }

}