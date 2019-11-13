---
title: Native segwit multisig
layout: chart.liquid
permalink: /{{ name }}
description: Analyze segwit input scripts counting the occurence of the NofM
---

<br><br>
<h2 style="text-align:center">{{ page.title }}</h2>
<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.script_type.segwit_multisig_other.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.script_type.segwit_multisig_other.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'pie',
    data: {
        labels: labels,
        datasets: [{
            label: 'NofM',
            data: values,
            backgroundColor: rainbowPalette,
            fill: true,
        }]
    }
});
</script>
