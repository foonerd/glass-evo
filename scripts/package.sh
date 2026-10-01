#!/bin/bash
# Assemble the component the Glass Manager installs: manifest.json, the
# binaries by architecture, the looks that ship (themes/) and the built-in
# look written out (face.txt), as dist/glass-evo-<version>.zip. The manifest
# names the version, the least Glass it works with (Cargo.toml's
# workspace.metadata.glass), the build (commit and time) and, per
# architecture, the binary's path and its sha256, which the Manager checks
# on install.
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$ROOT"
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)
GLASS=$(sed -n '/^\[workspace\.metadata\.glass\]/,/^\[/s/^plugin = "\(.*\)"/\1/p' Cargo.toml)
[ -n "$GLASS" ] || { echo "package: Cargo.toml names no least Glass (workspace.metadata.glass.plugin)" >&2; exit 1; }
STAGE=$(mktemp -d)
trap 'rm -rf "$STAGE"' EXIT
mkdir -p "$STAGE" dist
for arch in arm armv7 armv8 x64; do
  bin="bin/$arch/glass-evo"
  [ -x "$bin" ] || { echo "package: $bin is missing; run scripts/ship.sh first" >&2; exit 1; }
  install -D -m 755 "$bin" "$STAGE/bin/$arch/glass-evo"
done
# The looks that ship: every face theme but the example, which documents
# the format and is the built-in look written out. That one goes beside the
# manifest, for the Manager to know what a face shows when nothing is said.
for theme in themes/*/; do
  name=$(basename "$theme")
  [ "$name" = "Example" ] && continue
  install -D -m 644 "$theme/face.txt" "$STAGE/themes/$name/face.txt"
done
install -D -m 644 themes/Example/face.txt "$STAGE/face.txt"
COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo unknown)
python3 - "$STAGE" "$VERSION" "$COMMIT" "$GLASS" <<'EOF'
import hashlib, json, sys, time
stage, version, commit, glass = sys.argv[1:5]
binaries = {}
for arch in ("arm", "armv7", "armv8", "x64"):
    path = f"bin/{arch}/glass-evo"
    with open(f"{stage}/{path}", "rb") as f:
        binaries[arch] = {"path": path, "sha256": hashlib.sha256(f.read()).hexdigest()}
manifest = {
    "name": "glass-evo",
    "version": version,
    "requires": {"glass": glass},
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
