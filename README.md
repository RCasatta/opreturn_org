# Rustat

Generate statistic charts about the Bitcoin blockchain.

# Test

```
blocks_iterator --network testnet --blocks-dir $HOME/.bitcoin/testnet3/blocks/ --stop-at-height 200000 | ./target/release/opreturn_org --target-dir /tmp/
```

To also maintain a Bloom filter of every non-OP_RETURN scriptPubKey:

```
blocks_iterator_cli --network bitcoin --blocks-dir $HOME/.bitcoin/blocks/ | \
  ./target/release/opreturn_org \
    --target-dir /tmp/ \
    --used-scriptpubkeys-bloom-dir /tmp/used-scriptpubkeys-bloom \
    --bloom-false-positive-rate 0.001 \
    --bloom-expected-items 5000000000
```

The false-positive rate is a probability between zero and one, so `0.001` means
0.1%. The state directory is optional; no Bloom filter memory is allocated when it
is omitted. The first completed run writes `base.bloom`. Later runs load that base
and the existing deltas, ignore blocks through the stored tip, and atomically add
one delta for newly processed blocks. Delete the directory to rebuild the base or
change its parameters.

`blocks_iterator_cli` defaults to `--max-reorg 6`, so the stored Bloom tip is six
blocks behind the discovered chain tip. Delta headers also verify height and block
hash continuity when they are loaded.

# TODO

* move to svg created from rust, remove javascript
* use picocss, put pie chart in grid

# DONE

* move to 1000 blocks as labels (with date between parenthesis) so period more rightly compared
* merge periods until there are less than N points on the chart. N=100? more? Latest point could be estimated by duplicating latest complete value?
