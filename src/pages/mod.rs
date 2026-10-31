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
mod segwit_script_templates;
mod sighash_types;
mod spent_same_block;
mod total_tx_outputs_inputs;
mod witness_stats;

use crate::charts::Chart;
use crate::now;
use crate::process::{Bip158Stats, OpReturnData, ScriptType, Stats, TxStats};
use maud::{html, Markup, PreEscaped, DOCTYPE};
use std::path::Path;
use std::{fs, io};

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
pub use segwit_script_templates::segwit_script_templates;
pub use sighash_types::sighash_types;
pub use spent_same_block::spent_same_block;
pub use total_tx_outputs_inputs::total_tx_outputs_inputs;
pub use witness_stats::witness_stats;

#[derive(Default)]
pub struct Page {
    pub title: String,
    pub description: String,
    pub permalink: String,
    pub charts: Vec<Chart>,
    /// Totals shown in a table after the charts, as (label, value)
    pub stats: Vec<(String, u64)>,
    /// Html shown before the charts
    pub content: Option<Markup>,
    /// Additional files written in the page directory, as (file name, content)
    pub files: Vec<(String, String)>,
}

/// Inline stylesheet, colors are css variables redefined for dark mode and read by the charts
const STYLE: &str = r#"
:root { --bg: #fff; --fg: #1f2328; --muted: #59636e; --border: #d1d9e0; --stripe: #f6f8fa;
        --accent: #b4410c; --grid: rgba(0,0,0,.08); }
@media (prefers-color-scheme: dark) {
  :root { --bg: #0d1117; --fg: #e6edf3; --muted: #9198a1; --border: #3d444d; --stripe: #151b23;
          --accent: #f0883e; --grid: rgba(255,255,255,.1); }
}
* { box-sizing: border-box; }
html { font: 16px/1.5 system-ui, -apple-system, "Segoe UI", Roboto, sans-serif; }
body { margin: 0; background: var(--bg); color: var(--fg); }
.wrap { max-width: 1000px; margin: 0 auto; padding: 0 16px; }
a { color: var(--accent); }
code, textarea { font: .85em ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; }
header { border-bottom: 1px solid var(--border); margin-bottom: 24px; }
header .wrap { display: flex; flex-wrap: wrap; align-items: baseline; justify-content: space-between;
               gap: 8px 16px; padding-top: 12px; padding-bottom: 12px; }
.brand { font: 700 1.3rem ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
         color: var(--fg); text-decoration: none; }
nav a { margin-left: 16px; }
h1 { font-size: 1.6rem; line-height: 1.25; margin: 0 0 4px; }
h2 { font-size: 1.2rem; margin-top: 32px; }
.description { color: var(--muted); margin-top: 0; }
.chart { position: relative; height: clamp(280px, 60vh, 480px); margin: 24px 0 40px; }
.table-wrap { overflow-x: auto; }
table { border-collapse: collapse; width: 100%; font-size: .9rem; }
th, td { padding: 4px 8px; border-bottom: 1px solid var(--border); text-align: left;
         vertical-align: top; }
tbody tr:nth-child(even) { background: var(--stripe); }
.num { text-align: right; white-space: nowrap; }
.stats { width: auto; min-width: 280px; }
.nowrap { white-space: nowrap; }
textarea { width: 100%; padding: 8px; color: var(--fg); background: var(--bg);
           border: 1px solid var(--border); border-radius: 6px; }
.pages { list-style: none; padding: 0; }
.pages li { padding: 12px 0; border-bottom: 1px solid var(--border); }
.pages a { font-weight: 600; }
.pages span { display: block; color: var(--muted); }
footer { border-top: 1px solid var(--border); margin-top: 48px; padding: 16px 0;
         color: var(--muted); font-size: .875rem; }
"#;

/// Chart.js defaults: follow the page colors, keep the height given by the `.chart` container and
/// put pie legends on the side when there is room
const CHART_DEFAULTS_JS: &str = r#"
(function () {
  var style = getComputedStyle(document.documentElement);
  Chart.defaults.color = style.getPropertyValue('--muted').trim();
  Chart.defaults.borderColor = style.getPropertyValue('--grid').trim();
  Chart.defaults.elements.arc.borderColor = style.getPropertyValue('--bg').trim();
  Chart.defaults.font.family = style.fontFamily;
  Chart.defaults.maintainAspectRatio = false;
  // lines without points, hovering shows all the series at that x
  Chart.defaults.elements.point.radius = 0;
  Chart.defaults.interaction.mode = 'index';
  Chart.defaults.interaction.intersect = false;
  // but in a pie the index mode picks the slice by horizontal distance, highlight the one under
  // the mouse instead
  Chart.overrides.pie.interaction = { mode: 'nearest', intersect: true };
  Chart.overrides.pie.plugins.legend.position =
    window.matchMedia('(min-width: 700px)').matches ? 'right' : 'bottom';
})();
"#;

/// Pages headers.
fn header(title: &str, description: &str) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1.0";
            meta name="description" content=(description);
            title { (title) }
            style { (PreEscaped(STYLE)) }

            script src="https://cdn.jsdelivr.net/npm/chart.js@4" { }
            script { (PreEscaped(CHART_DEFAULTS_JS)) }
            script defer data-domain="opreturn.org" src="https://plausible.casatta.it/js/script.js" { }
        }
    }
}

/// A static footer.
fn footer() -> Markup {
    html! {
        footer {
            div class="wrap" {
                "Page created " (now()) " · "
                a href="https://github.com/RCasatta/opreturn_org" { "source" }
            }
        }
    }
}

/// The final Markup, including `header` and `footer`.
///
/// `title` is the html title, `description` its meta description.
pub fn page(title: &str, description: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang = "en" {
            (header(title, description))
            body {
                header {
                    div class="wrap" {
                        a class="brand" href="/" { "OP_RETURN" }
                        nav { a href="/" { "Charts" } a href="/about" { "About" } }
                    }
                }
                main class="wrap" {
                    (content)
                }
                (footer())
            }
        }
    }
}

impl Page {
    pub fn to_html(&self) -> Markup {
        let content = html! {
            h1 { (self.title) }
            p class="description" { (self.description) }
            @if let Some(content) = &self.content {
                (content)
            }
            @for (i, chart) in self.charts.iter().enumerate() {
                (chart.to_html(&format!("chart{}", i)))
            }
            @if !self.stats.is_empty() {
                table class="stats" {
                    tbody {
                        @for (label, value) in self.stats.iter() {
                            tr { th { (label) } td class="num" { (thousands(*value)) } }
                        }
                    }
                }
            }
        };
        page(
            &format!("{} · OP_RETURN", self.title),
            &self.description,
            content,
        )
    }
}

fn create_index(pages: &[Page]) -> Markup {
    let description = "Charts and statistics about the Bitcoin blockchain";
    let links = html! {
        h1 { "Charts" }
        p class="description" { (description) ", updated daily." }
        ul class="pages" {
            @for page in pages {
                li {
                    a href=(page.permalink) { (page.title) }
                    span { (page.description) }
                }
            }
        }
    };
    page("OP_RETURN", description, links)
}

fn create_about() -> Markup {
    let content = html! {
        h1 { "About" }
        p { "OP_RETURN shows charts about the Bitcoin blockchain, it's built once a day." }
        p {
            "Blocks are read with "
            a href="https://github.com/RCasatta/blocks_iterator" { "blocks_iterator" }
            ", the "
            a href="https://github.com/RCasatta/opreturn_org" { "source" }
            " of the site is on GitHub."
        }
    };
    page("About · OP_RETURN", "About the OP_RETURN site", content)
}

/// `n` with comma separated thousands, as `toLocaleString('en-US')` does in javascript
pub fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut result = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result
}

/// Writes the pages, each in its own directory, the index, the about page and the favicon in
/// `site_home`
pub fn write_site(site_home: &Path, pages: &[Page]) -> io::Result<()> {
    let write_page = |dir: &Path, html: Markup| -> io::Result<()> {
        fs::create_dir_all(dir)?;
        fs::write(dir.join("index.html"), html.into_string())
    };
    for page in pages {
        let page_dir = site_home.join(&page.permalink);
        write_page(&page_dir, page.to_html())?;
        for (name, content) in page.files.iter() {
            fs::write(page_dir.join(name), content)?;
        }
    }
    write_page(site_home, create_index(pages))?;
    write_page(&site_home.join("about"), create_about())?;
    fs::write(site_home.join("favicon.ico"), include_bytes!("favicon.ico"))
}

pub fn get_pages(
    bip158: &Bip158Stats,
    opret: &OpReturnData,
    script_type: &ScriptType,
    stats: &Stats,
    tx_stats: &TxStats,
) -> Vec<Page> {
    vec![
        blockchain_sizes(stats, bip158, tx_stats),
        witness_stats(stats),
        number_of_inputs_and_outputs(tx_stats),
        op_return_per_month(opret),
        op_return_protocols(opret),
        op_return_sizes(opret),
        script_types(script_type),
        rounded_amount(tx_stats),
        segwit_multisig(script_type),
        segwit_script_templates(script_type),
        spent_same_block(stats, tx_stats),
        sighash_types(stats),
        total_tx_outputs_inputs(tx_stats),
        bip69(tx_stats),
        locktime(tx_stats),
    ]
}

#[cfg(test)]
mod tests {
    use super::thousands;

    #[test]
    fn test_thousands() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(32714282), "32,714,282");
    }
}
