mod process_bip158;
mod process_opret;
mod process_stats;
mod process_tx;

pub use process_bip158::{Bip158Stats, ProcessBip158Stats};
pub use process_opret::{OpReturnData, ProcessOpRet, ScriptType};
pub use process_stats::{ProcessStats, Stats};
pub use process_tx::{ProcessTxStats, TxStats};

use blocks_iterator::bitcoin::blockdata::opcodes;
use chrono::{DateTime, Datelike, TimeZone, Utc};

pub fn parse_multisig(witness_script: &Vec<u8>) -> Option<String> {
    let witness_script_len = witness_script.len();
    if witness_script.last() == Some(&opcodes::all::OP_CHECKMULTISIG.into_u8())
        && witness_script_len > 1
    {
        let n = read_pushnum(witness_script[0]);
        let m = read_pushnum(witness_script[witness_script_len - 2]);
        if n.is_some() && m.is_some() {
            return Some(format!("{:02}of{:02}", n.unwrap(), m.unwrap()));
        }
    }
    None
}

pub fn read_pushnum(value: u8) -> Option<u8> {
    if value >= opcodes::all::OP_PUSHNUM_1.into_u8()
        && value <= opcodes::all::OP_PUSHNUM_16.into_u8()
    {
        Some(value - opcodes::all::OP_PUSHNUM_1.into_u8() + 1)
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
        assert!(d >= 1 && d <= 9);
        n /= 10;
        1 + (n * 9 + d - 1) * 10 + e
    } else {
        1 + ((n - 1) * 10) + 9
    }
}

#[cfg(test)]
pub fn decompress_amount(x: u64) -> u64 {
    if x == 0 {
        return 0;
    }
    let mut x = x;
    x -= 1;
    let mut e = x % 10;
    x /= 10;
    let mut n;
    if e < 9 {
        let d = (x % 9) + 1;
        x /= 9;
        n = x * 10 + d;
    } else {
        n = x + 1;
    }
    loop {
        if e == 0 {
            break;
        }
        n *= 10;
        e -= 1;
    }
    n
}

pub fn date_index(date: DateTime<Utc>) -> usize {
    return (date.year() as usize - 2009) * 12 + (date.month() as usize - 1);
}

pub fn month_date(yyyymm: String) -> DateTime<Utc> {
    let year: i32 = yyyymm[0..4].parse().unwrap();
    let month: u32 = yyyymm[4..6].parse().unwrap();
    Utc.ymd(year, month, 1).and_hms(0, 0, 0)
}

pub fn month_index(yyyymm: String) -> usize {
    date_index(month_date(yyyymm))
}

pub fn month_array_len() -> usize {
    date_index(Utc::now()) + 1
}
