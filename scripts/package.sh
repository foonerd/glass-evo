#!/bin/bash
# Assemble the component the Glass Manager installs: manifest.json and the
# binaries by architecture, as dist/glass-evo-<version>.zip. The manifest
# names the version, the build (commit and time) and, per architecture,
# the binary's path and its sha256, which the Manager checks on install.
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$ROOT"
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT
mkdir -p "$STAGE" dist
entries=""
for arch in arm armv7 armv8 x64; do
  bin="bin/$arch/glass-evo"
  [ -x "$bin" ] || { echo "package: $bin is missing; run scripts/ship.sh first" >&2; exit 1; }
  install -D -m 755 "$bin" "$STAGE/bin/$arch/glass-evo"
  sum=$(sha256sum "$bin" | cut -d' ' -f1)
  entries="$entries$([ -n "$entries" ] && printf ',')\n    \"$arch\": { \"path\": \"bin/$arch/glass-evo\", \"sha256\": \"$sum\" }"
done
printf '{\n  "name": "glass-evo",\n  "version": "%s",\n  "built": { "commit": "%s", "time": "%s" },\n  "binaries": {%b\n  }\n}\n' \
  "$VERSION" \
  "$(git rev-parse --short HEAD 2>/dev/null || echo unknown)" \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  "$entries" > "$STAGE/manifest.json"
python3 -c "import json,sys; json.load(open(sys.argv[1]))" "$STAGE/manifest.json"
( cd "$STAGE" && rm -f "$ROOT/dist/glass-evo-$VERSION.zip" && zip -qr "$ROOT/dist/glass-evo-$VERSION.zip" . )
cat "$STAGE/manifest.json"
ls -la "$ROOT/dist/glass-evo-$VERSION.zip"
