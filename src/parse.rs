use crate::Parsed;
use bitcoin::{BlockHeader, Transaction};
use bitcoin::consensus::deserialize;
use std::error::Error;
use std::collections::HashMap;

pub fn line(line : String, headers : &mut HashMap<u32,BlockHeader>) -> Result<Parsed, Box<Error>> {
    let mut x = line.split_whitespace();
    let height = x.next().unwrap().parse::<u32>()?;
    let size = x.next().unwrap().parse::<u32>()?;
    let tx_count = x.next().unwrap().parse::<u32>()?;
    let block_header = *headers.entry(height).or_insert(deserialize( &hex::decode( x.next().unwrap() )? )?);
    let tx : Transaction = deserialize( &hex::decode( x.next().unwrap() )? )?;

    Ok(Parsed {
        height,
        size,
        tx_count,
        block_header,
        tx,
    })
}