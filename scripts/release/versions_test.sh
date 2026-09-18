#!/usr/bin/env bash
# Prove coordinated version math for crates, npm, and gem one-surface cuts.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

[[ "$(release_version_max "1.18.0" "1.18.1")" == "1.18.1" ]] || fail "max of 1.18.0 and 1.18.1"
[[ "$(release_bump_patch "1.18.1")" == "1.18.2" ]] || fail "patch bump of 1.18.1"
[[ "$(release_bump_patch "1.19.0")" == "1.19.1" ]] || fail "patch bump of minor 1.19.0"

[[ "$(release_next_coordinated_from "1.18.0" "1.18.0" "1.18.1")" == "1.18.2" ]] ||
  fail "gem-only 1.18.1 skipped; next is 1.18.2"
[[ "$(release_next_coordinated_from "1.18.0" "1.19.0" "1.18.1")" == "1.19.1" ]] ||
  fail "npm-only 1.19.0 skipped; next is 1.19.1"
[[ "$(release_next_coordinated_from "1.19.0" "1.18.0" "1.18.1")" == "1.19.1" ]] ||
  fail "crates-only 1.19.0 skipped; next is 1.19.1"
[[ "$(release_next_coordinated_from "1.18.2" "1.18.2" "1.18.2")" == "1.18.2" ]] ||
  fail "aligned 1.18.2"
[[ "$(release_next_coordinated_from "1.19.0" "1.19.0" "1.18.1")" == "1.19.0" ]] ||
  fail "two surfaces at 1.19.0; gem catches up"

[[ "$(release_publish_kind "1.18.0" "1.18.0" "1.18.1")" == "gem" ]] || fail "kind gem"
[[ "$(release_publish_kind "1.18.0" "1.19.0" "1.18.1")" == "npm" ]] || fail "kind npm"
[[ "$(release_publish_kind "1.19.0" "1.18.0" "1.18.1")" == "crates" ]] || fail "kind crates"
[[ "$(release_publish_kind "1.18.2" "1.18.2" "1.18.2")" == "aligned" ]] || fail "kind aligned"
[[ "$(release_publish_kind "1.19.0" "1.19.0" "1.18.1")" == "drift" ]] || fail "kind drift"

ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
release_assert_coordinated_changelog "${ROOT}" "1.18.0" || fail "1.18.0 is coordinated"
release_assert_coordinated_changelog "${ROOT}" "1.19.0" || fail "1.19.0 is coordinated"
if release_assert_coordinated_changelog "${ROOT}" "1.18.1" >/dev/null 2>&1; then
  fail "1.18.1 must not be coordinated"
fi
[[ "$(release_next_coordinated_version "${ROOT}")" == "1.19.0" ]] ||
  fail "live next coordinated version is 1.19.0"

echo "coordinated version tests passed"
