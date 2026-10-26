#!/usr/bin/env bash
# Publish TypeScript pray-core library to jsr.io as @kiskolabs/pray-core.
#
# Usage:
#   scripts/release/jsr.sh            # publish --dry-run (default)
#   scripts/release/jsr.sh --publish  # real publish (manual)
#
# Prerequisites: @kiskolabs scope and pray-core package on jsr.io.
# Auth: browser login for local publish, or JSR_TOKEN / --token in CI.
# Publish skips when @kiskolabs/pray-core@version is already on jsr.io so release-all can resume.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"
# shellcheck source=registry_status.sh
source "${SCRIPT_DIR}/registry_status.sh"

ROOT="$(release_repo_root)"
VERSION="$(release_read_npm_version "${ROOT}")"
MODE="dry-run"
PACKAGE_DIR="${ROOT}/npmjs/pray-core"
JSR_PACKAGE="@kiskolabs/pray-core"

for argument in "$@"; do
  case "${argument}" in
    --publish) MODE="publish" ;;
    --dry-run) MODE="dry-run" ;;
    -h | --help)
      sed -n '2,13p' "$0"
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
release_require_command curl
release_assert_npm_typescript_match "${ROOT}"
release_assert_jsr_match "${ROOT}"
release_begin_surface_publish "${ROOT}" "npm"

cd "${PACKAGE_DIR}"

echo "jsr package: ${JSR_PACKAGE}@${VERSION}"
echo "mode: ${MODE}"

if [[ "${MODE}" == "publish" ]] && release_jsr_has_version "${JSR_PACKAGE}" "${VERSION}"; then
  echo "skip ${JSR_PACKAGE}@${VERSION} (already on jsr.io)"
  echo "jsr release ${MODE} finished"
  exit 0
fi

npm ci

PUBLISH_ARGS=(publish)
# Allow dirty trees: release-all often resumes with release-script or docs edits
# that are not part of the published TypeScript sources.
PUBLISH_ARGS+=(--allow-dirty)
if [[ "${MODE}" == "dry-run" ]]; then
  PUBLISH_ARGS+=(--dry-run)
elif ! release_confirm "Publish ${JSR_PACKAGE}@${VERSION} to jsr.io?"; then
  echo "skipped jsr publish"
  exit 0
fi

if [[ -n "${JSR_TOKEN:-}" ]]; then
  PUBLISH_ARGS+=(--token "${JSR_TOKEN}")
fi

echo "==> npx jsr ${PUBLISH_ARGS[*]}"
if [[ "${MODE}" == "dry-run" ]]; then
  npx --yes jsr "${PUBLISH_ARGS[@]}"
else
  publish_log="$(mktemp)"
  set +e
  npx --yes jsr "${PUBLISH_ARGS[@]}" 2>&1 | tee "${publish_log}"
  publish_status="${PIPESTATUS[0]}"
  set -e
  outcome="$(
    release_publish_outcome no "${publish_status}" release_jsr_already_exists \
      "$(cat "${publish_log}")" || true
  )"
  rm -f "${publish_log}"
  if [[ "${outcome}" == "skip" ]]; then
    echo "skip ${JSR_PACKAGE}@${VERSION} (already on jsr.io)"
  elif [[ "${publish_status}" -ne 0 ]]; then
    exit "${publish_status}"
  fi
fi

echo "jsr release ${MODE} finished"
