---
title: Tx, inputs and outputs
layout: chart.liquid
permalink: /{{ name }}
description: Charts showing total number of transactions, inputs and outputs per month
---

<canvas id="myChart" width="100%"></canvas>
<script>
var labels = {{ site.data.stats.total_outputs_per_month.labels | join: "','" | prepend: "['" | append : "']"}};
var outputs = {{ site.data.stats.total_outputs_per_month.values | join: "," | prepend: "[" | append: "]"}};
var inputs = {{ site.data.stats.total_inputs_per_month.values | join: "," | prepend: "[" | append: "]"}};
var tx = {{ site.data.stats.total_tx_per_month.values | join: "," | prepend: "[" | append: "]"}};
var outputs_per_tx = outputs.map(function(n,i) { return n / tx[i]; });
var inputs_per_tx = inputs.map(function(n,i) { return n / tx[i]; });
var ctx = document.getElementById("myChart").getContext('2d');
var myChart = new Chart(ctx, {
    type: 'line',
    data: {
        labels: labels,
        datasets: [{
            label: 'total tx per month',
            data: tx,
            backgroundColor: window.chartColors.blue,
            borderColor: window.chartColors.blue,
            fill: false,
	    yAxisID: 'y-axis-1',
        },{
           label: 'total outputs per month',
           data: outputs,
           backgroundColor: window.chartColors.red,
           borderColor: window.chartColors.red,
           fill: false,
	    yAxisID: 'y-axis-1',
       },{
          label: 'total inputs per month',
          data: inputs,
          backgroundColor: window.chartColors.orange,
          borderColor: window.chartColors.orange,
          fill: false,
	    yAxisID: 'y-axis-1',
      },{
          label: 'average outputs per tx per month',
          data: outputs_per_tx,
          backgroundColor: window.chartColors.purple,
          borderColor: window.chartColors.purple,
	  borderDash: [5, 5],
          fill: false,
	    yAxisID: 'y-axis-2',
      },{
          label: 'average inputs per tx per month',
          data: inputs_per_tx,
          backgroundColor: window.chartColors.green,
          borderColor: window.chartColors.green,
	  borderDash: [5, 5],
          fill: false,
	    yAxisID: 'y-axis-2',
      }]
    },
   options: {
   scales: {
		yAxes: [{
			type: 'linear',
			display: true,
			position: 'left',
			id: 'y-axis-1',
		}, {
			type: 'linear',
			display: true,
			position: 'right',
			id: 'y-axis-2',
			gridLines: {
				drawOnChartArea: false,
			},
		}],
	}
	}
});
</script>

<br>
<br>
