use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::{map_by_value, Page};
use crate::process::{OpReturnData, ProtoKey};
use std::collections::HashMap;

/// Unofficial protocol labels, matched as a prefix of the first 3 data bytes pushed after
/// OP_RETURN. Nothing in consensus enforces them, so they are only hints.
///
/// The first group comes from the labels used by the old op_return Java code
/// (StatsToJSONDataTable.java), including its variant grouping.
const LABELS: &[(&[u8], &str)] = &[
    (b"BIT", "Bitproof"),
    (b"DOC", "Docproof"),
    (b"FAC", "Factom"),
    (b"Fa\x00", "Factom"),
    (b"SPK", "Coinspark"),
    (b"OA\x01", "Open Assets"),
    (b"CC\x01", "Colu"),
    (b"CC\x02", "Colu"),
    (b"ASC", "Ascribe pool"),
    (b"EW ", "Eternity Wall"),
    (b"EWA", "Eternity Wall"),
    (b"EWC", "Eternity Wall"),
    (b"MG\x00", "Monegraph"),
    (b"MG\xff", "Monegraph"),
    (b"id;", "Blockstack"),
    (b"id+", "Blockstack"),
    (b"id:", "Blockstack"),
    (b"id?", "Blockstack"),
    (b"id>", "Blockstack"),
    (b"CNT", "Counterparty"),
    (b"omn", "Omni Layer"),
    // the Java code labelled these "Stampery", noting they are not Stampery
    (b"S4", "S4 (unknown)"),
    (b"S5", "S5 (unknown)"),
    // modern ones
    (b"\xaa\x21\xa9", "Segwit commitment"), // BIP141 witness commitment, aa21a9ed
    (b"X2", "Stacks"),                      // Stacks mainnet magic bytes
    (b"RSK", "RSK merge-mining"),           // "RSKBLOCK:"
    (b"OUT", "THORChain/Maya memo"),        // "OUT:<txid>"
    (b"=:", "THORChain/Maya memo"),         // "=:<asset>:<destination>" swap
];

/// Human readable label of a protocol key, unknown prefixes are shown as hex
pub fn label(key: &ProtoKey) -> String {
    match key {
        ProtoKey::Runes => "Runes".to_string(),
        _ => key
            .data()
            .and_then(|data| {
                LABELS
                    .iter()
                    .find(|(prefix, _)| data.starts_with(prefix))
                    .map(|(_, label)| label.to_string())
            })
            .unwrap_or_else(|| key.to_string()),
    }
}

/// Merge the counts of keys having the same label
pub fn group_by_label(map: &HashMap<ProtoKey, u64>) -> HashMap<String, u64> {
    let mut grouped = HashMap::new();
    for (key, count) in map {
        *grouped.entry(label(key)).or_insert(0) += count;
    }
    grouped
}

fn pie(title: &str, dataset_label: &str, map: &HashMap<ProtoKey, u64>) -> Chart {
    let map = map_by_value(&group_by_label(map));
    let mut chart = Chart::new(title.to_string(), Kind::Pie, map.keys().cloned().collect());
    let dataset = Dataset {
        label: dataset_label.to_string(),
        data: map.values().cloned().collect(),
        background_color: Color::rainbow(),
        border_color: vec![],
        fill: true,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);
    chart
}

pub fn op_return_protocols(opret: &OpReturnData) -> Page {
    let chart1 = pie("Last month", "chart1", &opret.op_ret_per_proto_last_month);
    let chart2 = pie("Last year", "chart2", &opret.op_ret_per_proto_last_year);
    let chart3 = pie("Ever", "chart3", &opret.op_ret_per_proto);

    Page {
        title: "OP_RETURN protocols".to_string(),
        description: "Protocols are guessed from the first 3 bytes of the data pushed after OP_RETURN (OP_RETURN OP_13 for Runes), which can indicate the protocol but it's not an enforced rule by the Bitcoin consensus. Known prefixes are labelled, unknown ones are shown as hex.".to_string(),
        permalink: "op-return-protocols".to_string(),
        charts: vec![chart1, chart2, chart3],
        text: "".to_string(),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{group_by_label, label};
    use crate::process::ProtoKey;
    use std::collections::HashMap;
    use std::str::FromStr;

    fn key(s: &str) -> ProtoKey {
        ProtoKey::from_str(s).unwrap()
    }

    #[test]
    fn test_label() {
        assert_eq!(label(&key("6f6d6e")), "Omni Layer");
        assert_eq!(label(&key("aa21a9")), "Segwit commitment");
        assert_eq!(label(&key("58325b")), "Stacks");
        assert_eq!(label(&key("533412")), "S4 (unknown)");
        assert_eq!(label(&ProtoKey::Runes), "Runes");
        assert_eq!(label(&key("7b2270")), "7b2270");
        assert_eq!(label(&ProtoKey::Bare), "bare");
        assert_eq!(label(&key("58")), "58"); // data shorter than the "X2" prefix
    }

    #[test]
    fn test_group_by_label() {
        let mut map = HashMap::new();
        map.insert(key("69643b"), 1); // id;
        map.insert(key("69642b"), 2); // id+
        map.insert(key("69643a"), 4); // id:
        map.insert(key("455720"), 8); // "EW "
        map.insert(key("455741"), 16); // EWA
        map.insert(ProtoKey::Runes, 32);
        map.insert(key("abcdef"), 64);
        let grouped = group_by_label(&map);
        assert_eq!(grouped.len(), 4);
        assert_eq!(grouped["Blockstack"], 7);
        assert_eq!(grouped["Eternity Wall"], 24);
        assert_eq!(grouped["Runes"], 32);
        assert_eq!(grouped["abcdef"], 64);
    }
}
