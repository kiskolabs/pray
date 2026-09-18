#!/usr/bin/env bash
# Prove crates.io skip treats an already-published version as continue, not fail.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=crates_status.sh
source "${SCRIPT_DIR}/crates_status.sh"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

exists_error='error: crate pray-core@1.19.0 already exists on crates.io index'
release_cargo_already_exists "${exists_error}" || fail "publish already-exists error must skip"

exists_warning='warning: crate pray-core@1.19.0 already exists on crates.io index'
release_cargo_already_exists "${exists_warning}" || fail "dry-run already-exists warning must match"

other_error='error: failed to prepare local package for uploading'
release_cargo_already_exists "${other_error}" && fail "unrelated cargo error must not skip"

local_info="$(cat <<'EOF'
pray-core
version: 1.19.0 (from ./crates/pray-core)
license: MIT
EOF
)"
release_crates_io_info_is_published "pray-core" "1.19.0" "${local_info}" &&
  fail "workspace cargo info must not count as published"

registry_info="$(cat <<'EOF'
pray-core
version: 1.19.0
crates.io: https://crates.io/crates/pray-core/1.19.0
license: MIT
EOF
)"
release_crates_io_info_is_published "pray-core" "1.19.0" "${registry_info}" ||
  fail "crates.io cargo info must count as published"

release_crates_io_info_is_published "pray-core" "1.18.0" "${registry_info}" &&
  fail "other version URL must not count as published"

[[ "$(release_crate_publish_outcome yes 0 "")" == "skip" ]] || fail "index hit skips before publish"
[[ "$(release_crate_publish_outcome no 0 "Uploading")" == "ok" ]] || fail "successful publish is ok"
[[ "$(release_crate_publish_outcome no 101 "${exists_error}")" == "skip" ]] ||
  fail "already-exists publish failure must skip"
if release_crate_publish_outcome no 101 "${other_error}" >/dev/null; then
  fail "unrelated publish failure must not skip"
fi
[[ "$(release_crate_publish_outcome no 101 "${other_error}" || true)" == "fail" ]] ||
  fail "unrelated publish failure reports fail"

echo "crates status tests passed"
