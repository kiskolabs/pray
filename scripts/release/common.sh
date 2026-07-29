#!/usr/bin/env bash
# Shared helpers for manual release scripts.
set -euo pipefail

release_repo_root() {
  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  cd "${script_dir}/../.." && pwd
}

release_read_workspace_version() {
  local root="$1"
  sed -n 's/^version = "\(.*\)"/\1/p' "${root}/Cargo.toml" | head -1
}

release_require_command() {
  local name="$1"
  if ! command -v "${name}" >/dev/null 2>&1; then
    echo "missing required command: ${name}" >&2
    exit 2
  fi
}

release_assert_version_alignment() {
  local root="$1"
  local version="$2"
  local npm_version
  local gem_version

  npm_version="$(
    node -e 'console.log(JSON.parse(require("fs").readFileSync(process.argv[1], "utf8")).version)' \
      "${root}/npmjs/pray-cli/package.json"
  )"
  gem_version="$(
    ruby -e 'load ARGV[0]; puts Pray::VERSION' "${root}/rubygems/pray-cli/lib/pray/version.rb"
  )"

  if [[ "${npm_version}" != "${version}" ]]; then
    echo "npm package version ${npm_version} does not match workspace ${version}" >&2
    exit 2
  fi
  if [[ "${gem_version}" != "${version}" ]]; then
    echo "gem version ${gem_version} does not match workspace ${version}" >&2
    exit 2
  fi

  local ts_version
  ts_version="$(
    sed -n 's/^export const PACKAGE_VERSION = "\(.*\)";/\1/p' \
      "${root}/npmjs/pray-cli/src/lockfile/types.ts" | head -1
  )"
  if [[ "${ts_version}" != "${version}" ]]; then
    echo "TypeScript PACKAGE_VERSION ${ts_version} does not match workspace ${version}" >&2
    exit 2
  fi
}

release_confirm() {
  local prompt="$1"
  if [[ "${PRAY_RELEASE_YES:-}" == "1" ]]; then
    return 0
  fi
  printf '%s [y/N] ' "${prompt}"
  local answer
  read -r answer
  [[ "${answer}" == "y" || "${answer}" == "Y" ]]
}
