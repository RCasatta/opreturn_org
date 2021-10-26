mod blockchain_and_filter_size;
mod number_of_inputs_and_outputs;
mod op_return_per_month;
mod op_return_protocols;
mod op_return_sizes;
mod rounded_amount;
mod script_types;
mod segwit_multisig;
mod sighash_types;
mod spent_same_block;
mod total_tx_outputs_inputs;
mod witness_stats;

use crate::charts::Chart;
use crate::pages;
use crate::process::{Bip158Stats, OpReturnData, ScriptType, Stats, TxStats};
use crate::templates::page;
use maud::html;
use maud::Markup;
use std::collections::{BTreeMap, HashMap};

pub use blockchain_and_filter_size::blockchain_and_filter_size;
pub use number_of_inputs_and_outputs::number_of_inputs_and_outputs;
pub use op_return_per_month::op_return_per_month;
pub use op_return_protocols::op_return_protocols;
pub use op_return_sizes::op_return_sizes;
pub use rounded_amount::rounded_amount;
pub use script_types::script_types;
pub use segwit_multisig::segwit_multisig;
pub use sighash_types::sighash_types;
pub use spent_same_block::spent_same_block;
pub use total_tx_outputs_inputs::total_tx_outputs_inputs;
pub use witness_stats::witness_stats;

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
                br { }
                br { }
            }
        };
        page(charts)
    }
}

pub fn create_index(pages: &[Page]) -> Markup {
    let links = html! {
        ul {
            @for page in pages {
                li  {
                    p {
                        a href=(page.permalink) { (page.title) }
                    }
                }
            }
        }
    };
    page(links)
}

pub fn create_contact() -> Markup {
    let content = html! {
        h2 { "Contact" }
        form action="https://formspree.io/f/xnqlrbey" method="POST" {
            label {
                p { "Your email:"}
                input type="email" name="_replyto" { }
            }
            br {}
            label {
                p { "Your message:"}
                textarea name="message" rows="4" cols="50" { }
            }
            input type="hidden" name="_tags" value="opreturn.org" { }
            br {}
            button type="submit" { "Send" }
            br {}
        }
    };

    page(content)
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
    tx_stats: &TxStats,
) -> Vec<Page> {
    let mut pages = vec![];

    pages.push(pages::blockchain_and_filter_size(&stats, &bip158));
    pages.push(pages::witness_stats(&stats));
    pages.push(pages::number_of_inputs_and_outputs(&tx_stats));
    pages.push(pages::op_return_per_month(&opret));
    pages.push(pages::op_return_protocols(&opret));
    pages.push(pages::op_return_sizes(&opret));
    pages.push(pages::script_types(&script_type));
    pages.push(pages::rounded_amount(&tx_stats));
    pages.push(pages::segwit_multisig(&script_type));
    pages.push(pages::spent_same_block(&stats, &tx_stats));
    pages.push(pages::sighash_types(&stats));
    pages.push(pages::total_tx_outputs_inputs(&tx_stats));

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
    use crate::process_stats::Stats;
    use crate::r#mod::{create_filter_and_blockchain_size, create_witness_stats, get_pages};
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
