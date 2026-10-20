#!/usr/bin/env bash
set -euo pipefail
export BUILD_COMMIT="${GITHUB_SHA:-$(git rev-parse HEAD)}"
cargo build --locked --release --target wasm32-unknown-unknown
WASM_BINDGEN_VERSION=$(python3 - <<'PY'
import tomllib
with open('Cargo.lock', 'rb') as lock:
    packages = tomllib.load(lock)['package']
print(next(p['version'] for p in packages if p['name'] == 'wasm-bindgen'))
PY
)
if ! command -v wasm-bindgen >/dev/null || [[ "$(wasm-bindgen --version)" != "wasm-bindgen $WASM_BINDGEN_VERSION" ]]; then
  cargo install wasm-bindgen-cli --version "$WASM_BINDGEN_VERSION" --locked
fi
mkdir -p dist/pkg
wasm-bindgen --target web --out-dir dist/pkg --out-name lord_of_towers target/wasm32-unknown-unknown/release/lord-of-towers.wasm
cp web/index.html dist/index.html
BUILD_SHA="${GITHUB_SHA:-$(git rev-parse HEAD)}" python3 - <<'PY'
import os
from pathlib import Path
p = Path('dist/index.html')
p.write_text(p.read_text().replace('__BUILD_SHA__', os.environ['BUILD_SHA'][:7]))
PY
touch dist/.nojekyll
