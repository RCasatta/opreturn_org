---
title: Spent in the same block
layout: chart.liquid
permalink: /{{ name }}
description: Charts showing the number of output which are spent in the same block they are created
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
            fill: true,
        },{
           label: 'total outputs per month',
           data: outputs,
           backgroundColor: window.chartColors.red,
           fill: true,
       },{
          label: 'total inputs per month',
          data: inputs,
          backgroundColor: window.chartColors.violet,
          fill: true,
      }]
    }
});
</script>

<br>
<br>
