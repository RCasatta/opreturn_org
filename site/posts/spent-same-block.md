---
title: Spent in the same block
layout: chart.liquid
permalink: /{{ name }}
description: Charts showing the number of output which are spent in the same block they are created
---

<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.stats.total_spent_in_block_per_month.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.stats.total_spent_in_block_per_month.values | join: "," | prepend: "[" | append: "]"}};
var total_values = {{ site.data.stats.total_outputs_per_month.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'line',
    data: {
        labels: labels,
        datasets: [{
            label: 'total outputs spent in the same block per month',
            data: values,
            backgroundColor: window.chartColors.blue,
            fill: true,
        },{
          label: 'total outputs per month',
          data: total_values,
          backgroundColor: window.chartColors.red,
          fill: true,
      }]
    }
});
</script>

<br>
<br>
