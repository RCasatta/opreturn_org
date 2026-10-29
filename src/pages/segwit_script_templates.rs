use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::{map_by_value, Page};
use crate::process::segwit_templates::{is_envelope, OTHER};
use crate::process::{ScriptType, SegwitTemplates};
use std::collections::HashMap;

/// Pie charts of spends and distinct scripts per template, and the histogram of templates by
/// anonymity set size, considering only envelope or only non-envelope templates
fn charts(templates: &SegwitTemplates, envelope: bool, what: &str) -> Vec<Chart> {
    let selected: Vec<_> = templates
        .templates
        .iter()
        .filter(|(k, _)| k.as_str() != OTHER && is_envelope(k) == envelope)
        .collect();
    let spends: HashMap<String, u64> = selected
        .iter()
        .map(|(k, v)| ((*k).clone(), v.spends))
        .collect();
    let distinct: HashMap<String, u64> = selected
        .iter()
        .map(|(k, v)| ((*k).clone(), v.distinct.count()))
        .collect();

    let mut charts = vec![];
    for (title, map) in [
        (format!("Spends per {}", what), spends),
        (format!("Distinct scripts per {}", what), distinct),
    ] {
        let map = map_by_value(&map);
        let mut chart = Chart::new(title.clone(), Kind::Pie, map.keys().cloned().collect());
        let dataset = Dataset {
            label: title,
            data: map.values().cloned().collect(),
            background_color: Color::rainbow(),
            border_color: vec![],
            fill: true,
            ..Default::default()
        };
        chart.add_dataset(dataset, None);
        charts.push(chart);
    }

    let histogram = templates.anonymity_set_histogram(envelope);
    let mut chart = Chart::new(
        format!(
            "Number of {}s by count of distinct scripts using it (anonymity set)",
            what
        ),
        Kind::Bar,
        histogram.iter().map(|(l, _)| l.clone()).collect(),
    );
    let dataset = Dataset {
        label: format!("{}s", what),
        data: histogram.iter().map(|(_, c)| *c).collect(),
        background_color: vec![Color::Blue],
        border_color: vec![Color::Blue],
        ..Default::default()
    };
    chart.add_dataset(dataset, None);
    charts.push(chart);
    charts
}

pub fn segwit_script_templates(script_type: &ScriptType) -> Page {
    let templates = &script_type.segwit_templates;

    // envelopes (mostly inscriptions) dominate taproot script spends, they are shown separately
    let mut all_charts = charts(templates, false, "script template");
    all_charts.extend(charts(templates, true, "envelope template"));

    let (envelope_templates, envelope_spends) = templates
        .templates
        .iter()
        .filter(|(k, _)| is_envelope(k))
        .fold((0, 0), |(t, s), (_, v)| (t + 1, s + v.spends));

    Page {
        title: "Segwit script templates".to_string(),
        description: "Scripts revealed spending P2WSH, P2SH-P2WSH and taproot script path, with keys and hashes replaced by placeholders".to_string(),
        permalink: "segwit-script-templates".to_string(),
        charts: all_charts,
        text: format!(
            "{} distinct templates over {} spends, of which {} templates and {} spends contain an OP_0 OP_IF envelope (eg. inscriptions) and are shown in the last three charts. Full list in raw/segwit_templates.json",
            templates.templates.len(),
            templates.total_spends,
            envelope_templates,
            envelope_spends,
        ),
    }
}
