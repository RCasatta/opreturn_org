---
title: Tx, inputs and outputs
layout: chart.liquid
permalink: /{{ name }}
description: Charts showing total number of transactions, inputs and outputs per month
---

<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.stats.total_outputs_per_month.labels | join: "','" | prepend: "['" | append : "']"}};
var outputs = {{ site.data.stats.total_outputs_per_month.values | join: "," | prepend: "[" | append: "]"}};
var inputs = {{ site.data.stats.total_inputs_per_month.values | join: "," | prepend: "[" | append: "]"}};
var tx = {{ site.data.stats.total_tx_per_month.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'line',
    data: {
        labels: labels,
        datasets: [{
            label: 'total tx per month',
            data: tx,
            backgroundColor: window.chartColors.blue,
            borderColor: window.chartColors.blue,
            fill: false,
        },{
           label: 'total outputs per month',
           data: outputs,
           backgroundColor: window.chartColors.red,
           borderColor: window.chartColors.red,
           fill: false,
       },{
          label: 'total inputs per month',
          data: inputs,
          backgroundColor: window.chartColors.orange,
          borderColor: window.chartColors.orange,
          fill: false,
      }]
    }
});
</script>

<br>
<br>
