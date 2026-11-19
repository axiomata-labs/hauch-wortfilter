#!/bin/sh
# Builds the plugin into dist/: plugin.wasm and plugin.json (plugin.base.json with the version and the SHA-256 of
# plugin.wasm), the two assets of a release. `scripts/release.sh 0.1.0`; the release workflow runs it for a tag
# `v0.1.0`. Needs a Rust toolchain with the target wasm32-wasip2 (`rustup target add wasm32-wasip2`) and jq.
set -eu
cd "$(dirname "$0")/.."
VERSION="${1:?the version, such as 0.1.0}"
# rustup's own cargo and rustc where rustup is there: a Homebrew Rust first on the PATH has no wasm targets.
if command -v rustup >/dev/null 2>&1 || [ -x /opt/homebrew/opt/rustup/bin/rustup ]; then
  RUSTUP="$(command -v rustup || echo /opt/homebrew/opt/rustup/bin/rustup)"
  SYSROOT="$("$RUSTUP" run stable rustc --print sysroot)"
  export RUSTC="$SYSROOT/bin/rustc"
  CARGO="$SYSROOT/bin/cargo"
else
  CARGO=cargo
fi
"$CARGO" build --release --target wasm32-wasip2
mkdir -p dist
cp target/wasm32-wasip2/release/hauch_wortfilter.wasm dist/plugin.wasm
SHA="$(shasum -a 256 dist/plugin.wasm | cut -d' ' -f1)"
jq --arg version "$VERSION" --arg sha "$SHA" '. + {version: $version, sha256: $sha}' plugin.base.json > dist/plugin.json
ls -l dist
