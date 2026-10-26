#!/usr/bin/env bash
# Prove registry skip treats an already-published version as continue, not fail.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=registry_status.sh
source "${SCRIPT_DIR}/registry_status.sh"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

release_http_ok "200" || fail "HTTP 200 must count as published"
release_http_ok "404" && fail "HTTP 404 must not count as published"
release_http_ok "000" && fail "curl failure status must not count as published"

npm_exists='npm error code E403
npm error 403 403 Forbidden - PUT https://registry.npmjs.org/pray-cli - You cannot publish over the previously published versions: 1.23.0.'
release_npm_already_exists "${npm_exists}" || fail "npm already-published error must skip"

npm_conflict='npm error code EPUBLISHCONFLICT
npm error EPUBLISHCONFLICT Unable to publish overlapping ranges.'
release_npm_already_exists "${npm_conflict}" || fail "npm EPUBLISHCONFLICT must skip"

npm_other='npm error code EOTP
npm error This operation requires a one-time password'
release_npm_already_exists "${npm_other}" && fail "unrelated npm error must not skip"

gem_exists='ERROR:  Yanking or push of gem versions is not allowed. Repushing of gem versions is not allowed.'
release_rubygems_already_exists "${gem_exists}" || fail "gem already-pushed error must skip"

gem_other='ERROR:  While executing gem ... (Gem::OperationNotSupportedError)
    Pushing to https://rubygems.org/ failed'
release_rubygems_already_exists "${gem_other}" && fail "unrelated gem error must not skip"

[[ "$(release_publish_outcome yes 0 release_npm_already_exists "")" == "skip" ]] ||
  fail "registry hit skips before publish"
[[ "$(release_publish_outcome no 0 release_npm_already_exists "Uploading")" == "ok" ]] ||
  fail "successful publish is ok"
[[ "$(release_publish_outcome no 1 release_npm_already_exists "${npm_exists}")" == "skip" ]] ||
  fail "already-exists publish failure must skip"
if release_publish_outcome no 1 release_npm_already_exists "${npm_other}" >/dev/null; then
  fail "unrelated publish failure must not skip"
fi
[[ "$(release_publish_outcome no 1 release_npm_already_exists "${npm_other}" || true)" == "fail" ]] ||
  fail "unrelated publish failure reports fail"

[[ "$(release_publish_outcome no 1 release_rubygems_already_exists "${gem_exists}")" == "skip" ]] ||
  fail "gem already-exists publish failure must skip"

jsr_exists='Warning: Skipping, already published @kiskolabs/pray-cli@1.23.0'
release_jsr_already_exists "${jsr_exists}" || fail "jsr already-published warning must skip"

jsr_duplicate='error: Failed to publish @kiskolabs/pray-cli at 1.23.0: duplicateVersionPublish'
release_jsr_already_exists "${jsr_duplicate}" || fail "jsr duplicateVersionPublish must skip"

jsr_other='error: No means to authenticate. Pass a token to `--token`.'
release_jsr_already_exists "${jsr_other}" && fail "unrelated jsr error must not skip"

[[ "$(release_publish_outcome no 1 release_jsr_already_exists "${jsr_duplicate}")" == "skip" ]] ||
  fail "jsr already-exists publish failure must skip"

echo "registry status tests passed"
