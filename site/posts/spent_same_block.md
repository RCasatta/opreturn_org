---
title: Spent in the same block
layout: chart.liquid
permalink: /{{ name }}
description: Charts showing the number of inputs which use an output created in the same block (per month).
---

<canvas id="myChart"></canvas>
<script>
var labels = {{ site.data.script_type.all.labels | join: "','" | prepend: "['" | append : "']"}};
var all = {{ site.data.script_type.all.values | join: "," | prepend: "[" | append: "]"}};
var spent_same_block =  {{ site.data.stats.total_spent_in_block_per_month.values | join: "," | prepend: "[" | append: "]"}};
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
            fill: false,
        },
        {
            label: 'spent_same_block',
            data: spent_same_block,
            backgroundColor: window.chartColors.green,
            borderColor: window.chartColors.green,
            fill: false,
        }
        ]
    }
});
</script>
<br><br>
