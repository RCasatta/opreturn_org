use crate::parse::TxOrBlock;
use crate::Startable;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};

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
        loop {
            let received = self.receiver.recv().expect("cannot get segwit");
            match received {
                TxOrBlock::Block(_block) => continue,
                TxOrBlock::Tx(_tx) => continue,
                _ => {
                    println!("Segwit: received {:?}", received);
                    break;
                },
            }
        }
        println!("ending Segwit processer");
    }

}
