// Filters the template tables on the whole templates.csv, loaded at the first use of the filter box
// or when the url contains `#q=<filter>`. Rows have the same columns rendered by `table` in
// src/pages/segwit_script_templates.rs. A script pasted in asm format is converted to its
// template with `scriptTemplate`, exact matches are shown first.
(function () {
  var MAX = 500;
  var input = document.getElementById('filter');
  var status = document.getElementById('filter-status');
  var searched = document.getElementById('filter-template');
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
      groups[false].total = groups[true].total = 0;
      text.split('\n').forEach(function (line) {
        var m = re.exec(line);
        if (!m) return;
        var row = { kind: m[1], template: m[2].replace(/""/g, '"'), spends: +m[3],
                    distinct: +m[4], exact: m[5] === 'true', height: +m[6], txid: m[7] };
        row.key = (row.kind + ' ' + row.template).toLowerCase();
        var group = groups[row.template.indexOf('<envelope') >= 0];
        row.rank = group.length + 1;
        group.total += row.spends;
        group.push(row);
      });
      done();
    }).catch(function (e) { loading = false; status.textContent = 'cannot load the full list: ' + e; });
  }

  // a script copied from mempool.space contains explicit pushes, a template never does
  function isScript(text) {
    return /OP_PUSHBYTES_|OP_PUSHDATA|OP_PUSHNUM_/.test(text);
  }

  function td(tr, child) {
    var cell = document.createElement('td');
    cell.append(child);
    tr.append(cell);
    return cell;
  }

  function render() {
    var text = input.value.trim();
    var template = isScript(text) ? scriptTemplate(text) : null;
    searched.textContent = template ? 'searching the template: ' + template : '';
    var query = (template || text).replace(/\s+/g, ' ').toLowerCase();
    var counts = [], exact = [];
    tables.forEach(function (table) {
      var group = groups[table.dataset.envelope === 'true'];
      var rows = group.filter(function (r) {
        return r.key.indexOf(query) >= 0;
      });
      // exact matches first, then templates containing the query
      rows.sort(function (a, b) {
        return (b.template.toLowerCase() === query) - (a.template.toLowerCase() === query) || a.rank - b.rank;
      });
      rows.forEach(function (r) { if (r.template.toLowerCase() === query) exact.push(r); });
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
        td(tr, (r.spends * 100 / group.total).toFixed(2));
        td(tr, (r.exact ? '' : '~') + r.distinct);
        var a = document.createElement('a');
        a.href = 'https://mempool.space/tx/' + r.txid;
        a.textContent = String(r.height);
        td(tr, a);
        tbody.append(tr);
      });
    });
    var summary = counts.join(' script and ') + ' envelope templates match' +
      (counts.some(function (c) { return c > MAX; }) ? ', showing the first ' + MAX : '');
    if (template) {
      summary += exact.length
        ? '. Exact match: ' + exact.map(function (r) { return r.kind + ' rank ' + r.rank + ' with ' + r.spends + ' spends'; }).join(', ')
        : '. No template is exactly the pasted script, templates containing it are listed';
    }
    status.textContent = summary;
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
