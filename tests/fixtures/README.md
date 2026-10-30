`testnet.bin` is a stream of serialized `BlockExtra` (the format `opreturn_org` reads from stdin)
of the first testnet blocks, generated from the
[blocks_iterator](https://github.com/RCasatta/blocks_iterator) fixture
`blocks/blk-testnet.dat` (commit 0e60a3a):

```sh
blocks_iterator_cli --blocks-dir blocks_iterator/blocks --network testnet --max-reorg 40 \
  > tests/fixtures/testnet.bin
```

It contains no segwit, taproot or OP_RETURN data, so it covers the plumbing only. For realistic
data, use a mainnet sample produced with `--sample-out` (see AGENTS.md).
