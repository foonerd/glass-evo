#!/bin/bash
# Build the face for each Volumio architecture and copy it into
# bin/<arch>/glass-evo, and for a browser into face/glass-evo-face.wasm.
# Players run those files; they do not run cargo. The
# recipe is Glass's: the libraries the display links against (SDL2, ALSA)
# come from Debian's own packages into a sysroot per architecture, so the
# binaries need nothing newer than Volumio's glibc 2.36.
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$ROOT"
unset CARGO_TARGET_DIR

# A library from Debian bookworm into the sysroot, held against the digest
# Debian's index gives for it, and a link the linker finds under its plain
# name. Looked for with -e, which follows links: a
# restored CI cache keeps the links and drops the files behind them, and
# such a library counts as missing and is unpacked again.
fetch() {
  local deb_arch=$1 multiarch=$2 url=$3 sha=$4 soname=$5 plain=$6
  local deb="target/sysroot/$(basename "$url")"
  local so="target/sysroot/$deb_arch/usr/lib/$multiarch/$soname"
  if [ ! -e "$so" ]; then
    mkdir -p "target/sysroot/$deb_arch"
    "$ROOT/scripts/fetch.sh" "$url" "$deb" "$sha"
    dpkg-deb -x "$deb" "target/sysroot/$deb_arch"
  fi
  mkdir -p "target/sysroot/link-$deb_arch"
  ln -sf "$ROOT/$so" "target/sysroot/link-$deb_arch/$plain"
}
# Per architecture: its name in Debian, its library directory, the digests
# of its SDL2 and its ALSA package.
for row in \
  "armhf arm-linux-gnueabihf b76abdb214e7213a1ea6542dd618c940d3b5601c14dcf88577db1d13d6a80ec7 97718ff5e552c9a7b9e77e8debbe219b0068a9ab5dcabedd0800c69dd8f0eb96" \
  "arm64 aarch64-linux-gnu 7d875b119108a240015e7739c7a0de402616975281bcd8169cb434326fa93a66 9fa889400fcee4b92c8f4a2fafbb7f2cd33444d9ec1665a71002ab67c06114bb"; do
  set -- $row
  fetch "$1" "$2" "http://deb.debian.org/debian/pool/main/libs/libsdl2/libsdl2-2.0-0_2.26.5+dfsg-1_$1.deb" "$3" libSDL2-2.0.so.0 libSDL2.so
  fetch "$1" "$2" "http://deb.debian.org/debian/pool/main/a/alsa-lib/libasound2_1.2.8-1+b1_$1.deb" "$4" libasound.so.2 libasound.so
done
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
export CARGO_TARGET_ARMV7_UNKNOWN_LINUX_GNUEABIHF_RUSTFLAGS="-L native=$ROOT/target/sysroot/link-armhf -C link-arg=-Wl,--allow-shlib-undefined"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS="-L native=$ROOT/target/sysroot/link-arm64 -C link-arg=-Wl,--allow-shlib-undefined"

ship() {
  local triple=$1 arch=$2 stripper=$3
  echo "ship: $arch ($triple)"
  cargo build --release --locked --target "$triple" -p glass-evo
  install -D -m 755 "target/$triple/release/glass-evo" "bin/$arch/glass-evo"
  "$stripper" "bin/$arch/glass-evo"
}
ship x86_64-unknown-linux-gnu x64 strip
ship armv7-unknown-linux-gnueabihf armv7 arm-linux-gnueabihf-strip
install -D -m 755 bin/armv7/glass-evo bin/arm/glass-evo
ship aarch64-unknown-linux-gnu armv8 aarch64-linux-gnu-strip

# The face for a browser: Glass's pipeline for a page with the face over
# it, one file the Glass Manager serves to its Face tab and to Anymote in
# the place of Glass's own module.
echo "ship: browser module"
cargo build --locked --profile face --target wasm32-unknown-unknown -p glass-evo-face
install -D -m 644 target/wasm32-unknown-unknown/face/glass_evo_face.wasm face/glass-evo-face.wasm

echo "ship: glibc"
for bin in bin/*/glass-evo; do
  version=$(readelf -W --dyn-syms "$bin" | grep -o 'GLIBC_2\.[0-9]*' | sort -t. -k2,2n -u | tail -n1)
  case "$version" in
    GLIBC_2.3[7-9]|GLIBC_2.[4-9]*|GLIBC_2.[1-9][0-9][0-9])
      echo "ship: $bin needs $version, newer than Volumio's glibc 2.36" >&2
      exit 1
      ;;
  esac
  echo "ship: $bin needs at most ${version:-no versioned glibc symbol}"
done
file bin/arm/glass-evo bin/armv7/glass-evo bin/armv8/glass-evo bin/x64/glass-evo
