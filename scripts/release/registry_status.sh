#!/usr/bin/env bash
# Skip already-published npm, RubyGems, and jsr.io versions during release.
set -euo pipefail

release_http_ok() {
  [[ "$1" == "200" ]]
}

release_fetch_http_status() {
  local url="$1"
  curl -sS -o /dev/null -w '%{http_code}' --max-time 30 "${url}"
}

release_npm_already_exists() {
  local output="$1"
  [[ "${output}" == *"cannot publish over the previously published versions"* ]] ||
    [[ "${output}" == *"previously staged version"* ]] ||
    [[ "${output}" == *"EPUBLISHCONFLICT"* ]] ||
    [[ "${output}" == *"code E409"* ]]
}

# After a publish attempt, the registry can return E409 for a staged version before
# GET /package/version is 200. Wait briefly so release-all can skip and continue.
release_npm_wait_for_version() {
  local package="$1"
  local version="$2"
  local attempt
  for attempt in 1 2 3 4 5 6 7 8; do
    if release_npm_has_version "${package}" "${version}"; then
      return 0
    fi
    sleep 2
  done
  return 1
}

release_rubygems_already_exists() {
  local output="$1"
  [[ "${output}" == *"Repushing of gem versions is not allowed"* ]] ||
    [[ "${output}" == *"has already been pushed"* ]]
}

release_jsr_already_exists() {
  local output="$1"
  [[ "${output}" == *"Skipping, already published"* ]] ||
    [[ "${output}" == *"duplicateVersionPublish"* ]] ||
    [[ "${output}" == *"version already exists"* ]]
}

release_publish_outcome() {
  local on_registry="$1"
  local status="$2"
  local already_exists_fn="$3"
  local output="$4"
  if [[ "${on_registry}" == "yes" ]]; then
    printf 'skip\n'
    return 0
  fi
  if [[ "${status}" -eq 0 ]]; then
    printf 'ok\n'
    return 0
  fi
  if "${already_exists_fn}" "${output}"; then
    printf 'skip\n'
    return 0
  fi
  printf 'fail\n'
  return 1
}

release_npm_has_version() {
  local package="$1"
  local version="$2"
  local http_status
  http_status="$(release_fetch_http_status "https://registry.npmjs.org/${package}/${version}")"
  release_http_ok "${http_status}"
}

release_rubygems_has_version() {
  local gem_name="$1"
  local version="$2"
  local http_status
  http_status="$(
    release_fetch_http_status \
      "https://rubygems.org/api/v2/rubygems/${gem_name}/versions/${version}.json"
  )"
  release_http_ok "${http_status}"
}

release_jsr_has_version() {
  local package="$1"
  local version="$2"
  local http_status
  http_status="$(release_fetch_http_status "https://jsr.io/${package}/${version}_meta.json")"
  release_http_ok "${http_status}"
}
