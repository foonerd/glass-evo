#!/bin/bash
# The release's assets signed, twice over:
#   1. Inside every archive (a zip or a tar.gz, the apk left to Android's
#      own signing): MANIFEST, every file's SHA-256 and path from the
#      archive's root, and MANIFEST.sig, its Ed25519 signature. A player or
#      a remote given the archive by hand verifies it with no network.
#   2. Beside the assets: DIST/SHA256SUMS, every file's SHA-256, and
#      DIST/SHA256SUMS.sig, for the online path and for a check by hand.
# Every signature is checked against PUB, the public key the products
# carry, so no release goes out that a player would refuse.
#   sign-release.sh DIST PUB
# The private key comes as PEM in RELEASE_SIGNING_KEY (the repository's
# secret). Directories in DIST and the notes are left out of the sums.
set -euo pipefail
dist=$(cd "$1" && pwd)
pub=$(cd "$(dirname "$2")" && pwd)/$(basename "$2")
[ -n "${RELEASE_SIGNING_KEY:-}" ] || { echo "sign-release: RELEASE_SIGNING_KEY is empty" >&2; exit 2; }
[ -f "$pub" ] || { echo "sign-release: no public key at $pub" >&2; exit 2; }
key=$(mktemp)
work=$(mktemp -d)
trap 'rm -rf "$key" "$work"' EXIT
printf '%s\n' "$RELEASE_SIGNING_KEY" > "$key"

# MANIFEST and MANIFEST.sig written at the root of the unpacked tree in $1.
manifest() {
  (
    cd "$1"
    rm -f MANIFEST MANIFEST.sig
    # Into a temporary name first: the manifest must not list itself.
    find . -type f ! -name MANIFEST ! -name MANIFEST.sig ! -name .manifest.part -printf '%P\n' | LC_ALL=C sort | tr '\n' '\0' | xargs -0 sha256sum > .manifest.part
    mv .manifest.part MANIFEST
    openssl pkeyutl -sign -inkey "$key" -rawin -in MANIFEST -out MANIFEST.sig
    openssl pkeyutl -verify -pubin -inkey "$pub" -rawin -in MANIFEST -sigfile MANIFEST.sig > /dev/null
  )
}

signed=0
for archive in "$dist"/*.zip "$dist"/*.tar.gz; do
  [ -f "$archive" ] || continue
  rm -rf "$work/tree" && mkdir -p "$work/tree"
  case "$archive" in
    *.zip)
      unzip -q "$archive" -d "$work/tree"
      manifest "$work/tree"
      (cd "$work/tree" && zip -q "$archive" MANIFEST MANIFEST.sig)
      ;;
    *.tar.gz)
      tar -C "$work/tree" -xzf "$archive"
      manifest "$work/tree"
      # The tree packed again with the two files at its root; owners and
      # order fixed, so the archive is the same for the same tree.
      (cd "$work/tree" && tar --sort=name --owner=0 --group=0 --numeric-owner --mtime='@0' -czf "$archive.new" -- *)
      mv "$archive.new" "$archive"
      ;;
  esac
  signed=$((signed + 1))
  echo "sign-release: $(basename "$archive") carries MANIFEST ($(wc -l < "$work/tree/MANIFEST") files) and its signature"
done

(
  cd "$dist"
  rm -f SHA256SUMS SHA256SUMS.sig
  files=$(find . -maxdepth 1 -type f ! -name 'SHA256SUMS*' ! -name 'notes.md' -printf '%f\n' | LC_ALL=C sort)
  [ -n "$files" ] || { echo "sign-release: nothing to sign in $dist" >&2; exit 2; }
  # shellcheck disable=SC2086
  sha256sum $files > SHA256SUMS
)
openssl pkeyutl -sign -inkey "$key" -rawin -in "$dist/SHA256SUMS" -out "$dist/SHA256SUMS.sig"
openssl pkeyutl -verify -pubin -inkey "$pub" -rawin -in "$dist/SHA256SUMS" -sigfile "$dist/SHA256SUMS.sig"
echo "sign-release: $signed archives signed inside, $(wc -l < "$dist/SHA256SUMS") assets in the sums, every signature verified against $pub"
cat "$dist/SHA256SUMS"
