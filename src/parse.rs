use crate::Parsed;
use bitcoin::{BlockHeader, Transaction};
use bitcoin::consensus::deserialize;
use std::error::Error;

pub fn line(line : String) -> Result<Parsed, Box<Error>> {
    let mut x = line.split_whitespace();
    let height = x.next().unwrap().parse::<u32>()?;
    let size = x.next().unwrap().parse::<u32>()?;
    let tx_count = x.next().unwrap().parse::<u32>()?;
    let block_header : BlockHeader = deserialize( &hex::decode( x.next().unwrap() )? )?;
    let tx : Transaction = deserialize( &hex::decode( x.next().unwrap() )? )?;

    Ok(Parsed {
        height,
        size,
        tx_count,
        block_header,
        tx,
    })
}