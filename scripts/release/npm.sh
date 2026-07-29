#!/usr/bin/env bash
# Publish TypeScript pray-cli to npmjs.
#
# Usage:
#   scripts/release/npm.sh            # pack dry-run (default)
#   scripts/release/npm.sh --publish  # real publish (manual)
#
# Requires: npm login (or NPM_TOKEN via .npmrc)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"

ROOT="$(release_repo_root)"
VERSION="$(release_read_workspace_version "${ROOT}")"
MODE="dry-run"
PACKAGE_DIR="${ROOT}/npmjs/pray-cli"

for argument in "$@"; do
  case "${argument}" in
    --publish) MODE="publish" ;;
    --dry-run) MODE="dry-run" ;;
    -h | --help)
      sed -n '2,10p' "$0"
      exit 0
      ;;
    *)
      echo "unknown argument: ${argument}" >&2
      exit 2
      ;;
  esac
done

release_require_command npm
release_require_command node
release_assert_version_alignment "${ROOT}" "${VERSION}"

cd "${PACKAGE_DIR}"

echo "npm package: pray-cli@${VERSION}"
echo "mode: ${MODE}"

npm ci
npm test
npm run lint

if [[ "${MODE}" == "dry-run" ]]; then
  echo "==> npm pack --dry-run"
  npm pack --dry-run
else
  if ! release_confirm "Publish pray-cli@${VERSION} to npmjs?"; then
    echo "skipped npm publish"
    exit 0
  fi
  echo "==> npm publish"
  npm publish
fi

echo "npm release ${MODE} finished"
