use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::Page;
use crate::process::OpReturnData;

pub fn op_return_sizes(opret: &OpReturnData) -> Page {
    let description = "Chart counting the number of OP_RETURN scripts per bucket of different sizes since inception.";
    let mut charts = vec![];
    let map = &opret.op_ret_size;
    let size_labels: Vec<_> = map.keys().cloned().collect();

    // Partition the data: first 10 elements and the rest
    let (first_labels, remaining_labels) = if size_labels.len() > 10 {
        let (first, rest) = size_labels.split_at(10);
        (first.to_vec(), rest.to_vec())
    } else {
        (size_labels.clone(), vec![])
    };

    // First chart with first 10 elements
    if !first_labels.is_empty() {
        let first_data: Vec<_> = first_labels.iter().map(|label| map[label]).collect();
        let mut chart1 = Chart::new(
            format!("{} (first 10)", description),
            Kind::Bar,
            first_labels,
        );

        let dataset1 = Dataset {
            label: "count".to_string(),
            data: first_data,
            background_color: vec![Color::Red],
            border_color: vec![],
            fill: true,
            ..Default::default()
        };
        chart1.add_dataset(dataset1, None);
        charts.push(chart1);
    }

    // Second chart with remaining elements
    if !remaining_labels.is_empty() {
        let remaining_data: Vec<_> = remaining_labels.iter().map(|label| map[label]).collect();
        let mut chart2 = Chart::new(
            format!("{} (remaining)", description),
            Kind::Bar,
            remaining_labels,
        );

        let dataset2 = Dataset {
            label: "count".to_string(),
            data: remaining_data,
            background_color: vec![Color::Red],
            border_color: vec![],
            fill: true,
            ..Default::default()
        };
        chart2.add_dataset(dataset2, None);
        charts.push(chart2);
    }

    Page {
        title: "OP_RETURN sizes".to_string(),
        description: description.to_string(),
        permalink: "op-return-sizes".to_string(),
        charts,
        text: "".to_string(),
    }
}
