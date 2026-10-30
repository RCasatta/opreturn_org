use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::Page;
use crate::process::segwit_templates::{is_envelope, TemplateRow, OTHER};
use crate::process::ScriptType;
use maud::{html, Markup, PreEscaped};

/// Rows shown in each table of the page, the full list is in [`CSV_NAME`]
const TABLE_ROWS: usize = 500;

const CSV_NAME: &str = "templates.csv";

/// Filters the tables on the whole [`CSV_NAME`], loaded at the first use of the filter box or when
/// the url contains `#q=<filter>`. Rows are rendered with the same columns of [`table`].
const FILTER_JS: &str = r#"
(function () {
  var MAX = 500;
  var input = document.getElementById('filter');
  var status = document.getElementById('filter-status');
  var tables = document.querySelectorAll('table.templates');
  var re = /^([^,]*),"((?:[^"]|"")*)",(\d+),(\d+),(true|false),(\d+),([0-9a-f]+)$/;
  var groups = null, loading = false;

  function load(done) {
    if (groups) return done();
    if (loading) return;
    loading = true;
    status.textContent = 'loading the full list...';
    fetch('templates.csv').then(function (r) { return r.text(); }).then(function (text) {
      groups = { false: [], true: [] };
      text.split('\n').forEach(function (line) {
        var m = re.exec(line);
        if (!m) return;
        var row = { kind: m[1], template: m[2].replace(/""/g, '"'), spends: +m[3],
                    distinct: +m[4], exact: m[5] === 'true', height: +m[6], txid: m[7] };
        row.key = (row.kind + ' ' + row.template).toLowerCase();
        var group = groups[row.template.indexOf('<envelope') >= 0];
        row.rank = group.length + 1;
        group.push(row);
      });
      done();
    }).catch(function (e) { loading = false; status.textContent = 'cannot load the full list: ' + e; });
  }

  function td(tr, child) {
    var cell = document.createElement('td');
    cell.append(child);
    tr.append(cell);
    return cell;
  }

  function render() {
    var query = input.value.trim().replace(/\s+/g, ' ').toLowerCase();
    var counts = [];
    tables.forEach(function (table) {
      var total = +table.dataset.total;
      var rows = groups[table.dataset.envelope === 'true'].filter(function (r) {
        return r.key.indexOf(query) >= 0;
      });
      counts.push(rows.length);
      var tbody = table.tBodies[0];
      tbody.replaceChildren();
      rows.slice(0, MAX).forEach(function (r) {
        var tr = document.createElement('tr');
        td(tr, String(r.rank));
        td(tr, r.kind);
        var code = document.createElement('code');
        code.textContent = r.template;
        td(tr, code);
        td(tr, String(r.spends));
        td(tr, (r.spends * 100 / total).toFixed(2));
        td(tr, (r.exact ? '' : '~') + r.distinct);
        var a = document.createElement('a');
        a.href = 'https://mempool.space/tx/' + r.txid;
        a.textContent = String(r.height);
        td(tr, a);
        tbody.append(tr);
      });
    });
    status.textContent = counts.join(' script and ') + ' envelope templates match' +
      (counts.some(function (c) { return c > MAX; }) ? ', showing the first ' + MAX : '');
  }

  // the filter is kept in the url fragment, so filtered views can be shared
  input.addEventListener('input', function () {
    var hash = input.value ? '#q=' + encodeURIComponent(input.value) : '';
    history.replaceState(null, '', location.pathname + hash);
    load(render);
  });
  var initial = new URLSearchParams(location.hash.slice(1)).get('q');
  if (initial) {
    input.value = initial;
    load(render);
  }
})();
"#;

/// The kind of spend (eg. `wsh`) and the template without it
fn split_kind(row: &TemplateRow) -> (&str, &str) {
    row.template.split_once(' ').unwrap_or(("", &row.template))
}

/// All the templates, a row per template sorted by spends
fn csv(rows: &[TemplateRow]) -> String {
    let mut csv = "kind,template,spends,distinct_scripts,distinct_exact,first_height,first_txid\n"
        .to_string();
    for row in rows {
        let (kind, template) = split_kind(row);
        csv.push_str(&format!(
            "{},\"{}\",{},{},{},{},{}\n",
            kind,
            template.replace('"', "\"\""),
            row.spends,
            row.distinct_scripts,
            row.distinct_exact,
            row.first_height,
            row.first_txid,
        ));
    }
    csv
}

/// The first [`TABLE_ROWS`] of `rows`, percentages are relative to the spends of all `rows`
fn table(rows: &[&TemplateRow], envelope: bool) -> Markup {
    let total: u64 = rows.iter().map(|r| r.spends).sum();
    html! {
        table class="templates" data-envelope=(envelope) data-total=(total) {
            thead {
                tr {
                    th { "#" }
                    th { "kind" }
                    th { "template" }
                    th { "spends" }
                    th { "% spends" }
                    th title="distinct scripts using the template, ~ when estimated" { "distinct scripts" }
                    th title="height of the first spend, links to the transaction" { "first seen" }
                }
            }
            tbody {
                @for (i, row) in rows.iter().take(TABLE_ROWS).enumerate() {
                    @let (kind, template) = split_kind(row);
                    tr {
                        td { (i + 1) }
                        td { (kind) }
                        td { code { (template) } }
                        td { (row.spends) }
                        td { (format!("{:.2}", row.spends as f64 * 100.0 / total as f64)) }
                        td { @if !row.distinct_exact { "~" } (row.distinct_scripts) }
                        td { a href=(format!("https://mempool.space/tx/{}", row.first_txid)) { (row.first_height) } }
                    }
                }
            }
        }
    }
}

/// Histogram of the templates by anonymity set size, considering only envelope or only
/// non-envelope templates
fn anonymity_set_chart(script_type: &ScriptType, envelope: bool, what: &str) -> Chart {
    let histogram = script_type
        .segwit_templates
        .anonymity_set_histogram(envelope);
    let mut chart = Chart::new(
        format!(
            "Number of {}s by count of distinct scripts using it (anonymity set)",
            what
        ),
        Kind::Bar,
        histogram.iter().map(|(l, _)| l.clone()).collect(),
    );
    let dataset = Dataset {
        label: format!("{}s", what),
        data: histogram.iter().map(|(_, c)| *c).collect(),
        background_color: vec![Color::Blue],
        border_color: vec![Color::Blue],
        ..Default::default()
    };
    chart.add_dataset(dataset, None);
    chart
}

pub fn segwit_script_templates(script_type: &ScriptType) -> Page {
    let templates = &script_type.segwit_templates;
    let rows: Vec<_> = templates
        .rows()
        .into_iter()
        .filter(|r| r.template != OTHER)
        .collect();
    // envelopes (mostly inscriptions) dominate taproot script spends, they are shown separately
    let (envelopes, scripts): (Vec<_>, Vec<_>) =
        rows.iter().partition(|r| is_envelope(&r.template));
    let envelope_spends: u64 = envelopes.iter().map(|r| r.spends).sum();

    let content = html! {
        p {
            (rows.len()) " distinct templates over " (templates.total_spends) " spends, of which "
            (envelopes.len()) " templates and " (envelope_spends) " spends contain an "
            code { "OP_0 OP_IF" } " envelope (eg. inscriptions) and are listed separately."
            @if templates.overflow_spends > 0 {
                " " (templates.overflow_spends) " spends are not counted because the maximum number of tracked templates was reached."
            }
        }
        p {
            "Keys and hashes are replaced by placeholders: " code { "<pk>" } " compressed key, "
            code { "<xpk>" } " x-only key, " code { "<20>" } " 20 bytes push (eg. HASH160), "
            code { "<32>" } " 32 bytes push, " code { "<height>" } "/" code { "<time>" }
            " absolute timelocks. Relative timelocks are kept as numbers, eg. the push "
            code { "ffff00" } " before " code { "OP_CSV" } " is " code { "65535" }
            ". Opcodes use short names, eg. " code { "OP_1" } " and " code { "OP_CSV" } "."
        }
        p {
            "Showing the top " (TABLE_ROWS) " templates per table. The filter searches the "
            a href=(CSV_NAME) { "full list" } " (CSV)."
        }
        p {
            input id="filter" type="search" size="60" placeholder="filter, eg. OP_TOALTSTACK or wsh OP_2";
            " " span id="filter-status" {}
        }
        h2 { "Script templates" }
        (table(&scripts, false))
        h2 { "Envelope templates" }
        (table(&envelopes, true))
        script { (PreEscaped(FILTER_JS)) }
    };

    Page {
        title: "Segwit script templates".to_string(),
        description: "Scripts revealed spending P2WSH, P2SH-P2WSH and taproot script path, with keys and hashes replaced by placeholders".to_string(),
        permalink: "segwit-script-templates".to_string(),
        charts: vec![
            anonymity_set_chart(script_type, false, "script template"),
            anonymity_set_chart(script_type, true, "envelope template"),
        ],
        content: Some(content),
        files: vec![(CSV_NAME.to_string(), csv(&rows))],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::segwit_templates::SpendKind;
    use blocks_iterator::bitcoin::hashes::Hash;
    use blocks_iterator::bitcoin::Txid;

    #[test]
    fn test_segwit_script_templates_page() {
        let mut script_type = ScriptType::default();
        let txid = Txid::all_zeros();
        let t = &mut script_type.segwit_templates;
        // OP_2 <pk> <pk> OP_2 OP_CHECKMULTISIG, twice with the same keys
        let multisig = hex::decode(format!("5221{}21{}52ae", "02".repeat(33), "03".repeat(33)));
        for _ in 0..2 {
            t.add_script(SpendKind::Wsh, &multisig.clone().unwrap(), &txid, 10);
        }
        // <pk> OP_CHECKSIG with a quote in an envelope tag
        let envelope = hex::decode(format!("21{}ac0063022a2268", "02".repeat(33))).unwrap();
        t.add_script(SpendKind::Tr(0xc0), &envelope, &txid, 20);

        let page = segwit_script_templates(&script_type);

        let (name, csv) = &page.files[0];
        assert_eq!(name, "templates.csv");
        let zeros = "0".repeat(64);
        assert_eq!(
            csv,
            &format!(
                "kind,template,spends,distinct_scripts,distinct_exact,first_height,first_txid\n\
                 wsh,\"OP_2 <pk> <pk> OP_2 OP_CHECKMULTISIG\",2,1,true,10,{zeros}\n\
                 tr,\"<pk> OP_CHECKSIG OP_0 OP_IF <envelope:*\"\"> OP_ENDIF\",1,1,true,20,{zeros}\n"
            )
        );

        let html = page.content.unwrap().into_string();
        let script_table = html
            .find("data-envelope=\"false\" data-total=\"2\"")
            .unwrap();
        let envelope_table = html
            .find("data-envelope=\"true\" data-total=\"1\"")
            .unwrap();
        let multisig_row = html
            .find("<td>1</td><td>wsh</td><td><code>OP_2 &lt;pk&gt; &lt;pk&gt; OP_2 OP_CHECKMULTISIG</code></td><td>2</td><td>100.00</td><td>1</td>")
            .unwrap();
        let envelope_row = html.find("&lt;envelope:*&quot;&gt;").unwrap();
        assert!(script_table < multisig_row && multisig_row < envelope_table);
        assert!(envelope_table < envelope_row);
    }
}
