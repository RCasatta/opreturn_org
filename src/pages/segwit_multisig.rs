use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::{map_by_value, Page};
use crate::process::ScriptType;

pub fn segwit_multisig(script_type: &ScriptType) -> Page {
    let map = map_by_value(&script_type.multisig);
    let mut chart = Chart::new(
        "Analyze segwit input scripts counting the occurence of the NofM".to_string(),
        Kind::Pie,
        map.keys().cloned().collect(),
    );
    let dataset = Dataset {
        label: "chart1".to_string(),
        data: map.values().cloned().collect(),
        background_color: Color::rainbow(),
        border_color: vec![],
        fill: true,
        ..Default::default()
    };
    chart.add_dataset(dataset);

    Page {
        title: "Native segwit multisig".to_string(),
        description: "Analyze segwit input scripts counting the occurence of the NofM".to_string(),
        permalink: "segwit-multisig".to_string(),
        charts: vec![chart],
    }
}
