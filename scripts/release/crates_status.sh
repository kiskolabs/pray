#!/usr/bin/env bash
# Skip already-published crate versions during crates.io release.
set -euo pipefail

release_cargo_already_exists() {
  [[ "$1" == *"already exists on crates.io index"* ]]
}

release_crates_io_info_is_published() {
  local crate="$1"
  local version="$2"
  local info="$3"
  [[ "${info}" == *"crates.io: https://crates.io/crates/${crate}/${version}"* ]]
}

release_crate_publish_outcome() {
  local on_index="$1"
  local status="$2"
  local output="$3"
  if [[ "${on_index}" == "yes" ]]; then
    printf 'skip\n'
    return 0
  fi
  if [[ "${status}" -eq 0 ]]; then
    printf 'ok\n'
    return 0
  fi
  if release_cargo_already_exists "${output}"; then
    printf 'skip\n'
    return 0
  fi
  printf 'fail\n'
  return 1
}

release_crates_io_has_version() {
  local crate="$1"
  local version="$2"
  local info
  if ! info="$(cargo info --quiet --registry crates-io "${crate}@${version}" 2>/dev/null)"; then
    return 1
  fi
  release_crates_io_info_is_published "${crate}" "${version}" "${info}"
}
