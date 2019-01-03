use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use crate::{Start, Parsed};

pub struct OpReturn {
    receiver : Receiver<Option<Parsed>>,
}

impl OpReturn {
    pub fn new(receiver : Receiver<Option<Parsed>>) -> OpReturn {
        OpReturn {
            receiver,
        }
    }

}

impl Start for OpReturn {
    fn start(&self) {
        loop {
            let received = self.receiver.recv().unwrap();
            match received {
                Some(_received) => print!("1"),
                None => break,
            }
        }
    }
}