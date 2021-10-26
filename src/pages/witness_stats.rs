use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::{map_by_value, Page};
use crate::process::Stats;

pub fn witness_stats(stats: &Stats) -> Page {
    let has_witness = map_by_value(&stats.has_witness);
    let mut chart1 = Chart::new(
        "Inputs with or without elements in witness".to_string(),
        Kind::Pie,
        has_witness.keys().cloned().collect(),
    );
    let dataset = Dataset {
        label: "chart1".to_string(),
        data: has_witness.values().cloned().collect(),
        background_color: Color::rainbow(),
        border_color: vec![],
        fill: true,
        ..Default::default()
    };
    chart1.add_dataset(dataset);

    let witness_elements = map_by_value(&stats.witness_elements);
    let mut chart2 = Chart::new(
        "Number of elements in non-empty witness".to_string(),
        Kind::Pie,
        witness_elements.keys().cloned().collect(),
    );
    let dataset = Dataset {
        label: "chart2".to_string(),
        data: witness_elements.values().cloned().collect(),
        background_color: Color::rainbow(),
        border_color: vec![],
        fill: true,
        ..Default::default()
    };
    chart2.add_dataset(dataset);

    let witness_byte_size = map_by_value(&stats.witness_byte_size);
    let mut chart3 = Chart::new(
        "Bytes in non-empty witness".to_string(),
        Kind::Pie,
        witness_byte_size.keys().cloned().collect(),
    );
    let dataset = Dataset {
        label: "chart3".to_string(),
        data: witness_byte_size.values().cloned().collect(),
        background_color: Color::rainbow(),
        border_color: vec![],
        fill: true,
        ..Default::default()
    };
    chart3.add_dataset(dataset);

    Page {
        title: "Witness stats".to_string(),
        description: "Stats about the witnesses, number of elements and bytes used".to_string(),
        permalink: "witness-stats".to_string(),
        charts: vec![chart1, chart2, chart3],
    }
}
