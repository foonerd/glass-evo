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
for arch in arm armv7 armv8 x64; do
  bin="bin/$arch/glass-evo"
  [ -x "$bin" ] || { echo "package: $bin is missing; run scripts/ship.sh first" >&2; exit 1; }
  install -D -m 755 "$bin" "$STAGE/bin/$arch/glass-evo"
done
COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
python3 - "$STAGE" "$VERSION" "$COMMIT" <<'EOF'
import hashlib, json, sys, time
stage, version, commit = sys.argv[1:4]
binaries = {}
for arch in ("arm", "armv7", "armv8", "x64"):
    path = f"bin/{arch}/glass-evo"
    with open(f"{stage}/{path}", "rb") as f:
        binaries[arch] = {"path": path, "sha256": hashlib.sha256(f.read()).hexdigest()}
manifest = {
    "name": "glass-evo",
    "version": version,
    "built": {"commit": commit, "time": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())},
    "binaries": binaries,
}
with open(f"{stage}/manifest.json", "w") as f:
    json.dump(manifest, f, indent=2)
    f.write("\n")
EOF
( cd "$STAGE" && rm -f "$ROOT/dist/glass-evo-$VERSION.zip" && zip -qr "$ROOT/dist/glass-evo-$VERSION.zip" . )
cat "$STAGE/manifest.json"
ls -la "$ROOT/dist/glass-evo-$VERSION.zip"
