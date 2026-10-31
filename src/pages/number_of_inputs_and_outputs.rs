use crate::charts::Chart;
use crate::pages::Page;
use crate::process::TxStats;

pub fn number_of_inputs_and_outputs(stats: &TxStats) -> Page {
    let title = "Number of inputs and outputs";
    Page {
        title: title.to_string(),
        description: "Show how many txs have A input and B output".to_string(),
        permalink: "number-of-inputs-and-outputs".to_string(),
        charts: vec![Chart::pie(title, &stats.in_out)],
        stats: vec![(
            "tx with more than 10 inputs".to_string(),
            stats.total_tx_with_more_than_10_inputs,
        )],
        ..Default::default()
    }
}
