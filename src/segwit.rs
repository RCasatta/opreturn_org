use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use crate::{Start, Parsed};

pub struct Segwit {
    sender : Sender<Option<Parsed>>,
    receiver : Receiver<Option<Parsed>>,
}

impl Segwit {
    pub fn new() -> Segwit {
        let (sender, receiver) = channel();
        Segwit {
            sender,
            receiver,
        }
    }
}

impl Start for Segwit {
    fn start(&self) {
        loop {
            let received = self.receiver.recv().unwrap();
            match received {
                Some(_received) => continue,
                None => break,
            }
        }
    }

    fn get_sender(&self) -> Sender<Option<Parsed>> {
        self.sender.clone()
    }
}
