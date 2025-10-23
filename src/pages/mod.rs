pub mod bip69;
mod blockchain_sizes;
mod locktime;
mod number_of_inputs_and_outputs;
mod op_return;
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
use crate::now;
use crate::process::{Bip158Stats, OpReturnData, ScriptType, Stats, TxStats};
use maud::{html, Markup, PreEscaped, DOCTYPE};
use std::collections::{BTreeMap, HashMap};

pub use bip69::bip69;
pub use blockchain_sizes::blockchain_sizes;
pub use locktime::locktime;
pub use number_of_inputs_and_outputs::number_of_inputs_and_outputs;
pub use op_return::op_return_per_month;
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
    pub text: String,
}

const NBSP: PreEscaped<&str> = PreEscaped("&nbsp;");

/// Pages headers.
fn header() -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";

            script src="https://cdn.jsdelivr.net/npm/chart.js" { }
            script defer data-domain="opreturn.org" src="https://plausible.casatta.it/js/script.js" { }

            title { "OP_RETURN" }
        }
    }
}

/// A static footer.
fn footer() -> Markup {
    html! {
        p { (PreEscaped("&nbsp;")) }
        footer {
            p { a href="/" { "Home" } " | " a href="/about" { "About" }  }
            p { "Page created " (now()) }
        }

    }
}

/// The final Markup, including `header` and `footer`.
///
/// Additionally takes a `greeting_box` that's `Markup`, not `&str`.
pub fn page(content: Markup, text: &str) -> Markup {
    html! {
        (DOCTYPE)
        html lang = "en" {
            (header())
            body style="font-family: Arial, Helvetica, sans-serif;" {
                h1 { a href="/" { "OP_RETURN" } }
                p { (NBSP) }
                (content)
                p { (text) }

                (footer())
            }
        }
    }
}

impl Page {
    pub fn to_html(&self) -> Markup {
        let charts = html! {
            @for chart in self.charts.iter() {
                (chart.to_html())
                p { (NBSP) }
            }
        };
        page(charts, &self.text)
    }
}

pub fn create_index(pages: &[Page]) -> Markup {
    let links = html! {
        ul {
            @for page in pages {
                li  {
                    p {
                        a href=(page.permalink) { (page.title) }
                        " - "
                        (page.description)
                    }
                }
            }
        }
    };
    page(links, "")
}

pub fn create_about() -> Markup {
    let blocks_iterator = html! {
        a href="https://github.com/RCasatta/blocks_iterator" { "blocks iterator" }
    };

    let source = html! {
        a href="https://github.com/RCasatta/rustat/" { "source" }
    };

    let content = html! {
        h2 { "About" }

        p { "OP_RETURN shows charts about Bitcoin." }
        p { "The site is built once a day." }
        p { "Show me the " (source) "." }
        p { "Built with " (blocks_iterator) "." }
    };

    page(content, "")
}

fn to_label_map(values: &[u64], mul: usize) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    for (i, value) in values.iter().enumerate() {
        map.insert(index_block(i, mul), *value);
    }
    map
}

pub fn index_block(index: usize, mul: usize) -> String {
    let from = index * mul;
    let to = from + mul;
    format!("{:4}k-{}k", from, to)
}

pub fn get_pages(
    bip158: &Bip158Stats,
    opret: &OpReturnData,
    script_type: &ScriptType,
    stats: &Stats,
    tx_stats: &TxStats,
) -> Vec<Page> {
    let mut pages = vec![];

    pages.push(blockchain_sizes(&stats, &bip158, &tx_stats));
    pages.push(witness_stats(&stats));
    pages.push(number_of_inputs_and_outputs(&tx_stats));
    pages.push(op_return_per_month(&opret));
    pages.push(op_return_protocols(&opret));
    pages.push(op_return_sizes(&opret));
    pages.push(script_types(&script_type));
    pages.push(rounded_amount(&tx_stats));
    pages.push(segwit_multisig(&script_type));
    pages.push(spent_same_block(&stats, &tx_stats));
    pages.push(sighash_types(&stats));
    pages.push(total_tx_outputs_inputs(&tx_stats));
    pages.push(bip69(&tx_stats));
    pages.push(locktime(&tx_stats));

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
    use crate::pages::{page, witness_stats};
    use crate::process::Stats;

    #[ignore]
    #[test]
    fn test_pie_page() {
        let chart = mock_pie_chart();
        let page = page(chart.to_html(), "").into_string();
        assert_eq!("", to_data_url(page, "text/html"));
    }

    #[ignore]
    #[test]
    fn test_lines_page() {
        let chart = mock_lines_chart();
        let page = page(chart.to_html(), "").into_string();
        assert_eq!("", to_data_url(page, "text/html"));
    }

    #[ignore]
    #[test]
    fn test_witness_stats() {
        let mut stats = Stats::new();
        stats.has_witness.insert("200901".to_string(), 10);
        stats.has_witness.insert("200902".to_string(), 15);
        stats.witness_elements.insert("200901".to_string(), 10);
        stats.witness_elements.insert("200902".to_string(), 15);
        stats.witness_byte_size.insert("200901".to_string(), 10);
        stats.witness_byte_size.insert("200902".to_string(), 15);

        let page = witness_stats(&stats);
        assert_eq!("", to_data_url(page.to_html().into_string(), "text/html"));
    }

    fn to_data_url<T: AsRef<[u8]>>(input: T, content_type: &str) -> String {
        let base64 = base64::encode(input.as_ref());
        format!("data:{};base64,{}", content_type, base64)
    }
}
