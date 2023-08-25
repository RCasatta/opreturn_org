use crate::charts::{Chart, Color, Dataset, Kind};
use crate::counter::cumulative;
use crate::pages::{to_label_map, Page};
use crate::process::{Bip158Stats, Stats, TxStats};

pub fn blockchain_sizes(stats: &Stats, bip158: &Bip158Stats, tx_stats: &TxStats) -> Page {
    let title = "Blockhain sizes".to_string();

    let (vec, mul) = stats.block_size_per_period.finish();
    let blockchain = to_label_map(&cumulative(&vec), mul);
    let (vec, mul) = bip158.bip158_filter_size_per_period.finish();
    let filters = to_label_map(&cumulative(&vec), mul);
    let (vec, mul) = stats.witness_size_per_period.finish();
    let witness = to_label_map(&cumulative(&vec), mul);
    let (vec, mul) = stats.script_sig_size_per_period.finish();
    let script_sig = to_label_map(&cumulative(&vec), mul);
    let (vec, mul) = tx_stats.script_pubkey_size_per_period.finish();
    let script_pubkey = to_label_map(&cumulative(&vec), mul);

    let mut charts = vec![];

    let blockchain_labels: Vec<_> = blockchain.keys().cloned().collect();
    let filters_labels: Vec<_> = filters.keys().cloned().collect();
    assert_eq!(blockchain_labels, filters_labels);
    let mut chart = Chart::new(title.clone(), Kind::Line, blockchain_labels);

    let dataset = Dataset {
        label: "Blockchain size [MB]".to_string(),
        data: blockchain.values().map(|e| *e >> 20).collect(),
        background_color: vec![Color::Orange],
        border_color: vec![Color::Orange],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "BIP158 filter size [MB]".to_string(),
        data: filters.values().map(|e| *e >> 20).collect(),
        background_color: vec![Color::Red],
        border_color: vec![Color::Red],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "Witnesses size [MB]".to_string(),
        data: witness.values().map(|e| *e >> 20).collect(),
        background_color: vec![Color::Yellow],
        border_color: vec![Color::Yellow],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "Script sig size [MB]".to_string(),
        data: script_sig.values().map(|e| *e >> 20).collect(),
        background_color: vec![Color::Green],
        border_color: vec![Color::Green],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "Script pubkey size [MB]".to_string(),
        data: script_pubkey.values().map(|e| *e >> 20).collect(),
        background_color: vec![Color::Blue],
        border_color: vec![Color::Blue],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    charts.push(chart);

    Page {
        title: title,
        description: "Megabyte size of: overall blockchain, BIP158 filters, witnesses, script sigs and script pubkey".to_string(),
        permalink: "blockchain-sizes".to_string(),  // old "blockchain-and-filter-size"
        charts,
        text: "".to_string(),
    }
}
