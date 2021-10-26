use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::{to_label_map, Page};
use crate::process::ScriptType;

pub fn script_types(script_type: &ScriptType) -> Page {
    let mut charts = vec![];

    let labels: Vec<_> = to_label_map(&script_type.all).keys().cloned().collect();

    let mut chart = Chart::new("Script types [-]".to_string(), Kind::Line, labels);

    let dataset = Dataset {
        label: "All".to_string(),
        data: script_type.all.clone(),
        background_color: vec![Color::Blue],
        border_color: vec![Color::Blue],
        fill: false,
        hidden: true,
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "p2pkh".to_string(),
        data: script_type.p2pkh.clone(),
        background_color: vec![Color::Green],
        border_color: vec![Color::Green],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "p2pk".to_string(),
        data: script_type.p2pk.clone(),
        background_color: vec![Color::Red],
        border_color: vec![Color::Red],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "v0_p2wpkh".to_string(),
        data: script_type.v0_p2wpkh.clone(),
        background_color: vec![Color::Yellow],
        border_color: vec![Color::Yellow],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "v0_p2wsh".to_string(),
        data: script_type.v0_p2wsh.clone(),
        background_color: vec![Color::Orange],
        border_color: vec![Color::Orange],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "p2sh".to_string(),
        data: script_type.p2sh.clone(),
        background_color: vec![Color::Purple],
        border_color: vec![Color::Purple],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "Other".to_string(),
        data: script_type.other.clone(),
        background_color: vec![Color::Grey],
        border_color: vec![Color::Grey],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset);

    charts.push(chart);

    Page {
        title: "Script types".to_string(),
        description: "Charts showing the script types per month.".to_string(),
        permalink: "script-types".to_string(),
        charts,
    }
}