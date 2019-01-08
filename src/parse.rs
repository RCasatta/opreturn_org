use crate::Parsed;
use bitcoin::{BlockHeader, Transaction};
use bitcoin::consensus::deserialize;
use std::error::Error;

pub fn line(line : &str) -> Result<Parsed, Box<Error>> {
    let mut x = line.split_whitespace();
    let height = x.next().expect("cannot get height").parse::<u32>()?;
    let size = x.next().expect("cannot get size").parse::<u32>()?;
    let tx_count = x.next().expect("cannot get tx_count").parse::<u32>()?;
    let block_header : BlockHeader = deserialize( &hex::decode( x.next().expect("cannot get block_header") )? )?;
    let tx : Transaction = deserialize( &hex::decode( x.next().expect("cannot get tx") )? )?;

    Ok(Parsed {
        height,
        size,
        tx_count,
        block_header,
        tx,
    })
}