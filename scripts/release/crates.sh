#!/usr/bin/env bash
# Publish Rust crates to crates.io in dependency order.
#
# Usage:
#   scripts/release/crates.sh            # dry-run (default)
#   scripts/release/crates.sh --publish  # real publish (manual)
#
# Requires: cargo login (or CARGO_REGISTRY_TOKEN)
#
# First-time note: cargo publish --dry-run for pray-transport / pray-cli
# only succeeds after pray-core (and then pray-transport) exist on crates.io.
# Until then, dry-run validates pray-core packaging and cargo check for dependents.
# Publish skips a crate whose version is already on crates.io so release-all can resume.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"
# shellcheck source=crates_status.sh
source "${SCRIPT_DIR}/crates_status.sh"

ROOT="$(release_repo_root)"
VERSION="$(release_read_workspace_version "${ROOT}")"
MODE="dry-run"

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

release_require_command cargo
release_begin_surface_publish "${ROOT}" "crates"

cd "${ROOT}"

CRATES=(pray-core pray-transport pray-cli)

echo "workspace version: ${VERSION}"
echo "mode: ${MODE}"
echo "crates: ${CRATES[*]}"

dry_run_crate() {
  local crate="$1"
  echo "==> cargo publish -p ${crate} --dry-run --locked --allow-dirty"
  if cargo publish -p "${crate}" --dry-run --locked --allow-dirty; then
    return 0
  fi
  echo "warn: full publish dry-run for ${crate} needs its crates.io dependencies published first" >&2
  echo "==> fallback cargo check -p ${crate} --locked"
  cargo check -p "${crate}" --locked
}

for crate in "${CRATES[@]}"; do
  if [[ "${MODE}" == "dry-run" ]]; then
    dry_run_crate "${crate}"
  else
    if release_crates_io_has_version "${crate}" "${VERSION}"; then
      echo "skip ${crate}@${VERSION} (already on crates.io)"
      continue
    fi
    if ! release_confirm "Publish ${crate} ${VERSION} to crates.io?"; then
      echo "skipped ${crate}"
      continue
    fi
    echo "==> cargo publish -p ${crate} --locked"
    publish_log="$(mktemp)"
    set +e
    cargo publish -p "${crate}" --locked 2>&1 | tee "${publish_log}"
    publish_status="${PIPESTATUS[0]}"
    set -e
    outcome="$(release_crate_publish_outcome no "${publish_status}" "$(cat "${publish_log}")" || true)"
    rm -f "${publish_log}"
    if [[ "${outcome}" == "skip" ]]; then
      echo "skip ${crate}@${VERSION} (already on crates.io)"
      continue
    fi
    if [[ "${publish_status}" -ne 0 ]]; then
      exit "${publish_status}"
    fi
    if [[ "${crate}" != "pray-cli" ]]; then
      echo "waiting briefly for crates.io index to observe ${crate}@${VERSION}"
      sleep 5
    fi
  fi
done

echo "crates release ${MODE} finished"
