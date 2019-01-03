use crate::op_return::OpReturn;
use crate::segwit::Segwit;
use crate::blocks::Blocks;
use std::io;
use std::thread;
use bitcoin::{BlockHeader, Transaction};
use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::Sender;
use std::sync::mpsc::sync_channel;

mod parse;
mod op_return;
mod segwit;
mod blocks;

pub trait Start {
    fn start(&self);
    fn get_sender(&self) -> Sender<Option<Parsed>>;
}

#[derive(Debug, Clone)]
pub struct Parsed {
    height : u32,
    size : u32,
    tx_count : u32,
    block_header: BlockHeader,
    tx: Transaction,
}

fn main() -> Result<(), Box<Error>> {
    let (line_sender, line_receiver) = sync_channel(1000);

    let mut vec : Vec<Box<Start + Send>> = vec![Box::new(Segwit::new()),
                                                Box::new(Blocks::new()),
                                                Box::new(OpReturn::new())];

    let vec_senders : Vec<Sender<Option<Parsed>>> = vec.iter().map(|el| el.get_sender()).collect();
    let handle = thread::spawn(move|| {
        let mut headers : HashMap<u32,BlockHeader> = HashMap::new();
        loop {
            let received = line_receiver.recv().unwrap();
            match received {
                Some(value) => {
                    //println!("{}", value);
                    let result = parse::line(value, &mut headers).unwrap();
                    //println!("{:?}", result)
                    for el in vec_senders.iter() {
                        el.send(Some(result.clone())).unwrap();
                    }
                },
                None => {
                    for el in vec_senders.iter() {
                        el.send(None).unwrap();
                    }
                    break;
                },
            }
        }
    });

    loop {
        match vec.pop() {
            Some(el) => {
                thread::spawn(move|| {
                    el.start();
                });
            },
            None => break,
        };
    }

    loop {
        let mut buffer = String::new();
        match io::stdin().read_line(&mut buffer) {
            Ok(n) => {
                if n == 0 {
                    break;
                }
                line_sender.send(Some(buffer)).unwrap();
            }
            Err(error) => {
                println!("error: {}", error);
                break;
            }
        }
    }
    line_sender.send(None).unwrap();
    handle.join().unwrap();

    Ok(())
}