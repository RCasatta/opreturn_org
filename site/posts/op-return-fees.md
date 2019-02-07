---
title: OP_RETURN fee per month
layout: chart.liquid
permalink: /{{ name }}
description: Chart showing the sum of the fees of transactions containing an OP_RETURN script per month.
---

<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.op_return.op_ret_fee_per_month.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.op_return.op_ret_fee_per_month.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'line',
    data: {
        labels: labels,
        datasets: [{
            label: 'fees of OP_RETURN tx [bitcoin]',
            data: values,
            backgroundColor: 'rgba(54, 162, 235,0.5)',
            fill: true,
        }]
    }
});
</script>
