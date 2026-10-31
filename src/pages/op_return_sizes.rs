use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::Page;
use crate::process::OpReturnData;

pub fn op_return_sizes(opret: &OpReturnData) -> Page {
    let description = "Chart counting the number of OP_RETURN scripts per bucket of different sizes since inception.";
    let map = &opret.op_ret_size;
    let labels: Vec<_> = map.keys().cloned().collect();

    // the first 10 sizes are much more common than the others, split in two charts
    let (first, remaining) = labels.split_at(labels.len().min(10));
    let charts = [(first, "first 10"), (remaining, "remaining")]
        .into_iter()
        .filter(|(labels, _)| !labels.is_empty())
        .map(|(labels, which)| {
            Chart::new(
                format!("{} ({})", description, which),
                Kind::Bar,
                labels.to_vec(),
            )
            .with(Dataset {
                label: "count".to_string(),
                data: labels.iter().map(|label| map[label]).collect(),
                background_color: vec![Color::Red],
                fill: true,
                ..Default::default()
            })
        })
        .collect();

    Page {
        title: "OP_RETURN sizes".to_string(),
        description: description.to_string(),
        permalink: "op-return-sizes".to_string(),
        charts,
        ..Default::default()
    }
}
