use crate::charts::{Chart, Color, Dataset};
use crate::pages::Page;
use crate::process::ScriptType;

pub fn script_types(script_type: &ScriptType) -> Page {
    let (all, mul) = script_type.all.finish();
    let mut chart = Chart::periods("Script types [-]", all.len(), mul).with(Dataset {
        hidden: true,
        ..Dataset::line("All", all, Color::Blue)
    });
    for (label, counter, color) in [
        ("p2pkh", &script_type.p2pkh, Color::Green),
        ("p2pk", &script_type.p2pk, Color::Red),
        ("v0_p2wpkh", &script_type.v0_p2wpkh, Color::Yellow),
        ("v0_p2wsh", &script_type.v0_p2wsh, Color::Orange),
        ("p2tr", &script_type.p2tr, Color::Custom(55, 11, 122, 0.8)),
        ("p2sh", &script_type.p2sh, Color::Purple),
        ("Other", &script_type.other, Color::Grey),
    ] {
        chart = chart.with(Dataset::line(label, counter.finish().0, color));
    }

    Page {
        title: "Script types".to_string(),
        description: "Charts showing the script types per month.".to_string(),
        permalink: "script-types".to_string(),
        charts: vec![chart],
        stats: vec![
            ("all".to_string(), script_type.all.sum_all()),
            ("p2pkh".to_string(), script_type.p2pkh.sum_all()),
            ("p2pk".to_string(), script_type.p2pk.sum_all()),
            ("v0_p2wpkh".to_string(), script_type.v0_p2wpkh.sum_all()),
            ("v0_p2wsh".to_string(), script_type.v0_p2wsh.sum_all()),
            ("p2tr".to_string(), script_type.p2tr.sum_all()),
            ("p2sh".to_string(), script_type.p2sh.sum_all()),
            ("other".to_string(), script_type.other.sum_all()),
        ],
        ..Default::default()
    }
}
