#!/usr/bin/env bash
# Publish TypeScript pray-cli to npmjs.
#
# Usage:
#   scripts/release/npm.sh            # pack dry-run (default)
#   scripts/release/npm.sh --publish  # real publish (manual)
#
# Requires: npm login (or NPM_TOKEN via .npmrc)
# Publish skips when pray-cli@version is already on npmjs so release-all can resume.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"
# shellcheck source=registry_status.sh
source "${SCRIPT_DIR}/registry_status.sh"

ROOT="$(release_repo_root)"
VERSION="$(release_read_npm_version "${ROOT}")"
MODE="dry-run"
PACKAGE_DIR="${ROOT}/npmjs/pray-cli"

for argument in "$@"; do
  case "${argument}" in
    --publish) MODE="publish" ;;
    --dry-run) MODE="dry-run" ;;
    -h | --help)
      sed -n '2,11p' "$0"
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
release_require_command curl
release_assert_npm_typescript_match "${ROOT}"
release_begin_surface_publish "${ROOT}" "npm"

cd "${PACKAGE_DIR}"

echo "npm package: pray-cli@${VERSION}"
echo "mode: ${MODE}"

if [[ "${MODE}" == "publish" ]] && release_npm_has_version "pray-cli" "${VERSION}"; then
  echo "skip pray-cli@${VERSION} (already on npmjs)"
  echo "npm release ${MODE} finished"
  exit 0
fi

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
  publish_log="$(mktemp)"
  set +e
  npm publish 2>&1 | tee "${publish_log}"
  publish_status="${PIPESTATUS[0]}"
  set -e
  outcome="$(
    release_publish_outcome no "${publish_status}" release_npm_already_exists \
      "$(cat "${publish_log}")" || true
  )"
  rm -f "${publish_log}"
  if [[ "${outcome}" == "skip" ]]; then
    echo "skip pray-cli@${VERSION} (already on npmjs)"
  elif [[ "${publish_status}" -ne 0 ]]; then
    exit "${publish_status}"
  fi
fi

echo "npm release ${MODE} finished"
