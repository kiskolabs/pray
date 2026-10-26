#!/usr/bin/env bash
# Publish Ruby pray-cli gem to RubyGems.
#
# Usage:
#   scripts/release/rubygems.sh            # build only (default)
#   scripts/release/rubygems.sh --publish  # gem push (manual)
#
# Requires: gem credentials (~/.gem/credentials) with MFA when enabled
# Publish skips when pray-cli@version is already on RubyGems so release-all can resume.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"
# shellcheck source=registry_status.sh
source "${SCRIPT_DIR}/registry_status.sh"

ROOT="$(release_repo_root)"
MODE="dry-run"
PACKAGE_DIR="${ROOT}/rubygems/pray-cli"

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

release_require_command gem
release_require_command bundle
release_require_command ruby
release_require_command curl
release_configure_ruby_environment "${PACKAGE_DIR}"
VERSION="$(release_read_gem_version "${ROOT}")"
release_begin_surface_publish "${ROOT}" "gem"

cd "${PACKAGE_DIR}"

echo "gem: pray-cli-${VERSION}"
echo "mode: ${MODE}"

if [[ "${MODE}" == "publish" ]] && release_rubygems_has_version "pray-cli" "${VERSION}"; then
  echo "skip pray-cli@${VERSION} (already on RubyGems)"
  echo "rubygems release ${MODE} finished"
  exit 0
fi

bundle install
bundle exec rake lint
bundle exec rspec
bundle exec rake build

GEM_FILE="pkg/pray-cli-${VERSION}.gem"
if [[ ! -f "${GEM_FILE}" ]]; then
  echo "expected gem artifact missing: ${GEM_FILE}" >&2
  exit 2
fi

if [[ "${MODE}" == "dry-run" ]]; then
  echo "==> built ${GEM_FILE} (no push)"
  gem specification "${GEM_FILE}" name version summary
else
  if ! release_confirm "Push ${GEM_FILE} to RubyGems?"; then
    echo "skipped gem push"
    exit 0
  fi
  echo "==> gem push ${GEM_FILE}"
  publish_log="$(mktemp)"
  set +e
  gem push "${GEM_FILE}" 2>&1 | tee "${publish_log}"
  publish_status="${PIPESTATUS[0]}"
  set -e
  outcome="$(
    release_publish_outcome no "${publish_status}" release_rubygems_already_exists \
      "$(cat "${publish_log}")" || true
  )"
  rm -f "${publish_log}"
  if [[ "${outcome}" == "skip" ]]; then
    echo "skip pray-cli@${VERSION} (already on RubyGems)"
  elif [[ "${publish_status}" -ne 0 ]]; then
    exit "${publish_status}"
  fi
fi

echo "rubygems release ${MODE} finished"
