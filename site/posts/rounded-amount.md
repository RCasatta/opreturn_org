---
title: Rounded amount
layout: chart.liquid
permalink: /{{ name }}
description: Charts showing the number of outputs which have a rounded amount as value (multiple of 1000)
---

<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.stats.rounded_amount_per_month.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.stats.rounded_amount_per_month.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'line',
    data: {
        labels: labels,
        datasets: [{
            label: '# of outputs with rounded amount',
            data: values,
            backgroundColor: window.chartColors.blue,
            fill: true,
        }]
    }
});
</script>

<br>
<br>
