#!/bin/bash
# Sign the bundle for a Windows remote display with Azure Trusted Signing,
# from Linux, through jsign, as Glass signs its own: glass-evo.exe gets an
# Authenticode signature and a timestamp, the two installer scripts a
# signature block. Windows then knows who published them: SmartScreen
# stops asking, and Smart App Control lets them run. What is signed is the
# bundle scripts/ship-windows.sh laid out under dist/windows.
#
# Needs an Azure login (az) with the Trusted Signing Certificate Profile
# Signer role on the account, and three values:
#   TRUSTED_SIGNING_ENDPOINT   the account's endpoint
#   TRUSTED_SIGNING_ACCOUNT    the Trusted Signing account
#   TRUSTED_SIGNING_PROFILE    its certificate profile (Public Trust)
# The release workflow runs this when the repository has those values.

set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)
cd "$ROOT"
: "${TRUSTED_SIGNING_ENDPOINT:?}" "${TRUSTED_SIGNING_ACCOUNT:?}" "${TRUSTED_SIGNING_PROFILE:?}"

# The two tools are small; the package lists are fetched only when the
# machine's own no longer name them.
apt_install() {
  sudo apt-get install -y --no-install-recommends "$@" \
    || { sudo apt-get update && sudo apt-get install -y --no-install-recommends "$@"; }
}
JSIGN_VERSION=7.5
JSIGN_SHA256=7b4a01ba81e9ee866f09a5e45d40c928707eb5286f4e20d4042c67a141ae5e62
if ! command -v jsign >/dev/null 2>&1; then
  "$ROOT/scripts/fetch.sh" "https://github.com/ebourg/jsign/releases/download/$JSIGN_VERSION/jsign_${JSIGN_VERSION}_all.deb" \
    /tmp/jsign.deb "$JSIGN_SHA256"
  apt_install /tmp/jsign.deb osslsigncode
fi
command -v osslsigncode >/dev/null 2>&1 || apt_install osslsigncode

token=$(az account get-access-token --resource https://codesigning.azure.net --query accessToken -o tsv)
TOP=$(ls dist/windows | head -n1)
[ -n "$TOP" ] && [ -f "dist/windows/$TOP/bin/glass-evo.exe" ] || { echo "sign-windows: no bundle laid out under dist/windows (scripts/ship-windows.sh makes it)" >&2; exit 1; }
EXE=dist/windows/$TOP/bin/glass-evo.exe
for file in "$EXE" "dist/windows/$TOP/remote/windows/install.ps1" "dist/windows/$TOP/remote/windows/uninstall.ps1"; do
  echo "sign-windows: $file"
  jsign --storetype TRUSTEDSIGNING \
    --keystore "${TRUSTED_SIGNING_ENDPOINT#https://}" \
    --storepass "$token" \
    --alias "$TRUSTED_SIGNING_ACCOUNT/$TRUSTED_SIGNING_PROFILE" \
    --tsaurl http://timestamp.acs.microsoft.com,http://timestamp.digicert.com --tsmode RFC3161 \
    --tsretries 5 --tsretrywait 15 \
    --name "Glass Remote" --url https://github.com/foonerd/glass-evo \
    "$file"
done
# The runner has no Microsoft root certificates, so osslsigncode cannot
# walk the chain and calls the verification failed; Windows walks it.
# What is checked here is what the runner can see: a signature issued by
# Microsoft's public code signing CA, and a timestamp.
echo "sign-windows: verify"
report=$(osslsigncode verify "$EXE" 2>&1 || true)
echo "$report" | grep -E "Subject:|verification" | sed 's/^/sign-windows:   /'
echo "$report" | grep -q "Microsoft ID Verified Code Signing PCA" \
  || { echo "sign-windows: glass-evo.exe carries no signature under Microsoft's public code signing CA" >&2; exit 1; }
echo "$report" | grep -q "Timestamping CA" \
  || { echo "sign-windows: glass-evo.exe carries no timestamp" >&2; echo "$report" | sed 's/^/sign-windows:   /' >&2; exit 1; }
echo "sign-windows: signed and timestamped"
