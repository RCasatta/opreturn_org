---
title: Number of sighash type used
layout: chart.liquid
permalink: /{{ name }}
description: Show distribution of sighash type used
---

<br><br>
<h2 style="text-align:center">{{ page.title }}</h2>
<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.stats.sighashtype.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.stats.sighashtype.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'pie',
    data: {
        labels: labels,
        datasets: [{
            label: '#in-#out',
            data: values,
            backgroundColor: rainbowPalette,
            fill: true,
        }]
    },
    options: window.optionsForPercentage
});
</script>
