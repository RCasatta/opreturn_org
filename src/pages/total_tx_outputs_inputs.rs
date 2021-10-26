use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::{to_label_map, Page};
use crate::process::Stats;

pub fn total_tx_outputs_inputs(stats: &Stats) -> Page {
    let mut charts = vec![];
    let map = to_label_map(&stats.total_outputs_per_month);
    let labels: Vec<_> = map.keys().cloned().collect();

    let mut chart = Chart::new("Tx, inputs and outputs".to_string(), Kind::Line, labels);

    let dataset = Dataset {
        label: "Total tx".to_string(),
        data: stats.total_tx_per_month.clone(),
        background_color: vec![Color::Blue],
        border_color: vec![Color::Blue],
        ..Default::default()
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "Total inputs".to_string(),
        data: stats.total_inputs_per_month.clone(),
        background_color: vec![Color::Orange],
        border_color: vec![Color::Orange],
        ..Default::default()
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "Total outputs".to_string(),
        data: stats.total_outputs_per_month.clone(),
        background_color: vec![Color::Red],
        border_color: vec![Color::Red],
        ..Default::default()
    };
    chart.add_dataset(dataset);

    //TODO add percentage ot outputs_per_tx and inputs_per

    charts.push(chart);

    Page {
        title: "Tx, inputs and outputs".to_string(),
        description: "Charts showing total number of transactions, inputs and outputs per month"
            .to_string(),
        permalink: "total-tx-outputs-inputs".to_string(),
        charts,
    }
}
