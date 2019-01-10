use crate::parse::{TxOrBlock, BlockParsed};
use crate::{Startable};
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::time::Instant;
use std::time::Duration;

pub struct Stats {
    sender : Sender<TxOrBlock>,
    receiver : Receiver<TxOrBlock>,
}

impl Stats {
    pub fn new() -> Stats {
        let (sender, receiver) = channel();
        Stats {
            sender,
            receiver,
        }
    }
    pub fn get_sender(&self) -> Sender<TxOrBlock> {
        self.sender.clone()
    }
}

impl Startable for Stats {
    fn start(&self) {
        println!("starting Stats processer");
        let mut max_outputs_per_tx = 100usize;
        let mut max_inputs_per_tx = 100usize;
        let mut min_weight_tx = 10000u64;
        let mut max_weight_tx = 0u64;
        let mut total_outputs = 0u64;
        let mut total_inputs = 0u64;
        let mut amount_over_32 = 0usize;
        let mut _current_block : Option<BlockParsed> = None;
        let mut wait_time =  Duration::from_secs(0);

        loop {
            let instant = Instant::now();
            let received = self.receiver.recv().expect("can't receive in stats");
            wait_time += instant.elapsed();
            match received {
                TxOrBlock::Block(block) => {
                    _current_block = Some(block);
                },
                TxOrBlock::Tx(tx) => {
                    let tx = tx.tx;
                    let weight = tx.get_weight();
                    let outputs = tx.output.len();
                    let inputs = tx.input.len();
                    total_outputs += outputs as u64;
                    total_inputs += inputs as u64;
                    if max_outputs_per_tx < outputs {
                        max_outputs_per_tx = outputs;
                        println!("max_outputs_per_tx is {} for {}", max_outputs_per_tx, tx.txid());
                    }
                    if max_inputs_per_tx < inputs {
                        max_inputs_per_tx = inputs;
                        println!("max_inputs_per_tx is {} for {}", max_inputs_per_tx, tx.txid());
                    }
                    if max_weight_tx < weight {
                        max_weight_tx = weight;
                        println!("max_weight_tx is {} for {}", max_weight_tx, tx.txid());
                    }
                    if min_weight_tx > weight {
                        min_weight_tx = weight;
                        println!("min_weight_tx is {} for {}", min_weight_tx, tx.txid());
                    }
                    let over_32 = tx.output.iter().filter(|o| o.value > 0xffffffff).count();
                    if over_32 > 0 {
                        amount_over_32 += over_32;
                    }
                },
                _ => {
                    println!("stats: received {:?}", received);
                    break;
                },
            }
        }
        println!("amount_over_32: {}", amount_over_32);
        println!("total_outputs: {}", total_outputs);
        println!("total_inputs: {}", total_inputs);
        println!("ending Stats processer, wait time: {:?}", wait_time );
    }
}

/*
fn trunc(outpoint : &OutPoint) -> Vec<u8> {
    serialize(outpoint)[26..].to_vec()
}
*/

#[cfg(test)]
mod test {
    use bitcoin::OutPoint;
    use bitcoin::consensus::serialize;

    #[test]
    fn test() {
        println!("{}", serialize(&OutPoint::default())[28..].len());
    }

    #[test]
    fn test_map() {
        println!("{}", serialize(&OutPoint::default())[28..].len());
    }

}