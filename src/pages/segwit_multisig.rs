use crate::charts::Chart;
use crate::pages::Page;
use crate::process::ScriptType;

pub fn segwit_multisig(script_type: &ScriptType) -> Page {
    let description = "Analyze segwit input scripts counting the occurence of the NofM";
    Page {
        title: "Native segwit multisig".to_string(),
        description: description.to_string(),
        permalink: "segwit-multisig".to_string(),
        charts: vec![Chart::pie(description, &script_type.multisig)],
        ..Default::default()
    }
}
