use crate::charts::{Chart, Color, Dataset, Kind};
use crate::process::{OpReturnData, ScriptType};
use crate::process_bip158::Bip158Stats;
use crate::process_stats::Stats;
use crate::templates::page;
use maud::html;
use maud::Markup;
use std::collections::{BTreeMap, HashMap};

pub struct Page {
    pub title: String,
    pub description: String,
    pub permalink: String,
    pub charts: Vec<Chart>,
}

impl Page {
    pub fn to_html(&self) -> Markup {
        let charts = html! {
            @for chart in self.charts.iter() {
                (chart.to_html())
            }
        };
        page(charts)
    }
    pub fn permalink(&self) -> String {
        format!("{}/index.html", self.permalink)
    }
}

pub fn create_index(pages: &[Page]) -> Markup {
    let links = html! {
        ul {
            @for page in pages {
                li  {
                    a href=(page.permalink()) { (page.title) }
                }
            }
        }
    };
    page(links)
}

pub fn create_number_of_inputs_and_outputs(stats: &Stats) -> Page {
    let mut charts = vec![];
    let map = map_by_value(&stats.in_out);

    let in_out_labels: Vec<_> = map.keys().cloned().collect();

    let mut chart = Chart::new(
        "Number of inputs and outputs".to_string(),
        Kind::Pie,
        in_out_labels,
    );

    let dataset = Dataset {
        label: "Number of inputs and outputs".to_string(),
        data: map.values().cloned().collect(),
        background_color: Color::rainbow(),
        border_color: vec![],
        fill: true,
        hidden: false,
    };
    chart.add_dataset(dataset);

    charts.push(chart);

    Page {
        title: "Number of inputs and outputs".to_string(),
        description: "Show how many txs have A input and B output".to_string(),
        permalink: "number-of-inputs-and-outputs".to_string(),
        charts,
    }
}

pub fn create_op_return_size(opret: &OpReturnData) -> Page {
    let mut charts = vec![];
    let map = &opret.op_ret_size;

    let size_labels: Vec<_> = map.keys().cloned().collect();

    let mut chart = Chart::new("OP_RETURN sizes".to_string(), Kind::Bar, size_labels);

    let dataset = Dataset {
        label: "size [kbytes]".to_string(),
        data: map.values().map(|e| *e >> 10).collect(),
        background_color: vec![Color::Red],
        border_color: vec![],
        fill: true,
        hidden: false,
    };
    chart.add_dataset(dataset);

    charts.push(chart);

    Page {
        title: "OP_RETURN sizes".to_string(),
        description:
            "Chart showing the distribution of the sizes of the OP_RETURN scripts since inception."
                .to_string(),
        permalink: "op-return-sizes".to_string(),
        charts,
    }
}

pub fn create_filter_and_blockchain_size(
    blockchain: &BTreeMap<String, u64>,
    filters: &BTreeMap<String, u64>,
) -> Page {
    let mut charts = vec![];

    let blockchain_labels: Vec<_> = blockchain.keys().cloned().collect();
    let filters_labels: Vec<_> = filters.keys().cloned().collect();
    assert_eq!(blockchain_labels, filters_labels);
    let mut chart = Chart::new(
        "Blockchain and BIP158 filter size [MB]".to_string(),
        Kind::Line,
        blockchain_labels,
    );

    let dataset = Dataset {
        label: "Blockchain size".to_string(),
        data: blockchain.values().map(|e| *e >> 20).collect(),
        background_color: vec![Color::Orange],
        border_color: vec![Color::Orange],
        fill: false,
        hidden: false,
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "BIP158 filter size".to_string(),
        data: filters.values().map(|e| *e >> 20).collect(),
        background_color: vec![Color::Red],
        border_color: vec![Color::Red],
        fill: false,
        hidden: false,
    };
    chart.add_dataset(dataset);

    charts.push(chart);

    Page {
        title: "Blockhain and BIP158 filter size [MB]".to_string(),
        description: "Charts showing the Blockchain size and BIP158 filter size".to_string(),
        permalink: "blockchain_and_filter_size".to_string(),
        charts,
    }
}

pub fn create_script_type(script_type: &ScriptType) -> Page {
    let mut charts = vec![];

    let labels: Vec<_> = to_label_map(&script_type.all).keys().cloned().collect();

    let mut chart = Chart::new("Script types [-]".to_string(), Kind::Line, labels);

    let dataset = Dataset {
        label: "All".to_string(),
        data: script_type.all.clone(),
        background_color: vec![Color::Blue],
        border_color: vec![Color::Blue],
        fill: false,
        hidden: true,
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "p2pkh".to_string(),
        data: script_type.p2pkh.clone(),
        background_color: vec![Color::Green],
        border_color: vec![Color::Green],
        fill: false,
        hidden: false,
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "p2pk".to_string(),
        data: script_type.p2pk.clone(),
        background_color: vec![Color::Red],
        border_color: vec![Color::Red],
        fill: false,
        hidden: false,
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "v0_p2wpkh".to_string(),
        data: script_type.v0_p2wpkh.clone(),
        background_color: vec![Color::Yellow],
        border_color: vec![Color::Yellow],
        fill: false,
        hidden: false,
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "v0_p2wsh".to_string(),
        data: script_type.v0_p2wsh.clone(),
        background_color: vec![Color::Orange],
        border_color: vec![Color::Orange],
        fill: false,
        hidden: false,
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "p2sh".to_string(),
        data: script_type.p2sh.clone(),
        background_color: vec![Color::Purple],
        border_color: vec![Color::Purple],
        fill: false,
        hidden: false,
    };
    chart.add_dataset(dataset);

    let dataset = Dataset {
        label: "Other".to_string(),
        data: script_type.other.clone(),
        background_color: vec![Color::Grey],
        border_color: vec![Color::Grey],
        fill: false,
        hidden: false,
    };
    chart.add_dataset(dataset);

    charts.push(chart);

    Page {
        title: "Script types".to_string(),
        description: "Charts showing the script types per month.".to_string(),
        permalink: "script-types".to_string(),
        charts,
    }
}

pub fn create_op_return_per_month(opret: &OpReturnData) -> Page {
    let op_ret_per_month = to_label_map(&opret.op_ret_per_month);

    let mut charts = vec![];

    let op_ret_per_month_labels: Vec<_> = op_ret_per_month.keys().cloned().collect();
    let mut chart = Chart::new(
        "OP_RETURN per month".to_string(),
        Kind::Line,
        op_ret_per_month_labels,
    );
    let dataset = Dataset {
        label: "OP_RETURN [-]".to_string(),
        data: op_ret_per_month.values().cloned().collect(),
        background_color: vec![Color::Orange],
        border_color: vec![Color::Orange],
        fill: false,
        hidden: false,
    };
    chart.add_dataset(dataset);
    charts.push(chart);
    drop(op_ret_per_month);

    let op_ret_fee_per_month = to_label_map(&opret.op_ret_fee_per_month);
    let op_ret_per_month_labels: Vec<_> = op_ret_fee_per_month.keys().cloned().collect();
    let mut chart = Chart::new(
        "fees of OP_RETURN tx [bitcoin]".to_string(),
        Kind::Line,
        op_ret_per_month_labels,
    );
    let dataset = Dataset {
        label: "OP_RETURN fee [bitcoin]".to_string(),
        data: op_ret_fee_per_month.values().cloned().collect(),
        background_color: vec![Color::Orange],
        border_color: vec![Color::Orange],
        fill: false,
        hidden: false,
    };
    chart.add_dataset(dataset);
    charts.push(chart);
    drop(op_ret_fee_per_month);

    Page {
        title: "OP_RETURN per month".to_string(),
        description: "Charts showing the number of OP_RETURN scripts and fee spent per month."
            .to_string(),
        permalink: "op-return-per-month".to_string(),
        charts,
    }
}

pub fn create_witness_stats(stats: &Stats) -> Page {
    let mut chart1 = Chart::new(
        "Inputs with or without elements in witness".to_string(),
        Kind::Pie,
        stats.has_witness.keys().cloned().collect(),
    );
    let dataset = Dataset {
        label: "chart1".to_string(),
        data: stats.has_witness.values().cloned().collect(),
        background_color: Color::rainbow(),
        border_color: vec![],
        fill: true,
        hidden: false,
    };
    chart1.add_dataset(dataset);

    let mut chart2 = Chart::new(
        "Number of elements in non-empty witness".to_string(),
        Kind::Pie,
        stats.witness_elements.keys().cloned().collect(),
    );
    let dataset = Dataset {
        label: "chart2".to_string(),
        data: stats.witness_elements.values().cloned().collect(),
        background_color: Color::rainbow(),
        border_color: vec![],
        fill: true,
        hidden: false,
    };
    chart2.add_dataset(dataset);

    let mut chart3 = Chart::new(
        "Bytes in non-empty witness".to_string(),
        Kind::Pie,
        stats.witness_byte_size.keys().cloned().collect(),
    );
    let dataset = Dataset {
        label: "chart3".to_string(),
        data: stats.witness_byte_size.values().cloned().collect(),
        background_color: Color::rainbow(),
        border_color: vec![],
        fill: true,
        hidden: false,
    };
    chart3.add_dataset(dataset);

    Page {
        title: "Witness stats".to_string(),
        description: "Stats about the witnesses, number of elements and bytes used".to_string(),
        permalink: "witness-stats".to_string(),
        charts: vec![chart1, chart2, chart3],
    }
}

pub fn create_op_return_protocols(opret: &OpReturnData) -> Page {
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
        hidden: false,
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
        hidden: false,
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
        hidden: false,
    };
    chart3.add_dataset(dataset);

    Page {
        title: "OP_RETURN protocols".to_string(),
        description: "Protocols just mean the first 3 bytes of the OP_RETURN data, which can indicate the protocol but it's not an enfoced rule by the Bitcoin consensus.".to_string(),
        permalink: "op-return-protocols".to_string(),
        charts: vec![chart1, chart2, chart3],
    }
}

pub fn cumulative(values: &[u64]) -> Vec<u64> {
    let mut result = Vec::with_capacity(values.len());
    let mut cum = 0;
    for val in values {
        cum += val;
        result.push(cum);
    }
    result
}

fn to_label_map(values: &[u64]) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    for (i, value) in values.iter().enumerate() {
        map.insert(index_month(i), *value);
    }
    map
}

pub fn index_month(index: usize) -> String {
    let year = 2009 + index / 12;
    let month = (index % 12) + 1;
    format!("{:04}{:02}", year, month)
}

pub fn get_pages(
    bip158: &Bip158Stats,
    opret: &OpReturnData,
    script_type: &ScriptType,
    stats: &Stats,
) -> Vec<Page> {
    let mut pages = vec![];

    let blockchain = to_label_map(&cumulative(&stats.block_size_per_month));
    let filters = to_label_map(&cumulative(&bip158.bip158_filter_size_per_month));

    pages.push(create_filter_and_blockchain_size(&blockchain, &filters));
    pages.push(create_witness_stats(&stats));
    pages.push(create_number_of_inputs_and_outputs(&stats));
    pages.push(create_op_return_per_month(&opret));
    pages.push(create_op_return_protocols(&opret));
    pages.push(create_op_return_size(&opret));
    pages.push(create_script_type(&script_type));

    pages
}

pub fn map_by_value(map: &HashMap<String, u64>) -> BTreeMap<String, u64> {
    let mut tree: BTreeMap<String, u64> = BTreeMap::new();
    let mut count_vec: Vec<(&String, &u64)> = map.iter().collect();
    count_vec.sort_by(|a, b| b.1.cmp(a.1));
    for (key, value) in count_vec.iter().take(10) {
        tree.insert(key.to_string(), **value);
    }
    let other = count_vec.iter().skip(10).fold(0, |acc, x| acc + x.1);
    if other > 0 {
        tree.insert("other".to_owned(), other);
    }
    tree
}

#[cfg(test)]
mod test {
    use crate::charts::test::{mock_lines_chart, mock_pie_chart};
    use crate::charts::{Chart, Kind};
    use crate::pages::{create_filter_and_blockchain_size, create_witness_stats, get_pages};
    use crate::process_stats::Stats;
    use crate::templates::page;
    use std::collections::BTreeMap;

    #[test]
    fn test_pie_page() {
        let chart = mock_pie_chart();
        let page = page(chart.to_html()).into_string();
        assert_eq!("", to_data_url(page, "text/html"));
    }

    #[test]
    fn test_lines_page() {
        let chart = mock_lines_chart();
        let page = page(chart.to_html()).into_string();
        assert_eq!("", to_data_url(page, "text/html"));
    }

    #[test]
    fn test_create_bip_158_filter_size() {
        let mut map = BTreeMap::new();
        map.insert("200901".to_string(), 10);
        map.insert("200902".to_string(), 15);
        map.insert("200904".to_string(), 25);
        map.insert("200905".to_string(), 45);

        let page = create_filter_and_blockchain_size(&map);
        assert_eq!("", to_data_url(page.to_html().into_string(), "text/html"));
    }

    #[test]
    fn test_witness_stats() {
        let mut stats = Stats::new();
        stats.has_witness.insert("200901".to_string(), 10);
        stats.has_witness.insert("200902".to_string(), 15);
        stats.witness_elements.insert("200901".to_string(), 10);
        stats.witness_elements.insert("200902".to_string(), 15);
        stats.witness_byte_size.insert("200901".to_string(), 10);
        stats.witness_byte_size.insert("200902".to_string(), 15);

        let page = create_witness_stats(&stats);
        assert_eq!("", to_data_url(page.to_html().into_string(), "text/html"));
    }

    fn to_data_url<T: AsRef<[u8]>>(input: T, content_type: &str) -> String {
        let base64 = base64::encode(input.as_ref());
        format!("data:{};base64,{}", content_type, base64)
    }
}
