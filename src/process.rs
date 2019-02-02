use std::sync::mpsc::Receiver;
use crate::fee::BlockSizeHeightValues;

pub struct Process {
    receiver : Receiver<Option<BlockSizeHeightValues>>,
}

impl Process {
    pub fn new(receiver : Receiver<Option<BlockSizeHeightValues>> ) -> Process {
        Process {
            receiver,
        }
    }

    pub fn start(&self) {
        loop {
            let received = self.receiver.recv().expect("cannot receive fee block");
            match received {
                Some(block) => {
                    process(block);
                },
                None => break,
            }
        }
    }
}

fn process(_block : BlockSizeHeightValues) {

}