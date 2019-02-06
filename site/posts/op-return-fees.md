---
title: OP_RETURN fee per month
layout: chart.liquid
permalink: /{{ name }}
description: Chart showing the sum of the fees of transactions per month containing an OP_RETURN script.
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
            label: '# of OP_RETURN outputs',
            data: values,
            backgroundColor: 'rgba(54, 162, 235,0.5)',
            fill: true,
        }]
    }
});
</script>
