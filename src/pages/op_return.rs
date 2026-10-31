use crate::charts::{Chart, Color, Dataset};
use crate::counter::Counter;
use crate::pages::Page;
use crate::process::OpReturnData;

/// A chart with a single line of the values in `counter`, converted by `map`
fn line(title: &str, label: &str, counter: &Counter, color: Color, map: fn(u64) -> u64) -> Chart {
    let (data, mul) = counter.finish();
    Chart::periods(title, data.len(), mul).with(Dataset::line(
        label,
        data.into_iter().map(map).collect(),
        color,
    ))
}

pub fn op_return_per_month(opret: &OpReturnData) -> Page {
    let title = "OP_RETURN";
    Page {
        title: title.to_string(),
        description: "Charts showing the number of OP_RETURN scripts and fee spent per month."
            .to_string(),
        permalink: "op-return".to_string(),
        charts: vec![
            line(
                title,
                "OP_RETURN [-]",
                &opret.op_ret_per_period,
                Color::Orange,
                |n| n,
            ),
            line(
                "OP_RETURN > 83 bytes",
                "OP_RETURN > 83 bytes [-]",
                &opret.op_ret_bigger_than_83_per_period,
                Color::Blue,
                |n| n,
            ),
            // in bitcoin most periods would be truncated to 0
            line(
                "fees of OP_RETURN tx [mBTC]",
                "OP_RETURN fee [mBTC]",
                &opret.op_ret_fee_per_period,
                Color::Orange,
                |sat| sat / 100_000,
            ),
        ],
        ..Default::default()
    }
}
