use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::{to_label_map, Page};
use crate::process::TxStats;

pub fn rounded_amount(tx_stats: &TxStats) -> Page {
    let mut charts = vec![];

    let (vec, mul) = tx_stats.rounded_amount_per_period.finish();
    let map = to_label_map(&vec, mul);
    let labels: Vec<_> = map.keys().cloned().collect();

    let mut chart = Chart::new("Rounded amount [-]".to_string(), Kind::Line, labels);

    let dataset = Dataset {
        label: "rounded amounts".to_string(),
        data: vec,
        background_color: vec![Color::Blue],
        border_color: vec![Color::Blue],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    charts.push(chart);

    Page {
        title: "Rounded amount".to_string(),
        description: "Charts showing the number of outputs which have a rounded amount as value (multiple of 1000)".to_string(),
        permalink: "rounded-amount".to_string(),
        charts,
        text: "".to_string(),
    }
}
