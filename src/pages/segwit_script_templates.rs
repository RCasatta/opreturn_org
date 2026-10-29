use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::{map_by_value, Page};
use crate::process::ScriptType;
use std::collections::HashMap;

pub fn segwit_script_templates(script_type: &ScriptType) -> Page {
    let templates = &script_type.segwit_templates;
    let mut charts = vec![];

    let spends: HashMap<String, u64> = templates
        .templates
        .iter()
        .map(|(k, v)| (k.clone(), v.spends))
        .collect();
    let distinct: HashMap<String, u64> = templates
        .templates
        .iter()
        .map(|(k, v)| (k.clone(), v.distinct.count()))
        .collect();

    for (title, map) in [
        ("Spends per script template", spends),
        ("Distinct scripts per script template", distinct),
    ] {
        let map = map_by_value(&map);
        let mut chart = Chart::new(title.to_string(), Kind::Pie, map.keys().cloned().collect());
        let dataset = Dataset {
            label: title.to_string(),
            data: map.values().cloned().collect(),
            background_color: Color::rainbow(),
            border_color: vec![],
            fill: true,
            ..Default::default()
        };
        chart.add_dataset(dataset, None);
        charts.push(chart);
    }

    let histogram = templates.anonymity_set_histogram();
    let mut chart = Chart::new(
        "Number of templates by count of distinct scripts using it (anonymity set)".to_string(),
        Kind::Bar,
        histogram.iter().map(|(l, _)| l.clone()).collect(),
    );
    let dataset = Dataset {
        label: "templates".to_string(),
        data: histogram.iter().map(|(_, c)| *c).collect(),
        background_color: vec![Color::Blue],
        border_color: vec![Color::Blue],
        ..Default::default()
    };
    chart.add_dataset(dataset, None);
    charts.push(chart);

    Page {
        title: "Segwit script templates".to_string(),
        description: "Scripts revealed spending P2WSH, P2SH-P2WSH and taproot script path, with keys and hashes replaced by placeholders".to_string(),
        permalink: "segwit-script-templates".to_string(),
        charts,
        text: format!(
            "{} distinct templates over {} spends. Full list in raw/segwit_templates.json",
            templates.templates.len(),
            templates.total_spends
        ),
    }
}
