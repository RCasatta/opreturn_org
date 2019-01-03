use std::io;
use std::sync::mpsc::channel;
use std::thread;
use bitcoin::{BlockHeader, Transaction};
use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc::Sender;
use crate::op_return::OpReturn;
use crate::segwit::Segwit;

mod parse;
mod op_return;
mod segwit;

pub trait Start {
    fn start(&self);
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
    let (line_sender, line_receiver) = channel();

    let (sender, receiver) = channel();
    let op_return = OpReturn::new(receiver);
    let (sender2, receiver2) = channel();
    let segwit = Segwit::new(receiver2);

    let vec = vec![sender, sender2];
    let mut vec2 : Vec<Box<Start + Send>> = vec![Box::new(segwit), Box::new(op_return)];

    let handle = thread::spawn(move|| {
        let mut headers : HashMap<u32,BlockHeader> = HashMap::new();
        loop {
            let received = line_receiver.recv().unwrap();
            match received {
                Some(value) => {
                    //println!("{}", value);
                    let result = parse::line(value, &mut headers).unwrap();
                    //println!("{:?}", result)
                    for el in vec.iter() {
                        el.send(Some(result.clone())).unwrap();
                    }
                },
                None => {
                    for el in vec.iter() {
                        el.send(None).unwrap();
                    }

                    println!("None");
                    break;
                },
            }
        }
    });

    loop {
        match vec2.pop() {
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