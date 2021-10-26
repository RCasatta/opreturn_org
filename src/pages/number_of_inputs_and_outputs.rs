use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::{map_by_value, Page};
use crate::process::TxStats;

pub fn number_of_inputs_and_outputs(stats: &TxStats) -> Page {
    let mut charts = vec![];
    let map = map_by_value(&stats.in_out);

    let in_out_labels: Vec<_> = map.keys().cloned().collect();

    let mut chart = Chart::new(
        "Number of inputs and outputs".to_string(),
        Kind::Pie,
        in_out_labels,
    );

    let dataset = Dataset {
        label: "Number of inputs and outputs".to_string(),
        data: map.values().cloned().collect(),
        background_color: Color::rainbow(),
        border_color: vec![],
        fill: true,
        ..Default::default()
    };
    chart.add_dataset(dataset);

    charts.push(chart);

    Page {
        title: "Number of inputs and outputs".to_string(),
        description: "Show how many txs have A input and B output".to_string(),
        permalink: "number-of-inputs-and-outputs".to_string(),
        charts,
    }
}
