use crate::charts::{Chart, Color, Dataset};
use crate::counter::perc_1000;
use crate::pages::Page;
use crate::process::TxStats;

pub fn total_tx_outputs_inputs(tx_stats: &TxStats) -> Page {
    let title = "Tx, inputs and outputs";
    let (tx, _) = tx_stats.total_tx_per_period.finish();
    let (inputs, _) = tx_stats.total_inputs_per_period.finish();
    let (outputs, mul) = tx_stats.total_outputs_per_period.finish();

    let chart = Chart::periods(title, outputs.len(), mul)
        .with(Dataset::line("Total tx", tx.clone(), Color::Blue))
        .with(Dataset::line("Total inputs", inputs.clone(), Color::Orange))
        .with(Dataset::line("Total outputs", outputs.clone(), Color::Red))
        .with_right_axis(
            Dataset::line(
                "Average outputs *1000",
                perc_1000(&outputs, &tx),
                Color::Purple,
            )
            .dashed(),
        )
        .with_right_axis(
            Dataset::line(
                "Average inputs *1000",
                perc_1000(&inputs, &tx),
                Color::Green,
            )
            .dashed(),
        );

    Page {
        title: title.to_string(),
        description: "Charts showing total number of transactions, inputs and outputs per month"
            .to_string(),
        permalink: "total-tx-outputs-inputs".to_string(),
        charts: vec![chart],
        text: format!(
            "total tx: {}, total inputs: {}, total outputs: {}, total_spendable_outputs: {}",
            tx_stats.total_tx,
            tx_stats.total_inputs,
            tx_stats.total_outputs,
            tx_stats.total_spendable_outputs
        ),
        ..Default::default()
    }
}
