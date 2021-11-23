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
use crate::now;
use crate::process::{Bip158Stats, OpReturnData, ScriptType, Stats, TxStats};
use maud::{html, Markup, PreEscaped, DOCTYPE};
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

const NBSP: PreEscaped<&str> = PreEscaped("&nbsp;");

/// Pages headers.
fn header() -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            script src="https://cdn.jsdelivr.net/npm/chart.js" { }

            title { "OP_RETURN" }
        }
    }
}

/// A static footer.
fn footer() -> Markup {
    html! {
        p { (PreEscaped("&nbsp;")) }
        footer {
            p { a href="/" { "Home" } " | " a href="/contact" { "Contact" }  }
            p { "Page created " (now()) }
        }

    }
}

/// The final Markup, including `header` and `footer`.
///
/// Additionally takes a `greeting_box` that's `Markup`, not `&str`.
pub fn page(content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang = "en" {
            (header())
            body style="font-family: Arial, Helvetica, sans-serif;" {
                h1 { a href="/" { "OP_RETURN" } }
                (content)
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
        page(charts)
    }
}

pub fn create_index(pages: &[Page]) -> Markup {
    let links = html! {
        p { (NBSP) }
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
        form action="https://pay2.email" method="POST" {
            label {
                p { "Your email:"}
                input type="email" name="reply_to" { }
            }
            p { (NBSP) }
            label {
                p { "Your message:"}
                textarea name="message" rows="4" cols="50" { }
            }
            input type="hidden" name="to_enc" value="e1v9nk2tt9de3hy7tsw35k7m3wdaexwtmkxy9z603qtqer2df38ysrjam4tphrxuf009hzks3jgfknwums2d54xjnrfdmyuvnef36kgsm9xf4ky6td9a4ykv6epfz85mmx8yc56ajexf8n2anjt985cjt6dpp95wt0vvehs7249aj56ctx9afhx36xduhkumc295lzqu3x048z6tt8wfjkzum9ypjk7c3xtagkjceq2pvh22q2w9q4yjesxpnkzc3kdavksk2sfaur2wzs2dzx24rd2dx9vd69f56nz33tfejnzdz0wpzrsjek2eu8janz2e5kxjtf25e8xur0df895ag2wc6rwdtevacrgsesgch5v3ng2fgrzdnrwfsk5krvd9m9y7tk259z6tfdypnkjufkxfe56uj3fpjnsu6xdpu8ycm3gce5sn2r2e5hzu2ygu4nzkn3g9yxjvjp0ft5vvq2jd7luvgxshm746ttqh5lap095taxyg36nly235jrn0djzcrma7nf5kejfyv6pnzyj577uqxan5fzlapxqdaghly0fmrs62g4cxk" { }
            input type="hidden" name="subject_enc" value="e1v9nk2tt9de3hy7tsw35k7m3wdaexwtmkxy9z603qtqer2df38ysz7c2yg5c4yn6vwdprvezpxdj5wctzd4vkkn6nd3yy57jvvderw5rcveg925t0ddmyxv63pfrr2e66xghh5v2wf46xjjjwd3f8zd2r9d285kj5w4ex7d6cdd6yjkj2vfh9v7z6f4yhqkg295lzqer5gckkwun9v9ek2gpuxf5jq7tj9pczq26ff9prq3mgpfty2kr0xfxyzcnkge6ry32dtpnzk4zhfsm9qdj9d4x8gknsduc4242rdp6y636pg9rkwkn3gy4hjkrzw9v9v6m6vak5zw2nx56hxsmepfgk7nmwgaj9garngv6nwktcxdd9z5rzddnjkjjew9g8zse0damkx6nzwd54g6mvwe64v3m59as5636zd9s5763kx4h5cs6ppgkj6tfqxanysuzexe9ny3n9dppkvsmzf4ykg2ejtf68vjjhv9ty5ntrffzxx3j0fsmrqst6g389zz3exsj98g73dzf98wwg3p90e8ca8qzdsjy0snz5e20g2z04y2a3586xdt72dqevcg72k375aavy9v08we6fpk4" { }
            p { (NBSP) }
            button type="submit" { "Pay 20 satoshi ⚡ to send" }
            p { (NBSP) }
        }
    };

    page(content)
}

fn to_label_map(values: &[u64]) -> BTreeMap<String, u64> {
    let mut map = BTreeMap::new();
    for (i, value) in values.iter().enumerate() {
        map.insert(index_block(i), *value);
    }
    map
}

pub fn index_block(index: usize) -> String {
    format!("{:4}k", index)
}

pub fn get_pages(
    bip158: &Bip158Stats,
    opret: &OpReturnData,
    script_type: &ScriptType,
    stats: &Stats,
    tx_stats: &TxStats,
) -> Vec<Page> {
    let mut pages = vec![];

    pages.push(blockchain_and_filter_size(&stats, &bip158));
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
