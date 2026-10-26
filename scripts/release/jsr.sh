#!/usr/bin/env bash
# Publish TypeScript pray-cli library to jsr.io as @kiskolabs/pray-cli.
#
# Usage:
#   scripts/release/jsr.sh            # publish --dry-run (default)
#   scripts/release/jsr.sh --publish  # real publish (manual)
#
# Prerequisites: @kiskolabs scope and pray-cli package on jsr.io.
# Auth: browser login for local publish, or JSR_TOKEN / --token in CI.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"

ROOT="$(release_repo_root)"
VERSION="$(release_read_npm_version "${ROOT}")"
MODE="dry-run"
PACKAGE_DIR="${ROOT}/npmjs/pray-cli"

for argument in "$@"; do
  case "${argument}" in
    --publish) MODE="publish" ;;
    --dry-run) MODE="dry-run" ;;
    -h | --help)
      sed -n '2,12p' "$0"
      exit 0
      ;;
    *)
      echo "unknown argument: ${argument}" >&2
      exit 2
      ;;
  esac
done

release_require_command npm
release_require_command npx
release_assert_npm_typescript_match "${ROOT}"
release_assert_jsr_match "${ROOT}"
release_begin_surface_publish "${ROOT}" "npm"

cd "${PACKAGE_DIR}"

echo "jsr package: @kiskolabs/pray-cli@${VERSION}"
echo "mode: ${MODE}"

npm ci

PUBLISH_ARGS=(publish)
if [[ "${MODE}" == "dry-run" ]]; then
  # Dry-run may run on a dirty tree while preparing a cut.
  PUBLISH_ARGS+=(--dry-run --allow-dirty)
elif ! release_confirm "Publish @kiskolabs/pray-cli@${VERSION} to jsr.io?"; then
  echo "skipped jsr publish"
  exit 0
fi

if [[ -n "${JSR_TOKEN:-}" ]]; then
  PUBLISH_ARGS+=(--token "${JSR_TOKEN}")
fi

# Slow types may remain until export APIs are annotated for JSR; allow until fixed.
PUBLISH_ARGS+=(--allow-slow-types)

echo "==> npx jsr ${PUBLISH_ARGS[*]}"
npx --yes jsr "${PUBLISH_ARGS[@]}"

echo "jsr release ${MODE} finished"
