use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::Page;
use crate::process::OpReturnData;

pub fn op_return_sizes(opret: &OpReturnData) -> Page {
    let description = "Chart counting the number of OP_RETURN scripts per bucket of different sizes since inception.";
    let mut charts = vec![];
    let map = &opret.op_ret_size;
    let size_labels: Vec<_> = map.keys().cloned().collect();

    let mut chart = Chart::new(description.to_string(), Kind::Bar, size_labels);

    let dataset = Dataset {
        label: "count".to_string(),
        data: map.values().cloned().collect(),
        background_color: vec![Color::Red],
        border_color: vec![],
        fill: true,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    charts.push(chart);

    Page {
        title: "OP_RETURN sizes".to_string(),
        description: description.to_string(),
        permalink: "op-return-sizes".to_string(),
        charts,
        text: "".to_string(),
    }
}
