use crate::charts::{Chart, Color, Dataset};
use crate::counter::{cumulative, Counter};
use crate::pages::Page;
use crate::process::{Bip158Stats, Stats, TxStats};

/// Cumulative size in megabytes
fn megabytes(counter: &Counter) -> Vec<u64> {
    cumulative(&counter.finish().0)
        .iter()
        .map(|bytes| bytes >> 20)
        .collect()
}

pub fn blockchain_sizes(stats: &Stats, bip158: &Bip158Stats, tx_stats: &TxStats) -> Page {
    let title = "Blockchain sizes";

    let (blocks, mul) = stats.block_size_per_period.finish();
    let (filters, filters_mul) = bip158.bip158_filter_size_per_period.finish();
    assert_eq!((blocks.len(), mul), (filters.len(), filters_mul));
    let chart = Chart::periods(title, blocks.len(), mul)
        .with(Dataset::line(
            "Blockchain size [MB]",
            megabytes(&stats.block_size_per_period),
            Color::Orange,
        ))
        .with(Dataset::line(
            "BIP158 filter size [MB]",
            megabytes(&bip158.bip158_filter_size_per_period),
            Color::Red,
        ))
        .with(Dataset::line(
            "Witnesses size [MB]",
            megabytes(&stats.witness_size_per_period),
            Color::Yellow,
        ))
        .with(Dataset::line(
            "Script sig size [MB]",
            megabytes(&stats.script_sig_size_per_period),
            Color::Green,
        ))
        .with(Dataset::line(
            "Script pubkey size [MB]",
            megabytes(&tx_stats.script_pubkey_size_per_period),
            Color::Blue,
        ));

    Page {
        title: title.to_string(),
        description: "Megabyte size of: overall blockchain, BIP158 filters, witnesses, script sigs and script pubkey".to_string(),
        permalink: "blockchain-sizes".to_string(), // old "blockchain-and-filter-size"
        charts: vec![chart],
        ..Default::default()
    }
}
