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
package_dir="dist/pkg/${BUILD_COMMIT}"
mkdir -p "$package_dir"
wasm-bindgen --target web --out-dir "$package_dir" --out-name lord_of_towers target/wasm32-unknown-unknown/release/lord-of-towers.wasm
cp web/index.html dist/index.html
cp THIRD_PARTY_NOTICES.md dist/THIRD_PARTY_NOTICES.md
cp third_party/magna-urbis/LICENSE dist/MAGNA_URBIS_LICENSE.txt
python3 - <<'PY'
import os
from pathlib import Path
p = Path('dist/index.html')
build = os.environ['BUILD_COMMIT']
p.write_text(p.read_text().replace('__BUILD_SHA__', build[:7]).replace('__BUILD_ID__', build))
PY
touch dist/.nojekyll
