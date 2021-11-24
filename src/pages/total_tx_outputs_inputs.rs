use crate::charts::{Chart, Color, Dataset, Kind};
use crate::counter::perc_1000;
use crate::pages::{to_label_map, Page};
use crate::process::TxStats;

pub fn total_tx_outputs_inputs(tx_stats: &TxStats) -> Page {
    let mut charts = vec![];
    let (vec, div) = tx_stats.total_outputs_per_month.finish();
    let map = to_label_map(&vec, div);
    let labels: Vec<_> = map.keys().cloned().collect();

    let mut chart = Chart::new("Tx, inputs and outputs".to_string(), Kind::Line, labels);

    let dataset = Dataset {
        label: "Total tx".to_string(),
        data: tx_stats.total_tx_per_month.finish().0,
        background_color: vec![Color::Blue],
        border_color: vec![Color::Blue],
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "Total inputs".to_string(),
        data: tx_stats.total_inputs_per_month.finish().0,
        background_color: vec![Color::Orange],
        border_color: vec![Color::Orange],
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "Total outputs".to_string(),
        data: tx_stats.total_outputs_per_month.finish().0,
        background_color: vec![Color::Red],
        border_color: vec![Color::Red],
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let perc_outputs = perc_1000(
        &tx_stats.total_inputs_per_month.finish().0,
        &tx_stats.total_tx_per_month.finish().0,
    );
    let dataset = Dataset {
        label: "Average Outputs *1000".to_string(),
        data: perc_outputs,
        background_color: vec![Color::Purple],
        border_color: vec![Color::Purple],
        border_dash: Some([5, 5]),
        ..Default::default()
    };
    chart.add_dataset(dataset, Some("y2".to_string()));

    let perc_inputs = perc_1000(
        &tx_stats.total_outputs_per_month.finish().0,
        &tx_stats.total_tx_per_month.finish().0,
    );
    let dataset = Dataset {
        label: "Average inputs *1000".to_string(),
        data: perc_inputs,
        background_color: vec![Color::Green],
        border_color: vec![Color::Green],
        border_dash: Some([5, 5]),
        ..Default::default()
    };
    chart.add_dataset(dataset, Some("y2".to_string()));

    charts.push(chart);

    Page {
        title: "Tx, inputs and outputs".to_string(),
        description: "Charts showing total number of transactions, inputs and outputs per month"
            .to_string(),
        permalink: "total-tx-outputs-inputs".to_string(),
        charts,
    }
}
