---
title: OP_RETURN per month
layout: chart.liquid
permalink: /{{ name }}
description: Charts showing the number of OP_RETURN scripts and fee spent per month.
---

<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.op_return.op_ret_per_month.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.op_return.op_ret_per_month.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'line',
    data: {
        labels: labels,
        datasets: [{
            label: '# of OP_RETURN outputs',
            data: values,
            backgroundColor: window.chartColors.blue,
            fill: true,
        }]
    }
});
</script>

<br>

<canvas id="myChart2" width="100%"></canvas>
<script>
var labels = {{ site.data.op_return.op_ret_fee_per_month.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.op_return.op_ret_fee_per_month.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart2").getContext('2d');
var myChart2 = new Chart(ctx, {
    type: 'line',
    data: {
        labels: labels,
        datasets: [{
            label: 'fees of OP_RETURN tx [bitcoin]',
            data: values,
            backgroundColor: window.chartColors.orange,
            fill: true,
        }]
    }
});
</script>
<br>

<div> Total btc spent on tx containing OP_RETURN: {{ site.data.op_return.totals.op_ret_fee }} BTC </div>

<br>
<br>
