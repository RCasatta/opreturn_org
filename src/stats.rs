use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use crate::{Start, Parsed};
use bitcoin::consensus::serialize;
use std::collections::HashMap;
use bitcoin::OutPoint;

pub struct Stats {
    sender : Sender<Option<Parsed>>,
    receiver : Receiver<Option<Parsed>>,
}

impl Stats {
    pub fn new() -> Stats {
        let (sender, receiver) = channel();
        Stats {
            sender,
            receiver,
        }
    }
}

impl Start for Stats {
    fn start(&self) {
        println!("starting Stats processer");
        let mut max_outputs_per_tx = 0usize;
        let mut total_outputs = 0u64;
        let mut utxo = HashMap::new();
        let mut amount_over_32 = 0usize;

        let mut c = 0u64;
        loop {
            let received = self.receiver.recv().unwrap();
            match received {
                Some(received) => {
                    let tx = received.tx;
                    let outputs = tx.output.len();
                    total_outputs += outputs as u64;
                    if max_outputs_per_tx < outputs {
                        max_outputs_per_tx = outputs;
                        println!("max_outputs_per_tx is {} for {}", max_outputs_per_tx, tx.txid());
                    }
                    for (i, output) in tx.output.iter().enumerate() {
                        let o = OutPoint { txid: tx.txid(), vout: i as u32};
                        utxo.insert(trunc(&o), output.value);
                    }
                    for input in tx.input {
                        utxo.remove(&trunc(&input.previous_output));
                    }
                    if c % 10_000_000 == 0 {
                        println!("amount_over_32: {}", amount_over_32);
                        println!("total_outputs: {}", total_outputs);
                        println!("utxo len: {}", utxo.len());
                    }
                    let over_32 = tx.output.iter().filter(|o| o.value > 0xffffffff).count();
                    if over_32 > 0 {
                        amount_over_32 += over_32;
                    }
                },
                None => break,
            }
        }
        println!("amount_over_32: {}", amount_over_32);
        println!("total_outputs: {}", total_outputs);
        println!("utxo len: {}", utxo.len());
        println!("ending Stats processer");

    }

    fn get_sender(&self) -> Sender<Option<Parsed>> {
        self.sender.clone()
    }
}

fn trunc(outpoint : &OutPoint) -> Vec<u8> {
    serialize(outpoint)[26..].to_vec()
}

#[cfg(test)]
mod test {
    use bitcoin::OutPoint;
    use bitcoin::consensus::serialize;

    #[test]
    fn test() {
        println!("{}", serialize(&OutPoint::default())[28..].len());
    }

}