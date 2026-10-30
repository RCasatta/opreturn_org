//! Segwit script templates.
//!
//! Scripts behind P2WSH, P2SH-P2WSH and P2TR script-path outputs are only revealed when spent,
//! so they are taken from the inputs' witnesses. Every script is normalized into a template by
//! replacing the data pushes (keys, hashes) with placeholders, so that the same policy used with
//! different keys produces the same template. Counting how many distinct scripts share a template
//! gives an idea of the anonymity set of that script type.

use blocks_iterator::bitcoin::blockdata::opcodes::all::*;
use blocks_iterator::bitcoin::blockdata::opcodes::Opcode;
use blocks_iterator::bitcoin::blockdata::script::Instruction;
use blocks_iterator::bitcoin::{Script, TxIn, Txid};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};

/// Maximum number of distinct templates tracked, new templates after this are counted in [`OTHER`]
pub const MAX_TEMPLATES: usize = 200_000;

/// Templates longer than this are truncated and suffixed with an hash of the full template
pub const MAX_TEMPLATE_LEN: usize = 2_000;

/// Per template, distinct scripts are counted exactly up to this number, then with HyperLogLog
pub const EXACT_DISTINCT_LIMIT: usize = 128;

/// Key used for spends whose template doesn't fit in the map anymore
pub const OTHER: &str = "other";

/// Whether the template contains an envelope `OP_0 OP_IF ... OP_ENDIF`, typically inscriptions
pub fn is_envelope(template: &str) -> bool {
    template.contains("<envelope")
}

const HLL_P: u32 = 10;
const HLL_M: usize = 1 << HLL_P;

/// The kind of output spent, which reveals the script
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpendKind {
    /// Native segwit v0 P2WSH
    Wsh,
    /// P2WSH nested in P2SH
    ShWsh,
    /// Taproot script path, with the leaf version
    Tr(u8),
}

impl SpendKind {
    fn prefix(&self) -> String {
        match self {
            SpendKind::Wsh => "wsh".to_string(),
            SpendKind::ShWsh => "sh-wsh".to_string(),
            SpendKind::Tr(0xc0) => "tr".to_string(),
            SpendKind::Tr(v) => format!("tr-leaf{:02x}", v),
        }
    }
}

/// Returns the script revealed by spending `input`, if any.
///
/// `prevout_script` is the script pubkey of the output being spent; when it is `None` the kind is
/// inferred from the shape of the witness (used in tests and in the offline harness).
pub fn revealed_script<'a>(
    input: &'a TxIn,
    prevout_script: Option<&Script>,
) -> Option<(SpendKind, &'a [u8])> {
    let witness = &input.witness;
    if witness.is_empty() {
        return None;
    }
    let script_sig = input.script_sig.as_bytes();
    let nested_wsh = script_sig.len() == 35 && script_sig[0] == 34 && script_sig[1..3] == [0, 32];
    match prevout_script {
        Some(s) if s.is_p2wsh() => Some((SpendKind::Wsh, witness.last()?)),
        Some(s) if s.is_p2sh() => {
            if nested_wsh {
                Some((SpendKind::ShWsh, witness.last()?))
            } else {
                None
            }
        }
        Some(s) if s.is_p2tr() => tapscript(witness.iter().collect()),
        Some(_) => None,
        None => {
            if nested_wsh {
                return Some((SpendKind::ShWsh, witness.last()?));
            }
            if !script_sig.is_empty() {
                return None;
            }
            let items: Vec<&[u8]> = witness.iter().collect();
            // P2WPKH: signature and compressed pubkey
            if items.len() == 2 && items[1].len() == 33 && matches!(items[1][0], 2 | 3) {
                return None;
            }
            // In segwit v0 a witness script starting with 0x50 or 0xc0/0xc1 would fail, so such last
            // elements unambiguously mean taproot (annex or control block)
            let last = items.last()?;
            if last.first() == Some(&0x50) && items.len() >= 2 {
                return tapscript(items);
            }
            if items.len() >= 2 && is_control_block(last) {
                return tapscript(items);
            }
            if items.len() == 1 && matches!(last.len(), 64 | 65) {
                return None; // taproot key path
            }
            Some((SpendKind::Wsh, last))
        }
    }
}

fn is_control_block(el: &[u8]) -> bool {
    el.len() >= 33
        && (el.len() - 33) & 31 == 0
        && (el.len() - 33) / 32 <= 128
        && el[0] & 0xfe == 0xc0
}

/// Given the witness elements of a taproot spend, return the leaf script if it's a script path spend
fn tapscript(mut items: Vec<&[u8]>) -> Option<(SpendKind, &[u8])> {
    if items.len() >= 2 && items.last()?.first() == Some(&0x50) {
        items.pop(); // annex
    }
    if items.len() < 2 {
        return None; // key path
    }
    let control_block = items[items.len() - 1];
    if control_block.is_empty() {
        return None;
    }
    let script = items[items.len() - 2];
    Some((SpendKind::Tr(control_block[0] & 0xfe), script))
}

/// Decode a script number, not requiring minimal encoding
fn script_num(bytes: &[u8]) -> Option<i64> {
    if bytes.is_empty() || bytes.len() > 5 {
        return None;
    }
    let mut v: i64 = 0;
    for (i, b) in bytes.iter().enumerate() {
        v |= (*b as i64) << (8 * i);
    }
    let last = bytes[bytes.len() - 1];
    if last & 0x80 != 0 {
        v &= !(0x80i64 << (8 * (bytes.len() - 1)));
        v = -v;
    }
    Some(v)
}

fn opcode_name(op: Opcode) -> String {
    let b = op.to_u8();
    if b >= OP_PUSHNUM_1.to_u8() && b <= OP_PUSHNUM_16.to_u8() {
        format!("OP_{}", b - OP_PUSHNUM_1.to_u8() + 1)
    } else {
        format!("{:?}", op)
    }
}

fn push_placeholder(data: &[u8], prev: Option<Opcode>, next: Option<Opcode>) -> String {
    let checksig =
        |op: Opcode| op == OP_CHECKSIG || op == OP_CHECKSIGVERIFY || op == OP_CHECKSIGADD;
    let equal = |op: Opcode| op == OP_EQUAL || op == OP_EQUALVERIFY;
    let sha = |op: Opcode| op == OP_SHA256 || op == OP_HASH256;
    match next {
        // absolute timelocks are specific to a wallet: keep only whether it's an height or a time
        Some(op) if op == OP_CLTV => match script_num(data) {
            Some(n) if n < 500_000_000 => "<height>".to_string(),
            Some(_) => "<time>".to_string(),
            None => format!("<data:{}>", data.len()),
        },
        // relative timelocks are policy parameters, typically chosen among few values: keep them
        Some(op) if op == OP_CSV => match script_num(data) {
            Some(n) => n.to_string(),
            None => format!("<data:{}>", data.len()),
        },
        // 32 bytes are labelled only when the context is unambiguous
        Some(op) if data.len() == 32 && checksig(op) => "<xpk>".to_string(),
        Some(op) if data.len() == 32 && equal(op) && prev.is_some_and(sha) => "<h32>".to_string(),
        _ => match data.len() {
            // small numbers are structural (thresholds, sizes, counts)
            1 | 2 => script_num(data).map(|n| n.to_string()).unwrap(),
            20 => "<20>".to_string(),
            32 => "<32>".to_string(),
            33 if matches!(data[0], 2 | 3) => "<pk>".to_string(),
            65 if data[0] == 4 => "<pk65>".to_string(),
            n => format!("<data:{}>", n),
        },
    }
}

/// Normalize a script into a template, replacing data pushes with placeholders.
///
/// Envelopes `OP_0 OP_IF ... OP_ENDIF` (eg. ordinals inscriptions) are collapsed in a single
/// `<envelope:tag>` element, since their content is never executed and would make every
/// inscription a different template.
pub fn template(script: &[u8]) -> String {
    let script = Script::from_bytes(script);
    let mut instructions = vec![];
    let mut parse_error = false;
    for i in script.instructions() {
        match i {
            Ok(i) => instructions.push(i),
            Err(_) => {
                parse_error = true;
                break;
            }
        }
    }
    let mut parts: Vec<String> = Vec::with_capacity(instructions.len());
    let mut i = 0;
    while i < instructions.len() {
        let next_op = instructions.get(i + 1).and_then(|n| n.opcode());
        match instructions[i] {
            Instruction::PushBytes(p) if p.is_empty() && next_op == Some(OP_IF) => {
                // envelope: skip until the matching OP_ENDIF
                let tag = match instructions.get(i + 2) {
                    Some(Instruction::PushBytes(t))
                        if !t.is_empty()
                            && t.len() <= 16
                            && t.as_bytes().iter().all(|c| c.is_ascii_graphic()) =>
                    {
                        format!(":{}", String::from_utf8_lossy(t.as_bytes()))
                    }
                    _ => String::new(),
                };
                let mut depth = 0usize;
                let mut j = i + 1;
                while j < instructions.len() {
                    match instructions[j].opcode() {
                        Some(op) if op == OP_IF || op == OP_NOTIF => depth += 1,
                        Some(op) if op == OP_ENDIF => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => (),
                    }
                    j += 1;
                }
                let end = if j < instructions.len() {
                    " OP_ENDIF"
                } else {
                    ""
                };
                let envelope = format!("OP_0 OP_IF <envelope{}>{}", tag, end);
                // consecutive identical envelopes (eg. batch inscriptions) are written once with
                // their count, which is kept since it's visible on chain
                let repeated = format!("({})x", envelope);
                let count = match parts.last() {
                    Some(last) if *last == envelope => Some(2),
                    Some(last) if last.starts_with(&repeated) => {
                        last[repeated.len()..].parse::<usize>().ok().map(|n| n + 1)
                    }
                    _ => None,
                };
                match count {
                    Some(n) => *parts.last_mut().expect("checked") = format!("{}{}", repeated, n),
                    None => parts.push(envelope),
                }
                i = j + 1;
                continue;
            }
            Instruction::PushBytes(p) if p.is_empty() => parts.push("OP_0".to_string()),
            Instruction::PushBytes(p) => {
                let prev_op = i.checked_sub(1).and_then(|p| instructions[p].opcode());
                parts.push(push_placeholder(p.as_bytes(), prev_op, next_op))
            }
            Instruction::Op(op) => parts.push(opcode_name(op)),
        }
        i += 1;
    }
    if parse_error {
        parts.push("<parse_error>".to_string());
    }
    let mut result = parts.join(" ");
    if result.len() > MAX_TEMPLATE_LEN {
        let hash = hash64(result.as_bytes());
        let mut cut = MAX_TEMPLATE_LEN;
        while !result.is_char_boundary(cut) {
            cut -= 1;
        }
        result.truncate(cut);
        result.push_str(&format!(" ...#{:016x}", hash));
    }
    result
}

fn hash64(data: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

/// Count of distinct scripts, exact up to [`EXACT_DISTINCT_LIMIT`] then estimated with HyperLogLog
#[derive(Debug, Clone)]
pub enum Distinct {
    Exact(HashSet<u64>),
    Hll(Box<[u8; HLL_M]>),
}

impl Default for Distinct {
    fn default() -> Self {
        Distinct::Exact(HashSet::new())
    }
}

impl Distinct {
    pub fn insert(&mut self, hash: u64) {
        match self {
            Distinct::Exact(set) => {
                set.insert(hash);
                if set.len() > EXACT_DISTINCT_LIMIT {
                    let mut registers = Box::new([0u8; HLL_M]);
                    for h in set.iter() {
                        hll_insert(&mut registers, *h);
                    }
                    *self = Distinct::Hll(registers);
                }
            }
            Distinct::Hll(registers) => hll_insert(registers, hash),
        }
    }

    pub fn count(&self) -> u64 {
        match self {
            Distinct::Exact(set) => set.len() as u64,
            Distinct::Hll(registers) => hll_estimate(registers),
        }
    }

    pub fn is_exact(&self) -> bool {
        matches!(self, Distinct::Exact(_))
    }

    /// Approximate heap memory used
    pub fn heap_size(&self) -> usize {
        match self {
            // hashbrown: 8 bytes per bucket plus 1 control byte, buckets are a power of 2
            Distinct::Exact(set) => hashbrown_buckets(set.capacity()) * 9,
            Distinct::Hll(_) => HLL_M,
        }
    }
}

/// Number of buckets allocated by a std HashMap/HashSet with the given capacity, (7/8 load factor)
fn hashbrown_buckets(capacity: usize) -> usize {
    if capacity == 0 {
        0
    } else {
        (capacity * 8 / 7).next_power_of_two()
    }
}

fn hll_insert(registers: &mut [u8; HLL_M], hash: u64) {
    // DefaultHasher output is well distributed, mix anyway to be safe
    let hash = hash.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let index = (hash >> (64 - HLL_P)) as usize;
    let rank = ((hash << HLL_P) | (1 << (HLL_P - 1))).leading_zeros() as u8 + 1;
    if rank > registers[index] {
        registers[index] = rank;
    }
}

fn hll_estimate(registers: &[u8; HLL_M]) -> u64 {
    let m = HLL_M as f64;
    let alpha = 0.7213 / (1.0 + 1.079 / m);
    let sum: f64 = registers.iter().map(|r| 2f64.powi(-(*r as i32))).sum();
    let estimate = alpha * m * m / sum;
    let zeros = registers.iter().filter(|r| **r == 0).count();
    if estimate <= 2.5 * m && zeros > 0 {
        (m * (m / zeros as f64).ln()).round() as u64
    } else {
        estimate.round() as u64
    }
}

#[derive(Debug, Clone)]
pub struct TemplateStats {
    pub spends: u64,
    pub distinct: Distinct,
    pub first_txid: Txid,
    pub first_height: u32,
}

#[derive(Default)]
pub struct SegwitTemplates {
    pub templates: HashMap<String, TemplateStats>,
    pub total_spends: u64,
    /// spends whose template was not tracked because [`MAX_TEMPLATES`] was reached
    pub overflow_spends: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TemplateRow {
    pub template: String,
    pub spends: u64,
    pub distinct_scripts: u64,
    pub distinct_exact: bool,
    pub first_txid: String,
    pub first_height: u32,
}

impl SegwitTemplates {
    pub fn add_input(
        &mut self,
        input: &TxIn,
        prevout_script: Option<&Script>,
        txid: &Txid,
        height: u32,
    ) {
        if let Some((kind, script)) = revealed_script(input, prevout_script) {
            self.add_script(kind, script, txid, height);
        }
    }

    pub fn add_script(&mut self, kind: SpendKind, script: &[u8], txid: &Txid, height: u32) {
        self.total_spends += 1;
        let key = format!("{} {}", kind.prefix(), template(script));
        let full = self.templates.len() >= MAX_TEMPLATES;
        let key = if full && !self.templates.contains_key(&key) {
            self.overflow_spends += 1;
            OTHER.to_string()
        } else {
            key
        };
        let stats = self.templates.entry(key).or_insert_with(|| TemplateStats {
            spends: 0,
            distinct: Distinct::default(),
            first_txid: *txid,
            first_height: height,
        });
        stats.spends += 1;
        // the kind is part of the identity: the same script nested or not is a different output
        let mut hasher = DefaultHasher::new();
        kind.prefix().hash(&mut hasher);
        script.hash(&mut hasher);
        stats.distinct.insert(hasher.finish());
    }

    /// Rows sorted by spends descending
    pub fn rows(&self) -> Vec<TemplateRow> {
        let mut rows: Vec<_> = self
            .templates
            .iter()
            .map(|(k, v)| TemplateRow {
                template: k.clone(),
                spends: v.spends,
                distinct_scripts: v.distinct.count(),
                distinct_exact: v.distinct.is_exact(),
                first_txid: v.first_txid.to_string(),
                first_height: v.first_height,
            })
            .collect();
        rows.sort_by(|a, b| b.spends.cmp(&a.spends).then(a.template.cmp(&b.template)));
        rows
    }

    /// Approximate heap memory used by the templates map
    pub fn heap_size(&self) -> usize {
        let buckets = hashbrown_buckets(self.templates.capacity());
        let entry = std::mem::size_of::<(String, TemplateStats)>() + 1;
        buckets * entry
            + self
                .templates
                .iter()
                .map(|(k, v)| k.capacity() + v.distinct.heap_size())
                .sum::<usize>()
    }

    /// Number of templates whose distinct scripts count falls in the given buckets
    pub fn anonymity_set_histogram(&self, envelope: bool) -> Vec<(String, u64)> {
        let buckets: [(u64, u64, &str); 7] = [
            (1, 1, "1"),
            (2, 2, "2"),
            (3, 10, "3-10"),
            (11, 100, "11-100"),
            (101, 1_000, "101-1k"),
            (1_001, 10_000, "1k-10k"),
            (10_001, u64::MAX, ">10k"),
        ];
        buckets
            .iter()
            .map(|(min, max, label)| {
                let count = self
                    .templates
                    .iter()
                    .filter(|(k, _)| k.as_str() != OTHER && is_envelope(k) == envelope)
                    .map(|(_, v)| v.distinct.count())
                    .filter(|c| c >= min && c <= max)
                    .count();
                (label.to_string(), count as u64)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use blocks_iterator::bitcoin::blockdata::script::Builder;
    use blocks_iterator::bitcoin::hashes::Hash;
    use blocks_iterator::bitcoin::{ScriptBuf, Sequence, Witness};

    fn pk(i: u8) -> [u8; 33] {
        let mut k = [i; 33];
        k[0] = 2;
        k
    }

    fn multisig(k: i64, n: u8) -> ScriptBuf {
        let mut b = Builder::new().push_int(k);
        for i in 0..n {
            b = b.push_slice(pk(i + 10));
        }
        b.push_int(n as i64)
            .push_opcode(OP_CHECKMULTISIG)
            .into_script()
    }

    fn input(script_sig: ScriptBuf, witness: Vec<Vec<u8>>) -> TxIn {
        TxIn {
            script_sig,
            witness: Witness::from_slice(&witness),
            sequence: Sequence::MAX,
            ..Default::default()
        }
    }

    #[test]
    fn test_template_multisig_2of3() {
        let t = template(multisig(2, 3).as_bytes());
        assert_eq!(t, "OP_2 <pk> <pk> <pk> OP_3 OP_CHECKMULTISIG");
        // same policy different keys: same template
        let script = hex::decode("52210293de2378b245e0c4a8325d2beb2e537041a3b9b12c96052a9f30954700e56ef3210230d013baf38205252c298625a7c7799e1f11a016d3738198410bcf8bcc1fecab2103aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa53ae").unwrap();
        assert_eq!(template(&script), t);
    }

    #[test]
    fn test_template_two_pkh_or_csv_2of2() {
        // two pkh summed by OP_ADD requiring exactly one, with a 65535 blocks CSV, or a 2-of-2,
        // the CSV value is pushed as `OP_PUSHBYTES_3 ffff00` as found on chain
        let script = Builder::new()
            .push_opcode(OP_IF)
            .push_opcode(OP_DUP)
            .push_opcode(OP_HASH160)
            .push_slice([1u8; 20])
            .push_opcode(OP_EQUALVERIFY)
            .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_TOALTSTACK)
            .push_opcode(OP_DUP)
            .push_opcode(OP_HASH160)
            .push_slice([2u8; 20])
            .push_opcode(OP_EQUALVERIFY)
            .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_FROMALTSTACK)
            .push_opcode(OP_ADD)
            .push_opcode(OP_PUSHNUM_1)
            .push_opcode(OP_EQUALVERIFY)
            .push_slice([0xff, 0xff, 0x00])
            .push_opcode(OP_CSV)
            .push_opcode(OP_ELSE)
            .push_slice(pk(3))
            .push_opcode(OP_CHECKSIGVERIFY)
            .push_slice(pk(4))
            .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_ENDIF)
            .into_script();
        assert_eq!(
            template(script.as_bytes()),
            "OP_IF OP_DUP OP_HASH160 <20> OP_EQUALVERIFY OP_CHECKSIG OP_TOALTSTACK OP_DUP OP_HASH160 <20> OP_EQUALVERIFY OP_CHECKSIG OP_FROMALTSTACK OP_ADD OP_1 OP_EQUALVERIFY 65535 OP_CSV OP_ELSE <pk> OP_CHECKSIGVERIFY <pk> OP_CHECKSIG OP_ENDIF"
        );
    }

    #[test]
    fn test_template_multisig_7of8() {
        assert_eq!(
            template(multisig(7, 8).as_bytes()),
            "OP_7 <pk> <pk> <pk> <pk> <pk> <pk> <pk> <pk> OP_8 OP_CHECKMULTISIG"
        );
    }

    #[test]
    fn test_template_liana_recovery() {
        // miniscript: or_d(pk(A),and_v(v:pkh(B),older(52560)))
        let script = Builder::new()
            .push_slice(pk(1))
            .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_IFDUP)
            .push_opcode(OP_NOTIF)
            .push_opcode(OP_DUP)
            .push_opcode(OP_HASH160)
            .push_slice([7u8; 20])
            .push_opcode(OP_EQUALVERIFY)
            .push_opcode(OP_CHECKSIGVERIFY)
            .push_int(52560)
            .push_opcode(OP_CSV)
            .push_opcode(OP_ENDIF)
            .into_script();
        assert_eq!(
            template(script.as_bytes()),
            "<pk> OP_CHECKSIG OP_IFDUP OP_NOTIF OP_DUP OP_HASH160 <20> OP_EQUALVERIFY OP_CHECKSIGVERIFY 52560 OP_CSV OP_ENDIF"
        );
        // absolute timelocks are bucketed, small CSV are OP_n
        let script = Builder::new()
            .push_int(800_000)
            .push_opcode(OP_CLTV)
            .push_opcode(OP_DROP)
            .push_int(1_700_000_000)
            .push_opcode(OP_CLTV)
            .push_opcode(OP_DROP)
            .push_int(1)
            .push_opcode(OP_CSV)
            .into_script();
        assert_eq!(
            template(script.as_bytes()),
            "<height> OP_CLTV OP_DROP <time> OP_CLTV OP_DROP OP_1 OP_CSV"
        );
    }

    #[test]
    fn test_template_checksigadd() {
        // 2-of-3 tapscript multisig, plus a 17 threshold pushed as data
        let x = |i: u8| [i; 32];
        let script = Builder::new()
            .push_slice(x(1))
            .push_opcode(OP_CHECKSIG)
            .push_slice(x(2))
            .push_opcode(OP_CHECKSIGADD)
            .push_slice(x(3))
            .push_opcode(OP_CHECKSIGADD)
            .push_int(2)
            .push_opcode(OP_NUMEQUAL)
            .into_script();
        assert_eq!(
            template(script.as_bytes()),
            "<xpk> OP_CHECKSIG <xpk> OP_CHECKSIGADD <xpk> OP_CHECKSIGADD OP_2 OP_NUMEQUAL"
        );
        let script = Builder::new()
            .push_int(17)
            .push_opcode(OP_NUMEQUAL)
            .into_script();
        assert_eq!(template(script.as_bytes()), "17 OP_NUMEQUAL");
    }

    #[test]
    fn test_template_32_bytes_context() {
        // hash lock: OP_SIZE 32 OP_EQUALVERIFY OP_SHA256 <h> OP_EQUALVERIFY <xpk> OP_CHECKSIG
        let script = Builder::new()
            .push_opcode(OP_SIZE)
            .push_int(32)
            .push_opcode(OP_EQUALVERIFY)
            .push_opcode(OP_SHA256)
            .push_slice([1u8; 32])
            .push_opcode(OP_EQUALVERIFY)
            .push_slice([2u8; 32])
            .push_opcode(OP_CHECKSIGVERIFY)
            .push_slice([3u8; 32])
            .push_opcode(OP_DROP)
            .push_slice([4u8; 7])
            .push_opcode(OP_DROP)
            .push_slice([5u8; 3])
            .into_script();
        assert_eq!(
            template(script.as_bytes()),
            "OP_SIZE 32 OP_EQUALVERIFY OP_SHA256 <h32> OP_EQUALVERIFY <xpk> OP_CHECKSIGVERIFY <32> OP_DROP <data:7> OP_DROP <data:3>"
        );
    }

    #[test]
    fn test_template_envelope() {
        let script = Builder::new()
            .push_slice([1u8; 32])
            .push_opcode(OP_CHECKSIG)
            .push_opcode(OP_PUSHBYTES_0)
            .push_opcode(OP_IF)
            .push_slice(b"ord")
            .push_int(1)
            .push_slice(b"text/plain")
            .push_opcode(OP_PUSHBYTES_0)
            .push_slice([9u8; 64])
            .push_opcode(OP_ENDIF)
            .into_script();
        assert_eq!(
            template(script.as_bytes()),
            "<xpk> OP_CHECKSIG OP_0 OP_IF <envelope:ord> OP_ENDIF"
        );
        let mut b = Builder::new()
            .push_slice([1u8; 32])
            .push_opcode(OP_CHECKSIG);
        for _ in 0..3 {
            b = b
                .push_opcode(OP_PUSHBYTES_0)
                .push_opcode(OP_IF)
                .push_slice(b"ord")
                .push_opcode(OP_ENDIF);
        }
        assert_eq!(
            template(b.into_script().as_bytes()),
            "<xpk> OP_CHECKSIG (OP_0 OP_IF <envelope:ord> OP_ENDIF)x3"
        );
    }

    #[test]
    fn test_template_long_and_invalid() {
        let mut b = Builder::new();
        for _ in 0..1000 {
            b = b.push_slice([1u8; 32]).push_opcode(OP_CHECKSIGADD);
        }
        let t = template(b.into_script().as_bytes());
        assert!(t.len() < MAX_TEMPLATE_LEN + 30);
        assert!(t.contains("...#"));
        assert_eq!(template(&[0x4c]), "<parse_error>"); // truncated OP_PUSHDATA1
    }

    #[test]
    fn test_revealed_script() {
        let ws = multisig(2, 3);
        let wsh = ScriptBuf::new_p2wsh(&ws.wscript_hash());

        // native p2wsh
        let i = input(
            ScriptBuf::new(),
            vec![vec![], vec![1; 71], vec![2; 71], ws.to_bytes()],
        );
        let expected = Some((SpendKind::Wsh, ws.as_bytes()));
        assert_eq!(revealed_script(&i, Some(&wsh)), expected);
        assert_eq!(revealed_script(&i, None), expected);

        // nested
        let script_sig = ScriptBuf::from_bytes([&[34u8][..], wsh.as_bytes()].concat());
        let i = input(script_sig, vec![vec![], ws.to_bytes()]);
        let p2sh = ScriptBuf::new_p2sh(&ScriptBuf::from(wsh.to_bytes()).script_hash());
        let expected = Some((SpendKind::ShWsh, ws.as_bytes()));
        assert_eq!(revealed_script(&i, Some(&p2sh)), expected);
        assert_eq!(revealed_script(&i, None), expected);

        // p2wpkh
        let i = input(ScriptBuf::new(), vec![vec![1; 71], pk(1).to_vec()]);
        assert_eq!(revealed_script(&i, None), None);
        let p2wpkh = ScriptBuf::new_p2wpkh(&blocks_iterator::bitcoin::WPubkeyHash::all_zeros());
        assert_eq!(revealed_script(&i, Some(&p2wpkh)), None);
    }

    #[test]
    fn test_revealed_tapscript_annex() {
        let leaf = vec![0x20; 34];
        let mut cb = vec![0xc1];
        cb.extend([5u8; 32]);
        let annex = vec![0x50, 1, 2, 3];
        let p2tr = ScriptBuf::from_bytes([&[0x51, 0x20][..], &[3u8; 32][..]].concat());
        let expected = Some((SpendKind::Tr(0xc0), &leaf[..]));

        let i = input(
            ScriptBuf::new(),
            vec![vec![1; 64], leaf.clone(), cb.clone()],
        );
        assert_eq!(revealed_script(&i, Some(&p2tr)), expected);
        assert_eq!(revealed_script(&i, None), expected);

        let w = vec![vec![1; 64], leaf.clone(), cb.clone(), annex.clone()];
        let i = input(ScriptBuf::new(), w);
        assert_eq!(revealed_script(&i, Some(&p2tr)), expected);
        assert_eq!(revealed_script(&i, None), expected);

        // key path, with and without annex
        let i = input(ScriptBuf::new(), vec![vec![1; 64]]);
        assert_eq!(revealed_script(&i, Some(&p2tr)), None);
        assert_eq!(revealed_script(&i, None), None);
        let i = input(ScriptBuf::new(), vec![vec![1; 64], annex]);
        assert_eq!(revealed_script(&i, Some(&p2tr)), None);
        assert_eq!(revealed_script(&i, None), None);
    }

    #[test]
    fn test_distinct_and_cap() {
        let mut d = Distinct::default();
        for i in 0..EXACT_DISTINCT_LIMIT as u64 {
            d.insert(hash64(&i.to_le_bytes()));
            d.insert(hash64(&i.to_le_bytes()));
        }
        assert_eq!(d.count(), EXACT_DISTINCT_LIMIT as u64);
        assert!(d.is_exact());
        for i in 0..100_000u64 {
            d.insert(hash64(&i.to_le_bytes()));
        }
        assert!(!d.is_exact());
        let c = d.count() as f64;
        assert!((c - 100_000.0).abs() / 100_000.0 < 0.1, "{}", c);

        let mut t = SegwitTemplates::default();
        let txid = Txid::all_zeros();
        for _ in 0..2 {
            t.add_script(SpendKind::Wsh, multisig(2, 3).as_bytes(), &txid, 1);
        }
        t.add_script(SpendKind::Wsh, multisig(7, 8).as_bytes(), &txid, 1);
        let rows = t.rows();
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0].template,
            "wsh OP_2 <pk> <pk> <pk> OP_3 OP_CHECKMULTISIG"
        );
        assert_eq!((rows[0].spends, rows[0].distinct_scripts), (2, 1));
    }
}
