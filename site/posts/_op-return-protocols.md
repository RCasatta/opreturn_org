---
title: OP_RETURN protocols
layout: chart.liquid
permalink: /{{ name }}
description: Protocols just mean the first 3 bytes of the OP_RETURN data, which can indicate the protocol but it's not an enfoced rule by the Bitcoin consensus.
---

<br><br>
<h2 style="text-align:center">Last month</h2>
<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.op_return.op_ret_per_proto_last_month.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.op_return.op_ret_per_proto_last_month.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'pie',
    data: {
        labels: labels,
        datasets: [{
            label: 'OP_RETURN protocols',
            data: values,
            backgroundColor: rainbowPalette,
            fill: true,
        }]
    },
    options: window.optionsForPercentage
});
</script>

<br><br>
<h2 style="text-align:center">Last year</h2>
<canvas id="myChart2" width="100%"></canvas>
<script>
var labels = {{ site.data.op_return.op_ret_per_proto_last_year.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.op_return.op_ret_per_proto_last_year.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart2").getContext('2d');
var myChart2 = new Chart(ctx, {
    type: 'pie',
    data: {
        labels: labels,
        datasets: [{
            label: 'OP_RETURN protocols',
            data: values,
            backgroundColor: rainbowPalette,
            fill: true,
        }]
    },
    options: window.optionsForPercentage
});
</script>

<br><br>
<h2 style="text-align:center">Ever</h2>
<canvas id="myChart3" width="100%"></canvas>
<script>
var labels = {{ site.data.op_return.op_ret_per_proto.labels | join: "','" | prepend: "['" | append : "']"}};
var values = {{ site.data.op_return.op_ret_per_proto.values | join: "," | prepend: "[" | append: "]"}};
var ctx = document.getElementById("myChart3").getContext('2d');
var myChart3 = new Chart(ctx, {
    type: 'pie',
    data: {
        labels: labels,
        datasets: [{
            label: 'OP_RETURN protocols',
            data: values,
            backgroundColor: rainbowPalette,
            fill: true,
        }]
    },
    options: window.optionsForPercentage
});
</script>
