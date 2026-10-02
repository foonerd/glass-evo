#!/bin/bash
# Assemble the component the Glass Manager installs: manifest.json, the
# binaries by architecture, the face for a browser (face/), the looks that
# ship (themes/) and the built-in look written out (face.txt), as
# dist/glass-evo-<version>.zip. The manifest names the version, the least
# Glass it works with (Cargo.toml's workspace.metadata.glass), the build
# (commit and time) and, per architecture and for the browser's module, the
# file's path and its sha256, which the Manager checks on install.
#
# And the bundle for a remote display: per architecture, the same binary
# with Glass's remote installer, as dist/glass-evo-<version>-<arch>.tar.gz.
# A remote installed from it is Glass's remote with the face in it.
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
# The face for a browser, one module for every player.
[ -f face/glass-evo-face.wasm ] || { echo "package: face/glass-evo-face.wasm is missing; run scripts/ship.sh first" >&2; exit 1; }
install -D -m 644 face/glass-evo-face.wasm "$STAGE/face/glass-evo-face.wasm"
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
with open(f"{stage}/face/glass-evo-face.wasm", "rb") as f:
    module = {"path": "face/glass-evo-face.wasm", "sha256": hashlib.sha256(f.read()).hexdigest()}
manifest = {
    "name": "glass-evo",
    "version": version,
    "requires": {"glass": glass},
    "built": {"commit": commit, "time": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())},
    "binaries": binaries,
    "face": module,
}
with open(f"{stage}/manifest.json", "w") as f:
    json.dump(manifest, f, indent=2)
    f.write("\n")
EOF
( cd "$STAGE" && rm -f "$ROOT/dist/glass-evo-$VERSION.zip" && zip -qr "$ROOT/dist/glass-evo-$VERSION.zip" . )
cat "$STAGE/manifest.json"
ls -la "$ROOT/dist/glass-evo-$VERSION.zip"

# The bundle for a remote display. The installer and its instructions are
# Glass's own, taken from the Glass this face is built on (the checkout
# cargo keeps of the tag the crates name), so there is one installer for
# both flavours of a remote and it cannot come to differ between them.
GLASS_SRC=$(cargo metadata --format-version 1 --locked | python3 -c '
import json, os, sys
packages = [p for p in json.load(sys.stdin)["packages"] if p["name"] == "glass"]
print(os.path.dirname(os.path.dirname(os.path.dirname(packages[0]["manifest_path"]))) if packages else "")')
[ -n "$GLASS_SRC" ] && [ -f "$GLASS_SRC/remote/linux/install.sh" ] || { echo "package: Glass's remote installer was not found (looked under: $GLASS_SRC)" >&2; exit 1; }
grep -q 'binary_named glass-evo' "$GLASS_SRC/remote/linux/install.sh" || { echo "package: the Glass this is built on has an installer that does not know the bundle (0.8.21 or later is needed)" >&2; exit 1; }
for arch in arm armv7 armv8 x64; do
  top="glass-evo-$VERSION-$arch"
  rm -rf "$STAGE/remote-$arch" && mkdir -p "$STAGE/remote-$arch/$top/remote"
  install -D -m 755 "bin/$arch/glass-evo" "$STAGE/remote-$arch/$top/bin/$arch/glass-evo"
  cp -r "$GLASS_SRC/remote/linux" "$STAGE/remote-$arch/$top/remote/linux"
  cp "$GLASS_SRC/remote/README.md" "$STAGE/remote-$arch/$top/remote/README.md"
  chmod +x "$STAGE/remote-$arch/$top/remote/linux/"*.sh
  tar -C "$STAGE/remote-$arch" -czf "dist/$top.tar.gz" "$top"
done
ls -la dist/glass-evo-"$VERSION"-*.tar.gz
