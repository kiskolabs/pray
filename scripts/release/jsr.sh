#!/usr/bin/env bash
# Publish TypeScript pray-core then pray-cli to jsr.io.
#
# Usage:
#   scripts/release/jsr.sh            # publish --dry-run (default)
#   scripts/release/jsr.sh --publish  # real publish (manual)
#
# Prerequisites: @kiskolabs scope; packages pray-core and pray-cli on jsr.io.
# Auth: browser login for local publish, or JSR_TOKEN / --token in CI.
# Order: @kiskolabs/pray-core, then @kiskolabs/pray-cli (CLI depends on core).
# Publish skips a package whose version is already on jsr.io so release-all can resume.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"
# shellcheck source=registry_status.sh
source "${SCRIPT_DIR}/registry_status.sh"

ROOT="$(release_repo_root)"
VERSION="$(release_read_npm_version "${ROOT}")"
MODE="dry-run"
NPMJS_ROOT="${ROOT}/npmjs"
CORE_DIR="${NPMJS_ROOT}/pray-core"
CLI_DIR="${NPMJS_ROOT}/pray-cli"

for argument in "$@"; do
  case "${argument}" in
    --publish) MODE="publish" ;;
    --dry-run) MODE="dry-run" ;;
    -h | --help)
      sed -n '2,14p' "$0"
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

echo "jsr packages: @kiskolabs/pray-core@${VERSION} then @kiskolabs/pray-cli@${VERSION}"
echo "mode: ${MODE}"

cd "${NPMJS_ROOT}"
npm ci

publish_one() {
  local jsr_name="$1"
  local directory="$2"
  echo "jsr package: ${jsr_name}@${VERSION}"
  if [[ "${MODE}" == "publish" ]] && release_jsr_has_version "${jsr_name}" "${VERSION}"; then
    echo "skip ${jsr_name}@${VERSION} (already on jsr.io)"
    return 0
  fi
  (
    cd "${directory}"
    PUBLISH_ARGS=(publish)
    # Allow dirty trees: release-all often resumes with release-script or docs edits
    # that are not part of the published TypeScript sources.
    PUBLISH_ARGS+=(--allow-dirty)
    if [[ "${MODE}" == "dry-run" ]]; then
      PUBLISH_ARGS+=(--dry-run)
    elif ! release_confirm "Publish ${jsr_name}@${VERSION} to jsr.io?"; then
      echo "skipped ${jsr_name} publish"
      return 0
    fi

    if [[ -n "${JSR_TOKEN:-}" ]]; then
      PUBLISH_ARGS+=(--token "${JSR_TOKEN}")
    fi

    echo "==> npx jsr ${PUBLISH_ARGS[*]} (${jsr_name})"
    if [[ "${MODE}" == "dry-run" ]]; then
      npx --yes jsr "${PUBLISH_ARGS[@]}"
      return 0
    fi
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
      echo "skip ${jsr_name}@${VERSION} (already on jsr.io)"
    elif [[ "${publish_status}" -ne 0 ]]; then
      exit "${publish_status}"
    fi
  )
}

publish_one "@kiskolabs/pray-core" "${CORE_DIR}"
publish_one "@kiskolabs/pray-cli" "${CLI_DIR}"

echo "jsr release ${MODE} finished"
