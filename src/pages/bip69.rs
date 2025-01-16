use crate::charts::{Chart, Color, Dataset, Kind};
use crate::counter::{cumulative, perc_100};
use crate::pages::{to_label_map, Page};
use crate::process::TxStats;

use core::cmp::Ordering;

use bitcoin::hashes::Hash;
use blocks_iterator::bitcoin::{Transaction, TxIn, TxOut};

pub fn bip69(tx_stats: &TxStats) -> Page {
    let title = "BIP69 adoption".to_string();

    let (no_vec, mul) = tx_stats.is_bip69[0].finish();
    let no_bip69 = to_label_map(&cumulative(&no_vec), mul);
    let (yes_vec, mul) = tx_stats.is_bip69[1].finish();
    let yes_bip69 = to_label_map(&cumulative(&yes_vec), mul);

    let mut charts = vec![];

    let labels: Vec<_> = no_bip69.keys().cloned().collect();
    let mut chart = Chart::new(title.clone(), Kind::Line, labels);

    let dataset = Dataset {
        label: "yes".to_string(),
        data: yes_bip69.values().cloned().collect(),
        background_color: vec![Color::Orange],
        border_color: vec![Color::Orange],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "no".to_string(),
        data: no_bip69.values().cloned().collect(),
        background_color: vec![Color::Red],
        border_color: vec![Color::Red],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let sum_vec: Vec<u64> = yes_vec
        .iter()
        .zip(no_vec.iter())
        .map(|(a, b)| a + b)
        .collect();
    let perc = perc_100(&yes_vec, &sum_vec);
    let dataset = Dataset {
        label: "% compliance".to_string(),
        data: perc,
        background_color: vec![Color::Green],
        border_color: vec![Color::Green],
        border_dash: Some([5, 5]),
        ..Default::default()
    };
    chart.add_dataset(dataset, Some("y2".to_string()));

    charts.push(chart);

    Page {
        title,
        description: "BIP69 compliance, ordered transaction inputs and outputs".to_string(),
        permalink: "bip69".to_string(),
        charts,
        text: "".to_string(),
    }
}

fn cmp_inputs(a: &TxIn, b: &TxIn) -> Ordering {
    match a
        .previous_output
        .txid
        .as_raw_hash()
        .as_byte_array()
        .iter()
        .rev()
        .cmp(
            b.previous_output
                .txid
                .as_raw_hash()
                .as_byte_array()
                .iter()
                .rev(),
        ) {
        Ordering::Less => Ordering::Less,
        Ordering::Greater => Ordering::Greater,
        Ordering::Equal => a.previous_output.vout.cmp(&b.previous_output.vout),
    }
}

fn cmp_outputs(a: &TxOut, b: &TxOut) -> Ordering {
    match a.value.cmp(&b.value) {
        Ordering::Less => Ordering::Less,
        Ordering::Greater => Ordering::Greater,
        Ordering::Equal => a.script_pubkey.cmp(&b.script_pubkey),
    }
}

/// tx having 1-input/1-output or 1-input/0-output are bip69 compliance but they haven't a chance
/// to be non compliant, so they are excluded from the counting
pub fn has_more_than_one_input_output(tx: &Transaction) -> bool {
    !(tx.input.len() <= 1 && tx.output.len() <= 1)
}

pub fn is_bip69(tx: &Transaction) -> bool {
    let inputs_not_ordered = tx
        .input
        .windows(2)
        .any(|inputs| cmp_inputs(&inputs[0], &inputs[1]) == Ordering::Greater);
    let outputs_not_ordered = tx
        .output
        .windows(2)
        .any(|outputs| cmp_outputs(&outputs[0], &outputs[1]) == Ordering::Greater);
    !(inputs_not_ordered || outputs_not_ordered)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::pages::bip69::{cmp_inputs, cmp_outputs, has_more_than_one_input_output, is_bip69};
    use bitcoin::ScriptBuf;
    use blocks_iterator::bitcoin::consensus::deserialize;
    use blocks_iterator::bitcoin::hashes::hex::FromHex;
    use blocks_iterator::bitcoin::{OutPoint, Script, Transaction, TxIn, TxOut};

    // tx 28204cad1d7fc1d199e8ef4fa22f182de6258a3eaafe1bbe56ebdcacd3069a5f 2-inputs/2-outputs
    const TX: &'static str = "010000000255605dc6f5c3dc148b6da58442b0b2cd422be385eab2ebea4119ee9c268d28350000000049483045022100aa46504baa86df8a33b1192b1b9367b4d729dc41e389f2c04f3e5c7f0559aae702205e82253a54bf5c4f65b7428551554b2045167d6d206dfe6a2e198127d3f7df1501ffffffff55605dc6f5c3dc148b6da58442b0b2cd422be385eab2ebea4119ee9c268d2835010000004847304402202329484c35fa9d6bb32a55a70c0982f606ce0e3634b69006138683bcd12cbb6602200c28feb1e2555c3210f1dddb299738b4ff8bbe9667b68cb8764b5ac17b7adf0001ffffffff0200e1f505000000004341046a0765b5865641ce08dd39690aade26dfbf5511430ca428a3089261361cef170e3929a68aee3d8d4848b0c5111b0a37b82b86ad559fd2a745b44d8e8d9dfdc0cac00180d8f000000004341044a656f065871a353f216ca26cef8dde2f03e8c16202d2e8ad769f02032cb86a5eb5e56842e92e19141d60a01928f8dd2c875a390f67c1f6c94cfc617c0ea45afac00000000";

    fn sort_outputs(outputs: &mut Vec<TxOut>) {
        outputs.sort_by(cmp_outputs)
    }

    fn sort_inputs(inputs: &mut Vec<TxIn>) {
        inputs.sort_by(cmp_inputs)
    }

    fn tx() -> Transaction {
        let v = Vec::<u8>::from_hex(TX).unwrap();
        deserialize(&v).unwrap()
    }

    #[test]
    fn test_has_more_than_one_input_output() {
        let mut tx = tx();
        assert!(has_more_than_one_input_output(&tx));
        tx.input.pop();
        assert!(has_more_than_one_input_output(&tx));
        tx.output.pop();
        assert!(!has_more_than_one_input_output(&tx));
        tx.output.pop();
        assert!(!has_more_than_one_input_output(&tx));
    }

    #[test]
    fn test_tx() {
        let tx = tx();
        assert_eq!(tx.input.len(), 2);
        assert_eq!(tx.output.len(), 2);
        assert!(is_bip69(&tx));

        let mut tx_swap_inputs = tx.clone();
        tx_swap_inputs.input[0] = tx.input[1].clone();
        tx_swap_inputs.input[1] = tx.input[0].clone();
        assert!(!is_bip69(&tx_swap_inputs));

        let mut tx_swap_outputs = tx.clone();
        tx_swap_outputs.input[0] = tx.input[1].clone();
        tx_swap_outputs.input[1] = tx.input[0].clone();
        assert!(!is_bip69(&tx_swap_outputs));
    }

    #[test]
    fn sort_inputs_1() {
        let unsorted = vec![
            "643e5f4e66373a57251fb173151e838ccd27d279aca882997e005016bb53d5aa:0",
            "28e0fdd185542f2c6ea19030b0796051e7772b6026dd5ddccd7a2f93b73e6fc2:0",
            "f0a130a84912d03c1d284974f563c5949ac13f8342b8112edff52971599e6a45:0",
            "0e53ec5dfb2cb8a71fec32dc9a634a35b7e24799295ddd5278217822e0b31f57:0",
            "381de9b9ae1a94d9c17f6a08ef9d341a5ce29e2e60c36a52d333ff6203e58d5d:1",
            "f320832a9d2e2452af63154bc687493484a0e7745ebd3aaf9ca19eb80834ad60:0",
            "de0411a1e97484a2804ff1dbde260ac19de841bebad1880c782941aca883b4e9:1",
            "3b8b2f8efceb60ba78ca8bba206a137f14cb5ea4035e761ee204302d46b98de2:0",
            "54ffff182965ed0957dba1239c27164ace5a73c9b62a660c74b7b7f15ff61e7a:1",
            "bafd65e3c7f3f9fdfdc1ddb026131b278c3be1af90a4a6ffa78c4658f9ec0c85:0",
            "a5e899dddb28776ea9ddac0a502316d53a4a3fca607c72f66c470e0412e34086:0",
            "7a1de137cbafb5c70405455c49c5104ca3057a1f1243e6563bb9245c9c88c191:0",
            "26aa6e6d8b9e49bb0630aac301db6757c02e3619feb4ee0eea81eb1672947024:1",
            "402b2c02411720bf409eff60d05adad684f135838962823f3614cc657dd7bc0a:1",
            "7d037ceb2ee0dc03e82f17be7935d238b35d1deabf953a892a4507bfbeeb3ba4:1",
            "6c1d56f31b2de4bfc6aaea28396b333102b1f600da9c6d6149e96ca43f1102b1:1",
            "b4112b8f900a7ca0c8b0e7c4dfad35c6be5f6be46b3458974988e1cdb2fa61b8:0",
        ];

        let expected_sorted = vec![
            "0e53ec5dfb2cb8a71fec32dc9a634a35b7e24799295ddd5278217822e0b31f57:0",
            "26aa6e6d8b9e49bb0630aac301db6757c02e3619feb4ee0eea81eb1672947024:1",
            "28e0fdd185542f2c6ea19030b0796051e7772b6026dd5ddccd7a2f93b73e6fc2:0",
            "381de9b9ae1a94d9c17f6a08ef9d341a5ce29e2e60c36a52d333ff6203e58d5d:1",
            "3b8b2f8efceb60ba78ca8bba206a137f14cb5ea4035e761ee204302d46b98de2:0",
            "402b2c02411720bf409eff60d05adad684f135838962823f3614cc657dd7bc0a:1",
            "54ffff182965ed0957dba1239c27164ace5a73c9b62a660c74b7b7f15ff61e7a:1",
            "643e5f4e66373a57251fb173151e838ccd27d279aca882997e005016bb53d5aa:0",
            "6c1d56f31b2de4bfc6aaea28396b333102b1f600da9c6d6149e96ca43f1102b1:1",
            "7a1de137cbafb5c70405455c49c5104ca3057a1f1243e6563bb9245c9c88c191:0",
            "7d037ceb2ee0dc03e82f17be7935d238b35d1deabf953a892a4507bfbeeb3ba4:1",
            "a5e899dddb28776ea9ddac0a502316d53a4a3fca607c72f66c470e0412e34086:0",
            "b4112b8f900a7ca0c8b0e7c4dfad35c6be5f6be46b3458974988e1cdb2fa61b8:0",
            "bafd65e3c7f3f9fdfdc1ddb026131b278c3be1af90a4a6ffa78c4658f9ec0c85:0",
            "de0411a1e97484a2804ff1dbde260ac19de841bebad1880c782941aca883b4e9:1",
            "f0a130a84912d03c1d284974f563c5949ac13f8342b8112edff52971599e6a45:0",
            "f320832a9d2e2452af63154bc687493484a0e7745ebd3aaf9ca19eb80834ad60:0",
        ];

        sort_inputs_and_assert(unsorted, expected_sorted);
    }

    #[test]
    fn sort_inputs_2() {
        let unsorted = vec![
            "35288d269cee1941eaebb2ea85e32b42cdb2b04284a56d8b14dcc3f5c65d6055:1",
            "35288d269cee1941eaebb2ea85e32b42cdb2b04284a56d8b14dcc3f5c65d6055:0",
        ];

        let expected_sorted = vec![
            "35288d269cee1941eaebb2ea85e32b42cdb2b04284a56d8b14dcc3f5c65d6055:0",
            "35288d269cee1941eaebb2ea85e32b42cdb2b04284a56d8b14dcc3f5c65d6055:1",
        ];

        sort_inputs_and_assert(unsorted, expected_sorted);
    }

    fn sort_inputs_and_assert(unsorted: Vec<&str>, expected_sorted: Vec<&str>) {
        let mut inputs: Vec<_> = unsorted
            .into_iter()
            .map(|input| TxIn {
                previous_output: OutPoint::from_str(input).unwrap(),
                ..Default::default()
            })
            .collect();

        sort_inputs(&mut inputs);

        assert_eq!(inputs.len(), expected_sorted.len());

        for (actual, expected) in inputs
            .iter()
            .map(|txin| txin.previous_output.to_string())
            .zip(expected_sorted)
        {
            assert_eq!(&actual, expected);
        }
    }

    #[test]
    fn sort_outputs_1() {
        let unsorted = vec![
            (
                40000000000,
                "76a9145be32612930b8323add2212a4ec03c1562084f8488ac",
            ),
            (
                400057456,
                "76a9144a5fba237213a062f6f57978f796390bdcf8d01588ac",
            ),
        ];

        let expected_sorted = vec![
            (
                400057456,
                "76a9144a5fba237213a062f6f57978f796390bdcf8d01588ac",
            ),
            (
                40000000000,
                "76a9145be32612930b8323add2212a4ec03c1562084f8488ac",
            ),
        ];

        sort_outputs_and_assert(unsorted, expected_sorted);
    }

    #[test]
    fn sort_outputs_2() {
        let unsorted = vec![
            (
                2400000000,
                "41044a656f065871a353f216ca26cef8dde2f03e8c16202d2e8ad769f02032cb86a5eb5e56842e92e19141d60a01928f8dd2c875a390f67c1f6c94cfc617c0ea45afac"
            ),
            (
                100000000,
                "41046a0765b5865641ce08dd39690aade26dfbf5511430ca428a3089261361cef170e3929a68aee3d8d4848b0c5111b0a37b82b86ad559fd2a745b44d8e8d9dfdc0cac"
            ),
        ];

        let expected_sorted = vec![
            (
                100000000,
                "41046a0765b5865641ce08dd39690aade26dfbf5511430ca428a3089261361cef170e3929a68aee3d8d4848b0c5111b0a37b82b86ad559fd2a745b44d8e8d9dfdc0cac"
            ),
            (
                2400000000,
                "41044a656f065871a353f216ca26cef8dde2f03e8c16202d2e8ad769f02032cb86a5eb5e56842e92e19141d60a01928f8dd2c875a390f67c1f6c94cfc617c0ea45afac"
            ),
        ];

        sort_outputs_and_assert(unsorted, expected_sorted);
    }

    #[test]
    fn sort_outputs_3() {
        let unsorted = vec![
            (1000, "76a9145be32612930b8323add2212a4ec03c1562084f8488ac"),
            (1000, "76a9144a5fba237213a062f6f57978f796390bdcf8d01588ac"),
        ];

        let expected_sorted = vec![
            (1000, "76a9144a5fba237213a062f6f57978f796390bdcf8d01588ac"),
            (1000, "76a9145be32612930b8323add2212a4ec03c1562084f8488ac"),
        ];

        sort_outputs_and_assert(unsorted, expected_sorted);
    }

    fn sort_outputs_and_assert(unsorted: Vec<(u64, &str)>, expected_sorted: Vec<(u64, &str)>) {
        let mut outputs: Vec<_> = unsorted
            .into_iter()
            .map(|(value, scriptpubkey)| TxOut {
                value: bitcoin::Amount::from_sat(value),
                script_pubkey: ScriptBuf::from_bytes(Vec::from_hex(scriptpubkey).unwrap()),
            })
            .collect();

        sort_outputs(&mut outputs);

        assert_eq!(outputs.len(), expected_sorted.len());

        for (actual, expected) in outputs.iter().zip(expected_sorted) {
            assert_eq!(actual.value, bitcoin::Amount::from_sat(expected.0));
            assert_eq!(
                actual.script_pubkey,
                ScriptBuf::from_bytes(Vec::from_hex(expected.1).unwrap())
            );
        }
    }
}
