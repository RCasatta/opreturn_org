use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::{to_label_map, Page};
use crate::process::ScriptType;

pub fn script_types(script_type: &ScriptType) -> Page {
    let mut charts = vec![];

    let (vec, div) = script_type.all.finish();
    let labels: Vec<_> = to_label_map(&vec, div).keys().cloned().collect();

    let mut chart = Chart::new("Script types [-]".to_string(), Kind::Line, labels);

    let dataset = Dataset {
        label: "All".to_string(),
        data: vec,
        background_color: vec![Color::Blue],
        border_color: vec![Color::Blue],
        hidden: true,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "p2pkh".to_string(),
        data: script_type.p2pkh.finish().0,
        background_color: vec![Color::Green],
        border_color: vec![Color::Green],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "p2pk".to_string(),
        data: script_type.p2pk.finish().0,
        background_color: vec![Color::Red],
        border_color: vec![Color::Red],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "v0_p2wpkh".to_string(),
        data: script_type.v0_p2wpkh.finish().0,
        background_color: vec![Color::Yellow],
        border_color: vec![Color::Yellow],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "v0_p2wsh".to_string(),
        data: script_type.v0_p2wsh.finish().0,
        background_color: vec![Color::Orange],
        border_color: vec![Color::Orange],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let color = Color::Custom(55, 11, 122, 0.8);
    let dataset = Dataset {
        label: "p2tr".to_string(),
        data: script_type.p2tr.finish().0,
        background_color: vec![color],
        border_color: vec![color],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "p2sh".to_string(),
        data: script_type.p2sh.finish().0,
        background_color: vec![Color::Purple],
        border_color: vec![Color::Purple],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    let dataset = Dataset {
        label: "Other".to_string(),
        data: script_type.other.finish().0,
        background_color: vec![Color::Grey],
        border_color: vec![Color::Grey],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);

    charts.push(chart);

    Page {
        title: "Script types".to_string(),
        description: "Charts showing the script types per month.".to_string(),
        permalink: "script-types".to_string(),
        charts,
    }
}
