#!/usr/bin/env bash
# Bump packaging/aur/pray-cli/PKGBUILD to the workspace version and refresh sha256.
#
# Requires the GitHub tag vX.Y.Z to exist so the source archive is fetchable.
#
# Usage:
#   scripts/release/aur_bump.sh
#   scripts/release/aur_bump.sh --version 1.23.0
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"

ROOT="$(release_repo_root)"
VERSION="$(release_read_workspace_version "${ROOT}")"
PKGBUILD="${ROOT}/packaging/aur/pray-cli/PKGBUILD"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      VERSION="$2"
      shift 2
      ;;
    -h | --help)
      sed -n '2,12p' "$0"
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

release_require_command curl
release_require_command shasum

ARCHIVE_URL="https://github.com/kiskolabs/pray/archive/refs/tags/v${VERSION}.tar.gz"
TMP="$(mktemp)"
trap 'rm -f "${TMP}"' EXIT

echo "fetching ${ARCHIVE_URL}"
curl -fsSL "${ARCHIVE_URL}" -o "${TMP}"
SUM="$(shasum -a 256 "${TMP}" | awk '{print $1}')"

# Portable in-place edit without relying on GNU sed -i.
python3 - "${PKGBUILD}" "${VERSION}" "${SUM}" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
version = sys.argv[2]
sha = sys.argv[3]
text = path.read_text()
text = re.sub(r"^pkgver=.*$", f"pkgver={version}", text, count=1, flags=re.M)
text = re.sub(r"^pkgrel=.*$", "pkgrel=1", text, count=1, flags=re.M)
text = re.sub(r"^sha256sums=\(.*\)$", f"sha256sums=('{sha}')", text, count=1, flags=re.M)
path.write_text(text)
PY

echo "updated ${PKGBUILD} to ${VERSION} sha256=${SUM}"
echo "copy this tree into the AUR git package when ready to publish"
