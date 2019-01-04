use std::sync::mpsc::channel;
use std::sync::mpsc::{Sender, Receiver};
use crate::{Start, Parsed};
use std::collections::HashMap;

pub struct Blocks {
    sender : Sender<Option<Parsed>>,
    receiver : Receiver<Option<Parsed>>,
}

impl Blocks {
    pub fn new() -> Blocks {
        let (sender, receiver) = channel();
        Blocks {
            sender,
            receiver,
        }
    }
}

impl Start for Blocks {
    fn start(&self) {
        println!("starting blocks processer");
        let mut map : HashMap<u32, u32> = HashMap::new();
        loop {
            let received = self.receiver.recv().unwrap();
            match received {
                Some(received) => {
                    map.entry(received.height).or_insert(received.size);
                },
                None => break,
            }
        }
        let mut sum = 0u64;
        for v in map.values() {
            sum += *v as u64;
        }
        println!("sum = {}", sum);
        println!("ending blocks processer");
    }

    fn get_sender(&self) -> Sender<Option<Parsed>> {
        self.sender.clone()
    }
}
