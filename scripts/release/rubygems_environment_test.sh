#!/usr/bin/env bash
# Prove RubyGems releases ignore user-installed gems from another Ruby build.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

fixture="$(mktemp -d)"
trap 'rm -rf "${fixture}"' EXIT
package_dir="${fixture}/pray-cli"
mkdir -p "${package_dir}"

export GEM_HOME="${fixture}/shared-gems"
export GEM_PATH="${fixture}/shared-gems:${fixture}/other-ruby-gems"
export BUNDLE_PATH="${fixture}/shared-bundle"
export BUNDLE_USER_HOME="${fixture}/shared-bundler-home"
export BUNDLE_APP_CONFIG="${fixture}/shared-bundler-config"

if ! declare -F release_configure_ruby_environment >/dev/null; then
  fail "missing release_configure_ruby_environment"
fi

original_directory="${PWD}"
cd "${fixture}"
release_configure_ruby_environment "pray-cli"
cd "${original_directory}"

case "${GEM_HOME}" in
  "${package_dir}/tmp/release-gems/"*) ;;
  *) fail "GEM_HOME is not isolated by Ruby build: ${GEM_HOME}" ;;
esac
[[ "${GEM_PATH}" == "${GEM_HOME}" ]] || fail "GEM_PATH includes gems outside the release environment"
[[ "${BUNDLE_PATH}" == "${GEM_HOME}" ]] || fail "Bundler installs outside the release environment"
[[ "${BUNDLE_USER_HOME}" == "${GEM_HOME}/.bundler" ]] || fail "Bundler user home is not isolated"
[[ "${BUNDLE_APP_CONFIG}" == "${GEM_HOME}/.bundler-app-config" ]] || fail "Bundler app config is not isolated"
[[ -d "${GEM_HOME}" ]] || fail "release gem home was not created"

ruby -e '
  expected = File.realpath(ENV.fetch("GEM_HOME"))
  actual = Gem.path.map { |path| File.realpath(path) }
  abort "unexpected Gem.path: #{Gem.path.inspect}" unless actual == [expected]
'

echo "RubyGems release environment isolation passed"
