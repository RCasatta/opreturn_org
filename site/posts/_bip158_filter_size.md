---
title: BIP158 filter size
layout: chart.liquid
permalink: /{{ name }}
description: Charts showing the cumulative BIP158 filter size
---

<h2 style="text-align:center">{{ page.title }}</h2>

<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.bip158_stats.bip158_filter_size_per_month_cum.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.bip158_stats.bip158_filter_size_per_month_cum.values | join: "," | prepend: "[" | append: "]"}};
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
