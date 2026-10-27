mod process_bip158;
mod process_opret;
mod process_stats;
mod process_tx;

pub use process_bip158::{Bip158Stats, ProcessBip158Stats};
pub use process_opret::{OpReturnData, ProcessOpRet, ScriptType};
pub use process_stats::{ProcessStats, Stats};
pub use process_tx::{ProcessTxStats, TxStats};

use blocks_iterator::bitcoin::blockdata::opcodes;
use blocks_iterator::bitcoin::blockdata::script::Instruction;
use blocks_iterator::bitcoin::{PublicKey, Script, Transaction};

pub fn parse_pubkeys_in_script(script: &Script) -> Vec<PublicKey> {
    let mut r = vec![];
    for el in script.instructions().flatten() {
        if let Instruction::PushBytes(inst) = el {
            if let Ok(p) = PublicKey::from_slice(inst.as_bytes()) {
                r.push(p);
            }
        }
    }
    r
}

pub fn parse_pubkeys_in_tx(tx: &Transaction) -> Vec<PublicKey> {
    let mut r = vec![];
    for input in tx.input.iter() {
        for witness_el in input.witness.iter() {
            if let Ok(p) = PublicKey::from_slice(witness_el) {
                r.push(p);
            }
        }
        r.extend(parse_pubkeys_in_script(&input.script_sig));
    }
    for output in tx.output.iter() {
        r.extend(parse_pubkeys_in_script(&output.script_pubkey));
    }
    r
}

pub fn parse_multisig(witness_script: &[u8]) -> Option<String> {
    let witness_script_len = witness_script.len();
    if witness_script.last() == Some(&opcodes::all::OP_CHECKMULTISIG.to_u8())
        && witness_script_len > 1
    {
        let n = read_pushnum(witness_script[0]);
        let m = read_pushnum(witness_script[witness_script_len - 2]);
        if let (Some(n), Some(m)) = (n, m) {
            return Some(format!("{:02}of{:02}", n, m));
        }
    }
    None
}

pub fn read_pushnum(value: u8) -> Option<u8> {
    if value >= opcodes::all::OP_PUSHNUM_1.to_u8() && value <= opcodes::all::OP_PUSHNUM_16.to_u8() {
        Some(value - opcodes::all::OP_PUSHNUM_1.to_u8() + 1)
    } else {
        None
    }
}

pub fn encoded_length_7bit_varint(mut value: u64) -> u64 {
    let mut bytes = 1;
    loop {
        if value <= 0x7F {
            return bytes;
        }
        bytes += 1;
        value >>= 7;
    }
}

pub fn compress_amount(n: u64) -> u64 {
    let mut n = n;
    if n == 0 {
        return 0;
    }
    let mut e = 0u64;
    loop {
        if (n % 10) != 0 || e >= 9 {
            break;
        }
        n /= 10;
        e += 1;
    }
    if e < 9 {
        let d = n % 10;
        assert!((1..=9).contains(&d));
        n /= 10;
        1 + (n * 9 + d - 1) * 10 + e
    } else {
        1 + ((n - 1) * 10) + 9
    }
}

pub fn block_index(height: u32) -> usize {
    height as usize / 1000
}
