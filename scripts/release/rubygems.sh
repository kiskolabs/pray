#!/usr/bin/env bash
# Publish Ruby pray-cli gem to RubyGems.
#
# Usage:
#   scripts/release/rubygems.sh            # build only (default)
#   scripts/release/rubygems.sh --publish  # gem push (manual)
#
# Requires: gem credentials (~/.gem/credentials) with MFA when enabled
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"

ROOT="$(release_repo_root)"
VERSION="$(release_read_workspace_version "${ROOT}")"
MODE="dry-run"
PACKAGE_DIR="${ROOT}/rubygems/pray-cli"

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

release_require_command gem
release_require_command bundle
release_require_command ruby
release_assert_version_alignment "${ROOT}" "${VERSION}"

cd "${PACKAGE_DIR}"

echo "gem: pray-cli-${VERSION}"
echo "mode: ${MODE}"

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
  gem push "${GEM_FILE}"
fi

echo "rubygems release ${MODE} finished"
