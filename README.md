# Rustat

Generate statistic charts about the Bitcoin blockchain.

# Test

```
cat testnet_blocks.dat | cargo run -- --target-dir /tmp >>/dev/null
```

# TODO

* move to svg created from rust, remove javascript
* use picocss, put pie chart in grid

# DONE

* move to 1000 blocks as labels (with date between parenthesis) so period more rightly compared
* merge periods until there are less than N points on the chart. N=100? more? Latest point could be estimated by duplicating latest complete value?

