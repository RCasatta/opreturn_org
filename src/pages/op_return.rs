use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::{to_label_map, Page};
use crate::process::OpReturnData;

pub fn op_return_per_month(opret: &OpReturnData) -> Page {
    let title = "OP_RETURN".to_string();
    let (v, div) = opret.op_ret_per_period.finish();
    let op_ret_per_period = to_label_map(&v, div);

    let mut charts = vec![];

    let op_ret_per_period_labels: Vec<_> = op_ret_per_period.keys().cloned().collect();

    let mut chart = Chart::new(title.clone(), Kind::Line, op_ret_per_period_labels);
    let dataset = Dataset {
        label: "OP_RETURN [-]".to_string(),
        data: op_ret_per_period.values().cloned().collect(),
        background_color: vec![Color::Orange],
        border_color: vec![Color::Orange],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);
    charts.push(chart);
    drop(op_ret_per_period);

    let (vec, mul) = opret.op_ret_fee_per_period.finish();
    let op_ret_fee_per_month = to_label_map(&vec, mul);
    let op_ret_per_month_labels: Vec<_> = op_ret_fee_per_month.keys().cloned().collect();
    let mut chart = Chart::new(
        "fees of OP_RETURN tx [bitcoin]".to_string(),
        Kind::Line,
        op_ret_per_month_labels,
    );
    let dataset = Dataset {
        label: "OP_RETURN fee [bitcoin]".to_string(),
        data: op_ret_fee_per_month
            .values()
            .map(|sat| sat / 100_000_000)
            .collect(),
        background_color: vec![Color::Orange],
        border_color: vec![Color::Orange],
        fill: false,
        ..Default::default()
    };
    chart.add_dataset(dataset, None);
    charts.push(chart);
    drop(op_ret_fee_per_month);

    Page {
        title,
        description: "Charts showing the number of OP_RETURN scripts and fee spent per month."
            .to_string(),
        permalink: "op-return".to_string(),
        charts,
        text: "".to_string(),
    }
}
