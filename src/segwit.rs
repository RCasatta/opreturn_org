use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use crate::{Start, Parsed};

pub struct Segwit {
    receiver : Receiver<Option<Parsed>>,
}

impl Segwit {
    pub fn new(receiver : Receiver<Option<Parsed>>) -> Segwit {
        Segwit {
            receiver,
        }
    }
}

impl Start for Segwit {
    fn start(&self) {
        loop {
            let received = self.receiver.recv().unwrap();
            match received {
                Some(_received) => print!("2"),
                None => break,
            }
        }
    }
}
