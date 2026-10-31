use crate::charts::Chart;
use crate::pages::Page;
use crate::process::Stats;

pub fn sighash_types(stats: &Stats) -> Page {
    Page {
        title: "Number of sighash type used".to_string(),
        description: "Show distribution of sighash type used".to_string(),
        permalink: "sighash-types".to_string(),
        charts: vec![Chart::pie(
            "Number of sighash type used",
            &stats.sighashtype,
        )],
        ..Default::default()
    }
}
