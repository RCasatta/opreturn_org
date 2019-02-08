---
title: veriblock
layout: chart.liquid
permalink: /{{ name }}
description: Charts showing the number of output scripts and fee spent per month by veriblock.
---

<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.op_return.veriblock_per_month.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.op_return.veriblock_per_month.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'line',
    data: {
        labels: labels,
        datasets: [{
            label: '# of veriblock outputs',
            data: values,
            backgroundColor: 'rgba(54, 162, 235,0.5)',
            fill: true,
        }]
    }
});
</script>
<br><br>

<canvas id="myChart2" width="100%"></canvas>
<script>
var labels = {{ site.data.op_return.veriblock_fee_per_month.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.op_return.veriblock_fee_per_month.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart2").getContext('2d');
var myChart2 = new Chart(ctx, {
    type: 'line',
    data: {
        labels: labels,
        datasets: [{
            label: 'veriblock txs fees [bitcoin]',
            data: values,
            backgroundColor: window.chartColors.orange,
            fill: true,
        }]
    }
});
</script>
