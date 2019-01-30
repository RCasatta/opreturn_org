use bitcoin::{BlockHeader, Transaction};
use bitcoin::consensus::deserialize;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub struct TxParsed {
    pub height : u32,
    pub tx: Transaction,
}

#[derive(Debug, Clone)]
pub struct BlockParsed {
    pub height : u32,
    pub size : u32,
    pub tx_count : u32,
    pub block_header: BlockHeader,
}

#[derive(Debug, Clone)]
pub enum TxOrBlock {
    Tx(TxParsed),
    Block(BlockParsed),
    End,
}

#[derive(Debug)]
struct MyError(String);

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error: {}", self.0)
    }
}

impl Error for MyError {}

pub fn line(line : &str) -> Result<TxOrBlock, Box<Error>> {
    let mut x = line.split_whitespace();
    let type_letter = match x.next() {
        Some(l) => l,
        None => return Err(Box::new(MyError("Oops".into()))),
    };
    match type_letter {
        "T" => {
            let height = x.next().expect("cannot get height").parse::<u32>()?;
            let tx : Transaction = deserialize( &hex::decode( x.next().expect("cannot get tx") )? )?;
            Ok(TxOrBlock::Tx(TxParsed {
                height,
                tx,
            }))
        },
        "B" => {
            let height = x.next().expect("cannot get height").parse::<u32>()?;
            let size = x.next().expect("cannot get size").parse::<u32>()?;
            let tx_count = x.next().expect("cannot get tx_count").parse::<u32>()?;
            let block_header : BlockHeader = deserialize( &hex::decode( x.next().expect("cannot get block_header") )? )?;
            Ok(TxOrBlock::Block(BlockParsed {
                height,
                size,
                tx_count,
                block_header,
            }))
        },
        _ => Err(Box::new(MyError("Oops".into()))),
    }
}
