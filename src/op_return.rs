use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use crate::{Start, Parsed};

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

}

impl Start for OpReturn {
    fn start(&self) {
        println!("starting op_return processer");
        loop {
            let received = self.receiver.recv().unwrap();
            match received {
                Some(_received) => continue,
                None => break,
            }
        }
        println!("ending op_return processer");
    }

    fn get_sender(&self) -> Sender<Option<Parsed>> {
        self.sender.clone()
    }
}