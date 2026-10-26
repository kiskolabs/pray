#!/usr/bin/env bash
# Publish TypeScript pray-core then pray-cli to npmjs.
#
# Usage:
#   scripts/release/npm.sh            # pack dry-run (default)
#   scripts/release/npm.sh --publish  # real publish (manual)
#
# Requires: npm login (or NPM_TOKEN via .npmrc).
# For accounts with publish 2FA, set NPM_OTP to an authenticator code for the run.
# Do not pipe npm publish: that made 2FA fail with EOTP under make release-all.
# Already-published races are detected with a registry version check after failure.
# Publish skips a package whose version is already on npmjs so release-all can resume.
# Order: pray-core, then pray-cli (CLI depends on core).
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
release_require_command node
release_require_command curl
release_assert_npm_typescript_match "${ROOT}"
release_begin_surface_publish "${ROOT}" "npm"

echo "npm packages: pray-core@${VERSION} then pray-cli@${VERSION}"
echo "mode: ${MODE}"

cd "${NPMJS_ROOT}"
npm ci

publish_one() {
  local name="$1"
  local directory="$2"
  echo "npm package: ${name}@${VERSION}"
  if [[ "${MODE}" == "publish" ]] && release_npm_has_version "${name}" "${VERSION}"; then
    echo "skip ${name}@${VERSION} (already on npmjs)"
    return 0
  fi
  (
    cd "${directory}"
    npm test
    npm run lint
    if [[ "${MODE}" == "dry-run" ]]; then
      echo "==> npm pack --dry-run (${name})"
      npm pack --dry-run
      return 0
    fi
    if ! release_confirm "Publish ${name}@${VERSION} to npmjs?"; then
      echo "skipped ${name} publish"
      return 0
    fi
    publish_args=(publish)
    if [[ -n "${NPM_OTP:-}" ]]; then
      publish_args+=(--otp "${NPM_OTP}")
    fi
    echo "==> npm ${publish_args[*]} (${name})"
    set +e
    # Keep the real terminal so browser/device 2FA can finish (no pipes).
    npm "${publish_args[@]}"
    publish_status=$?
    set -e
    if [[ "${publish_status}" -eq 0 ]]; then
      :
    elif release_npm_wait_for_version "${name}" "${VERSION}"; then
      # E409 "previously staged" can land before GET /version is 200.
      echo "skip ${name}@${VERSION} (already on npmjs)"
    else
      exit "${publish_status}"
    fi
  )
}

publish_one "pray-core" "${CORE_DIR}"
publish_one "pray-cli" "${CLI_DIR}"

echo "npm release ${MODE} finished"
