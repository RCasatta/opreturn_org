use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::{map_by_value, Page};
use crate::process::OpReturnData;

pub fn op_return_protocols(opret: &OpReturnData) -> Page {
    let map1 = map_by_value(&opret.op_ret_per_proto_last_month);
    let mut chart1 = Chart::new(
        "Last month".to_string(),
        Kind::Pie,
        map1.keys().cloned().collect(),
    );
    let dataset = Dataset {
        label: "chart1".to_string(),
        data: map1.values().cloned().collect(),
        background_color: Color::rainbow(),
        border_color: vec![],
        fill: true,
        ..Default::default()
    };
    chart1.add_dataset(dataset);
    drop(map1);

    let map2 = map_by_value(&opret.op_ret_per_proto_last_year);
    let mut chart2 = Chart::new(
        "Last year".to_string(),
        Kind::Pie,
        map2.keys().cloned().collect(),
    );
    let dataset = Dataset {
        label: "chart2".to_string(),
        data: map2.values().cloned().collect(),
        background_color: Color::rainbow(),
        border_color: vec![],
        fill: true,
        ..Default::default()
    };
    chart2.add_dataset(dataset);

    let map3 = map_by_value(&opret.op_ret_per_proto);
    let mut chart3 = Chart::new(
        "Ever".to_string(),
        Kind::Pie,
        map3.keys().cloned().collect(),
    );
    let dataset = Dataset {
        label: "chart3".to_string(),
        data: map3.values().cloned().collect(),
        background_color: Color::rainbow(),
        border_color: vec![],
        fill: true,
        ..Default::default()
    };
    chart3.add_dataset(dataset);

    Page {
        title: "OP_RETURN protocols".to_string(),
        description: "Protocols just mean the first 3 bytes of the OP_RETURN data, which can indicate the protocol but it's not an enfoced rule by the Bitcoin consensus.".to_string(),
        permalink: "op-return-protocols".to_string(),
        charts: vec![chart1, chart2, chart3],
    }
}
