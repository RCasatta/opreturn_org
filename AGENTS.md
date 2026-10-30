# Repository Guidelines

## Project Structure & Module Organization
The input is a stream of serialized `BlockExtra` (blocks with the previous outputs of their inputs) produced by [blocks_iterator](https://github.com/RCasatta/blocks_iterator) (`blocks_iterator_cli`) and read with its `PipeIterator`. In production `blocks_iterator_cli` runs as a continuous daemon over the full mainnet chain piped into `opreturn_org`, which also maintains the used-scriptPubKeys Bloom state dir; the site is served from `--target-dir/site` only, `raw/` is not public.

`src/main.rs` is the binary entrypoint. It reads block data from stdin, fans work out to processors in `src/process/`, and writes generated output under `--target-dir/{site,raw}`. Chart rendering lives in `src/charts.rs`, reusable counters in `src/counter.rs`, and each HTML page in `src/pages/` as one module per report, such as `src/pages/op_return.rs` or `src/pages/bip69.rs`. Static assets are limited; the bundled favicon is stored at `target_dir/site/favicon.ico`.

## Build, Test, and Development Commands
Use Cargo for day-to-day work:

- `cargo run -- --target-dir /tmp/opreturn` builds the binary and writes output for a local run.
- `cargo build --release` produces the optimized binary used in the README pipeline example.
- `cargo test` runs the unit tests embedded in `src/` and the end-to-end snapshot test in `tests/e2e.rs`.
- `scripts/dev.sh [STREAM]` builds the site from a block stream (default `tests/fixtures/testnet.bin`, `.zst` accepted) and serves it on `http://localhost:8000`.
- `cargo fmt -- --check` and `cargo clippy --all-targets -- -D warnings` must pass (CI enforces both).
- `nix flake check` runs the Nix-defined checks, including fmt, clippy, docs, audit, deny, and nextest.

Example data pipeline:
```sh
blocks_iterator_cli --network testnet --blocks-dir "$HOME/.bitcoin/testnet3/blocks/" \
  --stop-at-height 200000 | cargo run --release -- --target-dir /tmp/opreturn
```

### Test data
- `tests/fixtures/testnet.bin`: committed stream of the first few hundred testnet blocks (see its README). Fast, but has no segwit, taproot or OP_RETURN data, so most charts are empty.
- Mainnet sample: run the binary with `--sample-out FILE [--sample-every N]` (default N=1000) on a full-chain run to save every block whose height is a multiple of N, with prevouts, in the stdin format. Compress it with `zstd` and keep it out of git (`data/` is ignored), then iterate with `scripts/dev.sh data/sample.bin.zst`. Charts are sparse because only one block per N is present.
- Set `SOURCE_DATE_EPOCH` (unix seconds) to fix the "current time" used in pages; with it, the output is byte-for-byte reproducible.

## Coding Style & Naming Conventions
Follow `rustfmt` defaults: 4-space indentation, trailing commas where rustfmt adds them, and grouped `use` items only when it improves readability. Prefer snake_case for functions, modules, and files; use CamelCase for structs and enums. Keep page modules focused on one chart/report each, and keep helper logic in `process/` or shared modules instead of duplicating it across pages.

## Testing Guidelines
Tests live next to the code they cover under `#[cfg(test)]`. Add focused unit tests when changing counters, parsers, or page generation helpers. `tests/e2e.rs` runs the binary on the testnet fixture and compares the generated HTML with `tests/snapshots/site`; when a page change is intended, regenerate with `UPDATE_SNAPSHOTS=1 cargo test --test e2e` and review the snapshot diff in the commit. Refactors must leave the snapshot unchanged. Match existing naming like `test_chart`, `sort_outputs_1`, or `test_witness_stats`. Run `cargo test` before opening a PR; use `cargo test pages::bip69::tests::test_tx -- --exact` for targeted checks.

## Commit & Pull Request Guidelines
Recent commits use short, imperative subjects such as `fix order of element in opret sizes` and `split the op_return sizes chart`. Keep commit titles concise and specific to one change. Pull requests should describe the user-visible effect, note any data or pipeline assumptions, and include screenshots or generated HTML snippets when page output changes.

## Output & Configuration Notes
Do not commit generated `site/` or `raw/` output unless the change explicitly requires fixtures. Treat blockchain input paths and `--target-dir` as local environment details; prefer command examples with temporary directories.
