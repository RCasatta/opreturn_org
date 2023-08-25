use crate::charts::{Chart, Color, Dataset, Kind};
use crate::counter::cumulative;
use crate::pages::{to_label_map, Page};
use crate::process::TxStats;

pub fn locktime(tx_stats: &TxStats) -> Page {
    let title = "Number of tx non-deeply reorgable".to_string();

    let (no_vec, mul) = tx_stats.non_deeply_reorgable.finish();
    let no_reorg = to_label_map(&cumulative(&no_vec), mul);
    let (all_vec, mul) = tx_stats.total_tx_per_period.finish();
    let all = to_label_map(&cumulative(&all_vec), mul);

    let mut charts = vec![];

    let labels: Vec<_> = no_reorg.keys().cloned().collect();
    let mut chart = Chart::new(title.clone(), Kind::Line, labels);

    let dataset = Dataset {
        label: "non reorgable".to_string(),
        data: no_reorg.values().cloned().collect(),
        background_color: vec![Color::Orange],
        border_color: vec![Color::Orange],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "all".to_string(),
        data: all.values().cloned().collect(),
        background_color: vec![Color::Red],
        border_color: vec![Color::Red],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    charts.push(chart);

    Page {
        title,
        description:
            "Number of transactions with nlocktime greater than confirmation height minus 6"
                .to_string(),
        permalink: "locktime".to_string(),
        charts,
        text: "".to_string(),
    }
}
