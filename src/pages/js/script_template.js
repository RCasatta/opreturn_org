// Converts a script in the asm format used by rust-bitcoin and mempool.space, eg.
// `OP_DUP OP_HASH160 OP_PUSHBYTES_20 e92c... OP_EQUALVERIFY OP_CHECKSIG`, into its template.
// It mirrors `template` in src/process/segwit_templates.rs, the test
// `test_script_template_js_matches_rust` checks they agree.
function scriptTemplate(asm) {
  var tokens = asm.trim().split(/\s+/).filter(function (t) { return t.length > 0; });

  // instructions as {op: name} or {push: [bytes]}
  var instructions = [];
  for (var t = 0; t < tokens.length; t++) {
    var token = tokens[t];
    var pushbytes = /^OP_PUSHBYTES_(\d+)$/.exec(token);
    if (token === 'OP_0' || token === 'OP_PUSHBYTES_0') {
      instructions.push({ push: [] });
    } else if (pushbytes || /^OP_PUSHDATA[124]$/.test(token)) {
      var hex = tokens[t + 1];
      if (hex !== undefined && /^([0-9a-fA-F]{2})*$/.test(hex)) {
        t++;
        instructions.push({ push: hex.match(/../g) ? hex.match(/../g).map(function (h) { return parseInt(h, 16); }) : [] });
      } else {
        // data not given, only its length is known
        instructions.push({ push: new Array(pushbytes ? +pushbytes[1] : 0).fill(0) });
      }
    } else {
      instructions.push({ op: token });
    }
  }

  function opAt(i) {
    var instruction = instructions[i];
    return instruction && instruction.op !== undefined ? instruction.op : null;
  }

  // script number, not requiring minimal encoding
  function scriptNum(bytes) {
    if (bytes.length === 0 || bytes.length > 5) return null;
    var v = 0;
    for (var i = 0; i < bytes.length; i++) v += bytes[i] * Math.pow(256, i);
    var last = bytes[bytes.length - 1];
    if (last & 0x80) v = -(v - 0x80 * Math.pow(256, bytes.length - 1));
    return v;
  }

  function opcodeName(op) {
    var pushnum = /^OP_PUSHNUM_(\d+)$/.exec(op);
    return pushnum ? 'OP_' + pushnum[1] : op;
  }

  function pushPlaceholder(data, prev, next) {
    var checksig = next === 'OP_CHECKSIG' || next === 'OP_CHECKSIGVERIFY' || next === 'OP_CHECKSIGADD';
    var equal = next === 'OP_EQUAL' || next === 'OP_EQUALVERIFY';
    var sha = prev === 'OP_SHA256' || prev === 'OP_HASH256';
    var n = scriptNum(data);
    if (next === 'OP_CLTV') {
      if (n === null) return '<data:' + data.length + '>';
      return n < 500000000 ? '<height>' : '<time>';
    }
    if (next === 'OP_CSV') return n === null ? '<data:' + data.length + '>' : String(n);
    if (data.length === 32 && checksig) return '<xpk>';
    if (data.length === 32 && equal && sha) return '<h32>';
    switch (data.length) {
      case 1: case 2: return String(n);
      case 20: return '<20>';
      case 32: return '<32>';
      case 33: if (data[0] === 2 || data[0] === 3) return '<pk>'; break;
      case 65: if (data[0] === 4) return '<pk65>'; break;
    }
    return '<data:' + data.length + '>';
  }

  var parts = [];
  var i = 0;
  while (i < instructions.length) {
    var instruction = instructions[i];
    var nextOp = opAt(i + 1);
    if (instruction.push !== undefined && instruction.push.length === 0 && nextOp === 'OP_IF') {
      // envelope: skip until the matching OP_ENDIF
      var tagged = instructions[i + 2];
      var tag = '';
      if (tagged && tagged.push !== undefined && tagged.push.length > 0 && tagged.push.length <= 16 &&
          tagged.push.every(function (c) { return c >= 0x21 && c <= 0x7e; })) {
        tag = ':' + String.fromCharCode.apply(null, tagged.push);
      }
      var depth = 0;
      var j = i + 1;
      for (; j < instructions.length; j++) {
        var op = opAt(j);
        if (op === 'OP_IF' || op === 'OP_NOTIF') {
          depth++;
        } else if (op === 'OP_ENDIF') {
          depth--;
          if (depth === 0) break;
        }
      }
      var envelope = 'OP_0 OP_IF <envelope' + tag + '>' + (j < instructions.length ? ' OP_ENDIF' : '');
      // consecutive identical envelopes are written once with their count
      var repeated = '(' + envelope + ')x';
      var last = parts[parts.length - 1];
      var count = null;
      if (last === envelope) {
        count = 2;
      } else if (last !== undefined && last.indexOf(repeated) === 0 && /^\d+$/.test(last.slice(repeated.length))) {
        count = parseInt(last.slice(repeated.length), 10) + 1;
      }
      if (count === null) parts.push(envelope);
      else parts[parts.length - 1] = repeated + count;
      i = j + 1;
      continue;
    }
    if (instruction.push !== undefined) {
      parts.push(instruction.push.length === 0 ? 'OP_0' : pushPlaceholder(instruction.push, opAt(i - 1), nextOp));
    } else {
      parts.push(opcodeName(instruction.op));
    }
    i++;
  }
  return parts.join(' ');
}
