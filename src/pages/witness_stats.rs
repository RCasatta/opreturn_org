use crate::charts::Chart;
use crate::pages::Page;
use crate::process::Stats;

pub fn witness_stats(stats: &Stats) -> Page {
    Page {
        title: "Witness stats".to_string(),
        description: "Stats about the witnesses, number of elements and bytes used".to_string(),
        permalink: "witness-stats".to_string(),
        charts: vec![
            Chart::pie(
                "Inputs with or without elements in witness",
                &stats.has_witness,
            ),
            Chart::pie(
                "Number of elements in non-empty witness",
                &stats.witness_elements,
            ),
            Chart::pie("Bytes in non-empty witness", &stats.witness_byte_size),
        ],
        ..Default::default()
    }
}
