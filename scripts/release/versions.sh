#!/usr/bin/env bash
# Coordinated version helpers for language-registry publish.
#
# Three surfaces: crates (workspace), npm, gem. TypeScript PACKAGE_VERSION
# must match npm. Next version all three may share is the max of those
# numbers. A max held by one surface only cannot be aligned; the others
# skip it and jump to the next patch, whether that max was a patch or a
# minor.
set -euo pipefail

release_read_npm_version() {
  local root="$1"
  node -e 'console.log(JSON.parse(require("fs").readFileSync(process.argv[1], "utf8")).version)' \
    "${root}/npmjs/pray-cli/package.json"
}

release_read_typescript_version() {
  local root="$1"
  sed -n 's/^export const PACKAGE_VERSION = "\(.*\)";/\1/p' \
    "${root}/npmjs/pray-cli/src/lockfile/types.ts" | head -1
}

release_version_max() {
  printf '%s\n%s\n' "$1" "$2" | sort -V | tail -1
}

release_bump_patch() {
  local major minor patch
  IFS=. read -r major minor patch _ <<<"$1"
  printf '%s.%s.%s\n' "${major}" "${minor}" "$((patch + 1))"
}

release_next_coordinated_from() {
  local workspace="$1"
  local npm="$2"
  local gem="$3"
  local max holders
  max="$(release_version_max "${workspace}" "${npm}")"
  max="$(release_version_max "${max}" "${gem}")"
  holders=0
  [[ "${workspace}" == "${max}" ]] && holders=$((holders + 1))
  [[ "${npm}" == "${max}" ]] && holders=$((holders + 1))
  [[ "${gem}" == "${max}" ]] && holders=$((holders + 1))
  if [[ "${holders}" -le 1 ]]; then
    release_bump_patch "${max}"
    return
  fi
  printf '%s\n' "${max}"
}

release_publish_kind() {
  local workspace="$1"
  local npm="$2"
  local gem="$3"
  local max holders kind
  if [[ "${workspace}" == "${npm}" && "${npm}" == "${gem}" ]]; then
    printf 'aligned\n'
    return
  fi
  max="$(release_version_max "${workspace}" "${npm}")"
  max="$(release_version_max "${max}" "${gem}")"
  holders=0
  kind=""
  if [[ "${workspace}" == "${max}" ]]; then
    holders=$((holders + 1))
    kind="crates"
  fi
  if [[ "${npm}" == "${max}" ]]; then
    holders=$((holders + 1))
    kind="npm"
  fi
  if [[ "${gem}" == "${max}" ]]; then
    holders=$((holders + 1))
    kind="gem"
  fi
  if [[ "${holders}" -eq 1 ]]; then
    printf '%s\n' "${kind}"
    return
  fi
  printf 'drift\n'
}

release_next_coordinated_version() {
  local root="$1"
  release_next_coordinated_from \
    "$(release_read_workspace_version "${root}")" \
    "$(release_read_npm_version "${root}")" \
    "$(release_read_gem_version "${root}")"
}

release_assert_npm_typescript_match() {
  local root="$1"
  local npm typescript
  npm="$(release_read_npm_version "${root}")"
  typescript="$(release_read_typescript_version "${root}")"
  if [[ "${npm}" != "${typescript}" ]]; then
    echo "TypeScript PACKAGE_VERSION ${typescript} does not match npm ${npm}" >&2
    return 2
  fi
}

release_assert_coordinated_changelog() {
  local root="$1"
  local version="$2"
  local next file
  for file in \
    "${root}/CHANGELOG.md" \
    "${root}/npmjs/pray-cli/CHANGELOG.md" \
    "${root}/rubygems/pray-cli/CHANGELOG.md"; do
    if ! release_changelog_has_version "${file}" "${version}"; then
      next="$(release_next_coordinated_version "${root}")"
      echo "${version} is not a coordinated version; root, npm, and Ruby changelogs must all list it" >&2
      echo "next coordinated version is ${next}" >&2
      return 2
    fi
  done
}

release_begin_surface_publish() {
  local root="$1"
  local surface="$2"
  local workspace npm gem kind
  workspace="$(release_read_workspace_version "${root}")"
  npm="$(release_read_npm_version "${root}")"
  gem="$(release_read_gem_version "${root}")"
  kind="$(release_publish_kind "${workspace}" "${npm}" "${gem}")"
  if [[ "${kind}" == "aligned" ]]; then
    release_assert_version_alignment "${root}" "${workspace}"
    release_assert_coordinated_changelog "${root}" "${workspace}"
    return 0
  fi
  if [[ "${kind}" == "${surface}" ]]; then
    echo "warn: ${surface} is a one-surface version; other registries skip it"
    echo "next coordinated version is $(release_next_coordinated_from "${workspace}" "${npm}" "${gem}")"
    return 0
  fi
  echo "cannot publish ${surface}: crates ${workspace}, npm ${npm}, gem ${gem}" >&2
  echo "next coordinated version is $(release_next_coordinated_from "${workspace}" "${npm}" "${gem}")" >&2
  return 2
}
