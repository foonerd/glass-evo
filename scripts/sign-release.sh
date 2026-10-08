#!/bin/bash
# The release's assets signed: every file in DIST named with its SHA-256 in
# DIST/SHA256SUMS, the file signed with the project's Ed25519 release key
# into DIST/SHA256SUMS.sig, and the signature checked against PUB, the
# public key the products carry, so no release goes out that a player
# would refuse.
#   sign-release.sh DIST PUB
# The private key comes as PEM in RELEASE_SIGNING_KEY (the repository's
# secret). Directories in DIST and the notes are left out of the sums.
set -euo pipefail
dist=$1
pub=$2
[ -n "${RELEASE_SIGNING_KEY:-}" ] || { echo "sign-release: RELEASE_SIGNING_KEY is empty" >&2; exit 2; }
[ -f "$pub" ] || { echo "sign-release: no public key at $pub" >&2; exit 2; }
key=$(mktemp)
trap 'rm -f "$key"' EXIT
printf '%s\n' "$RELEASE_SIGNING_KEY" > "$key"
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
echo "sign-release: signed $(wc -l < "$dist/SHA256SUMS") assets, the signature verified against $pub"
cat "$dist/SHA256SUMS"
