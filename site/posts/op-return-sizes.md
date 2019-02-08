---
title: OP_RETURN sizes
layout: chart.liquid
permalink: /{{ name }}
description: Chart showing the distribution of the sizes of the OP_RETURN scripts since inception.
---

<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.op_return.op_ret_size.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.op_return.op_ret_size.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'bar',
    data: {
        labels: labels,
        datasets: [{
            label: 'OP_RETURN sizes [bytes]',
            data: values,
            backgroundColor: window.chartColors.purple,
            fill: true,
        }]
    }
});
</script>
