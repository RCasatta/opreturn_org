---
title: Witness stats
layout: chart.liquid
permalink: /{{ name }}
description: Stats about the witnesses, number of elements and bytes used
---


<br><br>
<h2 style="text-align:center">Number of elements in witness</h2>
<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.stats.has_witness.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.stats.has_witness.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'pie',
    data: {
        labels: labels,
        datasets: [{
            label: 'Inputs with or without elements in witness',
            data: values,
            backgroundColor: rainbowPalette,
            fill: true,
        }]
    },
    options: window.optionsForPercentage
});
</script>


<br><br>
<h2 style="text-align:center">Number of elements in witness</h2>
<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.stats.witness_elements.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.stats.witness_elements.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'pie',
    data: {
        labels: labels,
        datasets: [{
            label: 'Witness elements',
            data: values,
            backgroundColor: rainbowPalette,
            fill: true,
        }]
    },
    options: window.optionsForPercentage
});
</script>

<br><br>
<h2 style="text-align:center">Bytes in witness</h2>
<canvas id="myChart2" width="100%"></canvas>
<script>
var labels = {{ site.data.stats.witness_byte_size.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.stats.witness_byte_size.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart2").getContext('2d');
var myChart2 = new Chart(ctx, {
    type: 'pie',
    data: {
        labels: labels,
        datasets: [{
            label: 'Witness bytes',
            data: values,
            backgroundColor: rainbowPalette,
            fill: true,
        }]
    },
    options: window.optionsForPercentage
});
</script>
