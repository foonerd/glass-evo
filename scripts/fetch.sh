#!/bin/bash
# One download for the build scripts: fetch.sh URL FILE [SHA256].
# A connection that fails or stalls is tried again. The file appears under
# its name only when it is whole, and, with a digest given, only when it is
# the file expected.
set -euo pipefail
url=$1
file=$2
sha=${3:-}
part=$file.part
mkdir -p "$(dirname "$file")"
trap 'rm -f "$part"' EXIT
curl -fL --no-progress-meter --retry 5 --retry-delay 3 --retry-all-errors \
  --connect-timeout 20 --speed-limit 1024 --speed-time 30 -o "$part" "$url"
if [ -n "$sha" ]; then
  got=$(sha256sum "$part" | cut -d' ' -f1)
  [ "$got" = "$sha" ] || { echo "fetch: $url has sha256 $got, expected $sha" >&2; exit 1; }
fi
mv "$part" "$file"
