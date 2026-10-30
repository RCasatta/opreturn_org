#!/usr/bin/env bash
# Build the site from a block stream and serve it locally.
#
# Usage: scripts/dev.sh [STREAM]
#   STREAM  serialized BlockExtra stream, default tests/fixtures/testnet.bin. A `.zst` file is
#           decompressed on the fly, eg. a mainnet sample produced with `--sample-out`
# Env: OUT (output dir, default $TMPDIR/opreturn-dev), PORT (default 8000)
set -euo pipefail

input=${1:-tests/fixtures/testnet.bin}
out=${OUT:-${TMPDIR:-/tmp}/opreturn-dev}
port=${PORT:-8000}

mkdir -p "$out"
case "$input" in
*.zst) reader=(zstd -dc "$input") ;;
*) reader=(cat "$input") ;;
esac

"${reader[@]}" | cargo run --release -- --target-dir "$out"

echo "serving $out/site on http://localhost:$port"
exec python3 -m http.server --directory "$out/site" "$port"
