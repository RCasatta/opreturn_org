---
title: Blockchain size
layout: chart.liquid
permalink: /{{ name }}
description: Charts showing the blockchain size (only blocks)
---

<h2 style="text-align:center">{{ page.title }}</h2>

<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.stats.block_size_per_month.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.stats.block_size_per_month.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'line',
    data: {
        labels: labels,
        datasets: [{
            label: 'size',
            data: values,
            backgroundColor: window.chartColors.blue,
            fill: true,
        }]
    }
});
</script>

<br>
