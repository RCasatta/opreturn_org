---
title: Script types
layout: chart.liquid
permalink: /{{ name }}
description: Charts showing the script types per month.
---

<canvas id="myChart"></canvas>
<script>
var labels = {{ site.data.script_type.all.labels | join: "','" | prepend: "['" | append : "']"}};
var all = {{ site.data.script_type.all.values | join: "," | prepend: "[" | append: "]"}};
var p2pkh = {{ site.data.script_type.p2pkh.values | join: "," | prepend: "[" | append: "]"}};
var p2pk = {{ site.data.script_type.p2pk.values | join: "," | prepend: "[" | append: "]"}};
var v0_p2wpkh = {{ site.data.script_type.v0_p2wpkh.values | join: "," | prepend: "[" | append: "]"}};
var v0_p2wsh = {{ site.data.script_type.v0_p2wsh.values | join: "," | prepend: "[" | append: "]"}};
var p2sh = {{ site.data.script_type.p2sh.values | join: "," | prepend: "[" | append: "]"}};
var other = {{ site.data.script_type.other.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'line',
    data: {
        labels: labels,
        datasets: [{
            label: 'all',
            data: all,
            backgroundColor: window.chartColors.blue,
            borderColor: window.chartColors.blue,
            hidden: true,
            fill: false,
        },
        {
            label: 'p2pkh',
            data: p2pkh,
            backgroundColor: window.chartColors.green,
            borderColor: window.chartColors.green,
            fill: false,
        },
        {
            label: 'p2pk',
            data: p2pk,
            backgroundColor: window.chartColors.red,
            borderColor: window.chartColors.red,
            fill: false,
        },
        {
            label: 'v0_p2wpkh',
            data: v0_p2wpkh,
            backgroundColor: window.chartColors.yellow,
            borderColor: window.chartColors.yellow,
            fill: false,
        },
        {
            label: 'v0_p2wsh',
            data: v0_p2wsh,
            backgroundColor: window.chartColors.grey,
            borderColor: window.chartColors.grey,
            fill: false,
        },
        {
            label: 'p2sh',
            data: p2sh,
            backgroundColor: window.chartColors.purple,
            borderColor: window.chartColors.purple,
            fill: false,
        },
        {
            label: 'other',
            data: other,
            backgroundColor: window.chartColors.orange,
            borderColor: window.chartColors.orange,
            fill: false,
        }
        ]
    }
});
</script>
<br><br>
