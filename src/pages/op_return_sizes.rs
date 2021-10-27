use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::Page;
use crate::process::OpReturnData;

pub fn op_return_sizes(opret: &OpReturnData) -> Page {
    let mut charts = vec![];
    let map = &opret.op_ret_size;

    let size_labels: Vec<_> = map.keys().cloned().collect();

    let mut chart = Chart::new("OP_RETURN sizes".to_string(), Kind::Bar, size_labels);

    let dataset = Dataset {
        label: "size [kbytes]".to_string(),
        data: map.values().map(|e| *e >> 10).collect(),
        background_color: vec![Color::Red],
        border_color: vec![],
        fill: true,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    charts.push(chart);

    Page {
        title: "OP_RETURN sizes".to_string(),
        description:
            "Chart showing the distribution of the sizes of the OP_RETURN scripts since inception."
                .to_string(),
        permalink: "op-return-sizes".to_string(),
        charts,
    }
}
