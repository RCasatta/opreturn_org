use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::{cumulative, to_label_map, Page};
use crate::process::{Bip158Stats, Stats};

pub fn blockchain_and_filter_size(stats: &Stats, bip158: &Bip158Stats) -> Page {
    let blockchain = to_label_map(&cumulative(&stats.block_size_per_month));
    let filters = to_label_map(&cumulative(&bip158.bip158_filter_size_per_month));

    let mut charts = vec![];

    let blockchain_labels: Vec<_> = blockchain.keys().cloned().collect();
    let filters_labels: Vec<_> = filters.keys().cloned().collect();
    assert_eq!(blockchain_labels, filters_labels);
    let mut chart = Chart::new(
        "Blockchain and BIP158 filter size [MB]".to_string(),
        Kind::Line,
        blockchain_labels,
    );

    let dataset = Dataset {
        label: "Blockchain size".to_string(),
        data: blockchain.values().map(|e| *e >> 20).collect(),
        background_color: vec![Color::Orange],
        border_color: vec![Color::Orange],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "BIP158 filter size".to_string(),
        data: filters.values().map(|e| *e >> 20).collect(),
        background_color: vec![Color::Red],
        border_color: vec![Color::Red],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    charts.push(chart);

    Page {
        title: "Blockhain and BIP158 filter size".to_string(),
        description: "Charts showing the Blockchain size and BIP158 filter size".to_string(),
        permalink: "blockchain-and-filter-size".to_string(),
        charts,
    }
}
