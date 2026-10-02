#!/bin/bash
# Cross-compile the face for Windows (x86_64, MinGW-w64) and lay out the
# bundle for a remote display, unsigned: dist/windows/glass-evo-<version>-
# windows-x64/ with bin/glass-evo.exe, the SDL2.dll it loads, and Glass's
# own remote installer for Windows, taken from the Glass this face is built
# on. The release signs the display and the installer scripts and zips it.
# The recipe is Glass's (scripts/ship-windows.sh there); it runs on Linux
# with mingw-w64 and the x86_64-pc-windows-gnu target, as Glass's builder
# image has them. SDL2 comes from the MinGW development package of the SDL
# project, pinned here by version and checksum.
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$ROOT"
TARGET_DIR=${CARGO_TARGET_DIR:-$ROOT/target}
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)

SDL_VERSION=2.32.10
SDL_SHA256=83a5d74012311edc3c0d40ea6faecbe57ad692aa033fa5dc273cc937e3938ff2
SDL_DIR=$TARGET_DIR/sysroot/windows/SDL2-$SDL_VERSION/x86_64-w64-mingw32
TRIPLE=x86_64-pc-windows-gnu

command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1 || {
  echo "ship-windows: x86_64-w64-mingw32-gcc is missing (apt install mingw-w64, or use Glass's builder image)" >&2
  exit 1
}

if [ ! -e "$SDL_DIR/bin/SDL2.dll" ]; then
  mkdir -p "$TARGET_DIR/sysroot/windows"
  tarball=$TARGET_DIR/sysroot/windows/SDL2-devel-$SDL_VERSION-mingw.tar.gz
  "$ROOT/scripts/fetch.sh" "https://github.com/libsdl-org/SDL/releases/download/release-$SDL_VERSION/SDL2-devel-$SDL_VERSION-mingw.tar.gz" \
    "$tarball" "$SDL_SHA256"
  tar -C "$TARGET_DIR/sysroot/windows" -xzf "$tarball"
fi

export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
# The import library of SDL2.dll for the link; the GCC runtime linked in,
# so the executable asks Windows for nothing beyond its own libraries and
# SDL2.dll beside it.
export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="-L native=$SDL_DIR/lib -C link-arg=-static-libgcc"

echo "ship-windows: glass-evo ($TRIPLE)"
cargo build --release --locked --target "$TRIPLE" -p glass-evo
install -D -m 755 "$TARGET_DIR/$TRIPLE/release/glass-evo.exe" bin/windows-x64/glass-evo.exe
install -D -m 644 "$SDL_DIR/bin/SDL2.dll" bin/windows-x64/SDL2.dll
x86_64-w64-mingw32-strip bin/windows-x64/glass-evo.exe

# What the executable loads at start: Windows' own libraries and SDL2.dll.
echo "ship-windows: imports"
imports=$(x86_64-w64-mingw32-objdump -p bin/windows-x64/glass-evo.exe | awk '/DLL Name:/ {print $3}' | sort -u)
echo "$imports" | sed 's/^/ship-windows:   /'
for dll in $imports; do
  case "$(echo "$dll" | tr '[:upper:]' '[:lower:]')" in
    sdl2.dll|kernel32.dll|user32.dll|ws2_32.dll|advapi32.dll|bcrypt.dll|bcryptprimitives.dll|ntdll.dll|msvcrt.dll|userenv.dll|shell32.dll|ole32.dll|crypt32.dll|secur32.dll|gdi32.dll|imm32.dll|winmm.dll|version.dll|setupapi.dll|oleaut32.dll|api-ms-win-*) ;;
    *) echo "ship-windows: glass-evo.exe needs $dll, which Windows does not ship" >&2; exit 1 ;;
  esac
done

# The bundle, laid out as Glass's Windows archive is: the display and
# SDL2.dll under bin, Glass's installer and its instructions under remote.
GLASS_SRC=$(cargo metadata --format-version 1 --locked | python3 -c '
import json, os, sys
packages = [p for p in json.load(sys.stdin)["packages"] if p["name"] == "glass"]
print(os.path.dirname(os.path.dirname(os.path.dirname(packages[0]["manifest_path"]))) if packages else "")')
[ -n "$GLASS_SRC" ] && [ -f "$GLASS_SRC/remote/windows/install.ps1" ] || { echo "ship-windows: Glass's Windows installer was not found (looked under: $GLASS_SRC)" >&2; exit 1; }
grep -q "glass-evo.exe" "$GLASS_SRC/remote/windows/install.ps1" || { echo "ship-windows: the Glass this is built on has an installer that does not know the bundle (0.8.23 or later is needed)" >&2; exit 1; }
TOP=glass-evo-$VERSION-windows-x64
rm -rf dist/windows && mkdir -p "dist/windows/$TOP/remote"
cp -r bin/windows-x64 "dist/windows/$TOP/bin"
cp -r "$GLASS_SRC/remote/windows" "dist/windows/$TOP/remote/windows"
cp "$GLASS_SRC/remote/README.md" "dist/windows/$TOP/remote/README.md"
chmod -R u+w "dist/windows/$TOP"
echo "ship-windows: bundle laid out, unsigned"
find "dist/windows/$TOP" -type f | sort | sed 's/^/ship-windows:   /'
