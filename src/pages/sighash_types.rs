use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::{map_by_value, Page};
use crate::process_stats::Stats;

pub fn sighash_types(stats: &Stats) -> Page {
    let map = map_by_value(&stats.sighashtype);
    let mut chart = Chart::new(
        "Number of sighash type used".to_string(),
        Kind::Pie,
        map.keys().cloned().collect(),
    );
    let dataset = Dataset {
        label: "chart1".to_string(),
        data: map.values().cloned().collect(),
        background_color: Color::rainbow(),
        border_color: vec![],
        fill: true,
        hidden: false,
    };
    chart.add_dataset(dataset);

    Page {
        title: "Number of sighash type used".to_string(),
        description: "Show distribution of sighash type used".to_string(),
        permalink: "sighash-types".to_string(),
        charts: vec![chart],
    }
}
