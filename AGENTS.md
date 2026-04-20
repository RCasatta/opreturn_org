# Repository Guidelines

## Project Structure & Module Organization
`src/main.rs` is the binary entrypoint. It reads block data from stdin, fans work out to processors in `src/process/`, and writes generated output under `--target-dir/{site,raw}`. Chart rendering lives in `src/charts.rs`, reusable counters in `src/counter.rs`, and each HTML page in `src/pages/` as one module per report, such as `src/pages/op_return.rs` or `src/pages/bip69.rs`. Static assets are limited; the bundled favicon is stored at `target_dir/site/favicon.ico`.

## Build, Test, and Development Commands
Use Cargo for day-to-day work:

- `cargo run -- --target-dir /tmp/opreturn` builds the binary and writes output for a local run.
- `cargo build --release` produces the optimized binary used in the README pipeline example.
- `cargo test` runs the unit tests embedded in `src/`.
- `cargo fmt -- --check` verifies formatting and currently passes.
- `cargo clippy --all-targets -- -D warnings` is the intended lint command, but it currently fails on pre-existing warnings in the repository.
- `nix flake check` runs the Nix-defined checks, including fmt, clippy, docs, audit, deny, and nextest.

Example data pipeline:
```sh
blocks_iterator --network testnet --blocks-dir "$HOME/.bitcoin/testnet3/blocks/" \
  --stop-at-height 200000 | cargo run --release -- --target-dir /tmp/opreturn
```

## Coding Style & Naming Conventions
Follow `rustfmt` defaults: 4-space indentation, trailing commas where rustfmt adds them, and grouped `use` items only when it improves readability. Prefer snake_case for functions, modules, and files; use CamelCase for structs and enums. Keep page modules focused on one chart/report each, and keep helper logic in `process/` or shared modules instead of duplicating it across pages.

## Testing Guidelines
Tests live next to the code they cover under `#[cfg(test)]`. Add focused unit tests when changing counters, parsers, or page generation helpers. Match existing naming like `test_chart`, `sort_outputs_1`, or `test_witness_stats`. Run `cargo test` before opening a PR; use `cargo test pages::bip69::tests::test_tx -- --exact` for targeted checks.

## Commit & Pull Request Guidelines
Recent commits use short, imperative subjects such as `fix order of element in opret sizes` and `split the op_return sizes chart`. Keep commit titles concise and specific to one change. Pull requests should describe the user-visible effect, note any data or pipeline assumptions, and include screenshots or generated HTML snippets when page output changes.

## Output & Configuration Notes
Do not commit generated `site/` or `raw/` output unless the change explicitly requires fixtures. Treat blockchain input paths and `--target-dir` as local environment details; prefer command examples with temporary directories.
