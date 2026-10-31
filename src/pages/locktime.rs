use crate::charts::{Chart, Color, Dataset};
use crate::counter::cumulative;
use crate::pages::Page;
use crate::process::TxStats;

pub fn locktime(tx_stats: &TxStats) -> Page {
    let title = "Number of tx non-deeply reorgable";

    let (no_reorg, mul) = tx_stats.non_deeply_reorgable.finish();
    let (all, _) = tx_stats.total_tx_per_period.finish();
    let chart = Chart::periods(title, no_reorg.len(), mul)
        .with(Dataset::line(
            "non reorgable",
            cumulative(&no_reorg),
            Color::Orange,
        ))
        .with(Dataset::line("all", cumulative(&all), Color::Red));

    Page {
        title: title.to_string(),
        description:
            "Number of transactions with nlocktime greater than confirmation height minus 6"
                .to_string(),
        permalink: "locktime".to_string(),
        charts: vec![chart],
        ..Default::default()
    }
}
