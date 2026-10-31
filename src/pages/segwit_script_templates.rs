use crate::charts::{Chart, Color, Dataset, Kind};
use crate::pages::Page;
use crate::process::segwit_templates::{is_envelope, TemplateRow, OTHER};
use crate::process::ScriptType;
use maud::{html, Markup, PreEscaped};

/// Rows shown in each table of the page, the full list is in [`CSV_NAME`]
const TABLE_ROWS: usize = 500;

const CSV_NAME: &str = "templates.csv";

/// Converts a script in asm format to its template, mirroring [`crate::process::segwit_templates::template`]
const SCRIPT_TEMPLATE_JS: &str = include_str!("js/script_template.js");

/// Filters the tables, see the file for details
const FILTER_JS: &str = include_str!("js/templates_filter.js");

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

/// `n` with comma separated thousands, as `toLocaleString('en-US')` does in javascript
fn thousands(n: u64) -> String {
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

/// The first [`TABLE_ROWS`] of `rows`, percentages are relative to the spends of all `rows`
fn table(rows: &[&TemplateRow], envelope: bool) -> Markup {
    let total: u64 = rows.iter().map(|r| r.spends).sum();
    html! {
        div class="table-wrap" {
            table class="templates" data-envelope=(envelope) {
                thead {
                    tr {
                        th class="num" { "#" }
                        th { "kind" }
                        th { "template" }
                        th class="num" { "spends" }
                        th class="num" { "% spends" }
                        th class="num" title="distinct scripts using the template, ~ when estimated" { "distinct scripts" }
                        th class="num" title="height of the first spend, links to the transaction" { "first seen" }
                    }
                }
                tbody {
                    @for (i, row) in rows.iter().take(TABLE_ROWS).enumerate() {
                        @let (kind, template) = split_kind(row);
                        tr {
                            td class="num" { (i + 1) }
                            td class="nowrap" { (kind) }
                            td { code { (template) } }
                            td class="num" { (thousands(row.spends)) }
                            td class="num" { (format!("{:.2}", row.spends as f64 * 100.0 / total as f64)) }
                            td class="num" { @if !row.distinct_exact { "~" } (thousands(row.distinct_scripts)) }
                            td class="num" { a href=(format!("https://mempool.space/tx/{}", row.first_txid)) { (row.first_height) } }
                        }
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
            (thousands(rows.len() as u64)) " distinct templates over " (thousands(templates.total_spends)) " spends, of which "
            (thousands(envelopes.len() as u64)) " templates and " (thousands(envelope_spends)) " spends contain an "
            code { "OP_0 OP_IF" } " envelope (eg. inscriptions) and are listed separately."
            @if templates.overflow_spends > 0 {
                 " " (thousands(templates.overflow_spends)) " spends are not counted because the maximum number of tracked templates was reached."
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
            a href=(CSV_NAME) { "full list" } " (CSV) for templates containing the given text. "
            "A script pasted as shown by mempool.space, eg. "
            code { "OP_DUP OP_HASH160 OP_PUSHBYTES_20 e92c61fd47c4e776e91495e121d29989812fbbfb OP_EQUALVERIFY OP_CHECKSIG" }
            ", is converted to its template first."
        }
        p {
            textarea id="filter" rows="4" cols="80" placeholder="filter, eg. OP_TOALTSTACK, wsh OP_2, or a script copied from mempool.space" {}
            br;
            span id="filter-template" {}
            br;
            span id="filter-status" {}
        }
        h2 { "Script templates" }
        (table(&scripts, false))
        h2 { "Envelope templates" }
        (table(&envelopes, true))
        script { (PreEscaped(SCRIPT_TEMPLATE_JS)) (PreEscaped(FILTER_JS)) }
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
    use crate::process::segwit_templates::{template, SpendKind};
    use bitcoin::hashes::Hash;
    use bitcoin::opcodes::all::*;
    use bitcoin::script::{Builder, PushBytesBuf};
    use bitcoin::{ScriptBuf, Txid};
    use std::convert::TryFrom;
    use std::io::Write;
    use std::process::{Command, Stdio};

    /// Deterministic pseudo random scripts, built from a vocabulary exercising all the template
    /// rules: envelopes, timelocks, 32 bytes contexts, keys and small numbers
    fn random_scripts(count: usize) -> Vec<ScriptBuf> {
        let ops = [
            OP_IF,
            OP_NOTIF,
            OP_ELSE,
            OP_ENDIF,
            OP_CHECKSIG,
            OP_CHECKSIGVERIFY,
            OP_CHECKSIGADD,
            OP_EQUAL,
            OP_EQUALVERIFY,
            OP_SHA256,
            OP_HASH256,
            OP_HASH160,
            OP_CSV,
            OP_CLTV,
            OP_DUP,
            OP_ADD,
            OP_TOALTSTACK,
            OP_CHECKMULTISIG,
            OP_PUSHNUM_1,
            OP_PUSHNUM_16,
            OP_PUSHNUM_NEG1,
            OP_DROP,
        ];
        let lens = [0usize, 1, 2, 3, 4, 5, 6, 20, 32, 33, 65, 80];
        let mut state = 42u64;
        let mut next = move |n: usize| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as usize % n
        };
        (0..count)
            .map(|_| {
                let mut builder = Builder::new();
                for _ in 0..1 + next(25) {
                    builder = match next(4) {
                        0 | 1 => builder.push_opcode(ops[next(ops.len())]),
                        // consecutive envelopes, with or without a printable tag, usually closed
                        2 if next(4) == 0 => {
                            let tag: &[u8] = [&b"ord"[..], b"a b", b"\x01x"][next(3)];
                            for _ in 0..1 + next(3) {
                                builder = builder
                                    .push_slice([0u8; 0])
                                    .push_opcode(OP_IF)
                                    .push_slice(PushBytesBuf::try_from(tag.to_vec()).unwrap());
                                if next(4) != 0 {
                                    builder = builder.push_opcode(OP_ENDIF);
                                }
                            }
                            builder
                        }
                        _ => {
                            let len = lens[next(lens.len())];
                            let mut data: Vec<u8> = (0..len).map(|_| next(256) as u8).collect();
                            match (len, next(2)) {
                                (33, 0) => data[0] = 2 + next(2) as u8,
                                (65, 0) => data[0] = 4,
                                _ => (),
                            }
                            builder.push_slice(PushBytesBuf::try_from(data).unwrap())
                        }
                    };
                }
                builder.into_script()
            })
            .collect()
    }

    #[test]
    fn test_script_template_js_matches_rust() {
        // the example from the page, as copied from mempool.space
        let mut scripts = vec![ScriptBuf::from_hex("6376a914e92c61fd47c4e776e91495e121d29989812fbbfb88ac6b76a9146c88322160c87df9165f0cae54d4b0c8b05616b588ac6c93518803ffff00b267210324c47c81f4a2df9bfff087d6b748b356258b452dffb11e666d998c6306f8cafaad2102061cbe5b925dc4ac02a0e003beda11e04b6d3e6ca8a0dbf550980b87af2a5052ac68").unwrap()];
        scripts.extend(random_scripts(2000));
        let asms: Vec<_> = scripts.iter().map(|s| s.to_asm_string()).collect();

        let js = format!(
            "{}\nconst input = JSON.parse(require('fs').readFileSync(0, 'utf8'));\n\
             process.stdout.write(JSON.stringify(input.map(scriptTemplate)));",
            SCRIPT_TEMPLATE_JS
        );
        let mut child = Command::new("node")
            .arg("-e")
            .arg(js)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("node is required to test the javascript");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(serde_json::to_string(&asms).unwrap().as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let js_templates: Vec<String> = serde_json::from_slice(&output.stdout).unwrap();

        assert_eq!(
            js_templates[0],
            "OP_IF OP_DUP OP_HASH160 <20> OP_EQUALVERIFY OP_CHECKSIG OP_TOALTSTACK OP_DUP OP_HASH160 <20> OP_EQUALVERIFY OP_CHECKSIG OP_FROMALTSTACK OP_ADD OP_1 OP_EQUALVERIFY 65535 OP_CSV OP_ELSE <pk> OP_CHECKSIGVERIFY <pk> OP_CHECKSIG OP_ENDIF"
        );
        for ((script, asm), js_template) in scripts.iter().zip(asms.iter()).zip(js_templates) {
            assert_eq!(template(script.as_bytes()), js_template, "asm: {}", asm);
        }
    }

    #[test]
    fn test_thousands() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(32714282), "32,714,282");
    }

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
        let script_table = html.find("data-envelope=\"false\"").unwrap();
        let envelope_table = html.find("data-envelope=\"true\"").unwrap();
        let multisig_row = html
            .find("<td class=\"num\">1</td><td class=\"nowrap\">wsh</td><td><code>OP_2 &lt;pk&gt; &lt;pk&gt; OP_2 OP_CHECKMULTISIG</code></td><td class=\"num\">2</td><td class=\"num\">100.00</td><td class=\"num\">1</td>")
            .unwrap();
        let envelope_row = html.find("&lt;envelope:*&quot;&gt;").unwrap();
        assert!(script_table < multisig_row && multisig_row < envelope_table);
        assert!(envelope_table < envelope_row);
    }
}
