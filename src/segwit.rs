use crate::parse::TxOrBlock;
use crate::Startable;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use std::time::Instant;
use std::time::Duration;

pub struct Segwit {
    sender : Sender<TxOrBlock>,
    receiver : Receiver<TxOrBlock>,
}

impl Segwit {
    pub fn new() -> Segwit {
        let (sender, receiver) = channel();
        Segwit {
            sender,
            receiver,
        }
    }
    pub fn get_sender(&self) -> Sender<TxOrBlock> {
        self.sender.clone()
    }
}

impl Startable for Segwit {
    fn start(&self) {
        println!("starting Segwit processer");
        let mut wait_time =  Duration::from_secs(0);
        loop {
            let instant = Instant::now();
            let received = self.receiver.recv().expect("cannot get segwit");
            wait_time += instant.elapsed();
            match received {
                TxOrBlock::Block(_block) => continue,
                TxOrBlock::Tx(_tx) => continue,
                _ => {
                    println!("Segwit: received {:?}", received);
                    break;
                },
            }
        }
        println!("ending Segwit processer, wait time: {:?}", wait_time );
    }

}
