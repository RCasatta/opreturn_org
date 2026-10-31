use crate::charts::{Chart, Color, Dataset};
use crate::pages::Page;
use crate::process::TxStats;

pub fn rounded_amount(tx_stats: &TxStats) -> Page {
    let (data, mul) = tx_stats.rounded_amount_per_period.finish();
    let chart = Chart::periods("Rounded amount [-]", data.len(), mul).with(Dataset::line(
        "rounded amounts",
        data,
        Color::Blue,
    ));

    Page {
        title: "Rounded amount".to_string(),
        description: "Charts showing the number of outputs which have a rounded amount as value (multiple of 1000)".to_string(),
        permalink: "rounded-amount".to_string(),
        charts: vec![chart],
        ..Default::default()
    }
}
