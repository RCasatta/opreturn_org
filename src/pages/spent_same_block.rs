use crate::charts::{Chart, Color, Dataset};
use crate::pages::Page;
use crate::process::{Stats, TxStats};

pub fn spent_same_block(stats: &Stats, tx_stats: &TxStats) -> Page {
    let title = "Spent in the same block";
    let (outputs, mul) = tx_stats.total_outputs_per_period.finish();
    let chart = Chart::periods(title, outputs.len(), mul)
        .with(Dataset {
            fill: true,
            ..Dataset::line("outputs", outputs, Color::Orange)
        })
        .with(Dataset {
            fill: true,
            ..Dataset::line(
                "spent in same block",
                stats.total_spent_in_block_per_period.finish().0,
                Color::Red,
            )
        });

    Page {
        title: title.to_string(),
        description:
            "Charts showing the number of output which are spent in the same block they are created"
                .to_string(),
        permalink: "spent-same-block".to_string(),
        charts: vec![chart],
        ..Default::default()
    }
}
