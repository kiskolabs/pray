#!/usr/bin/env bash
# Create or update a GitHub Release from one CHANGELOG.md version section.
#
# Usage:
#   scripts/release/github.sh                 # dry-run workspace version
#   scripts/release/github.sh --publish       # create or edit that release
#   scripts/release/github.sh --publish --all # edit every existing GitHub Release
#   scripts/release/github.sh --notes-only
#
# Title is vX.Y.Z so Discord and GitHub never show an empty name.
# Notes are the matching CHANGELOG heading and its bullets only.
# --all edits existing releases. It does not create missing tags or releases.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=common.sh
source "${SCRIPT_DIR}/common.sh"
# shellcheck source=changelog.sh
source "${SCRIPT_DIR}/changelog.sh"

ROOT="$(release_repo_root)"
CHANGELOG="${ROOT}/CHANGELOG.md"
VERSION="$(release_read_workspace_version "${ROOT}")"
MODE="dry-run"
SYNC_ALL=0
NOTES_ONLY=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --publish) MODE="publish"; shift ;;
    --dry-run) MODE="dry-run"; shift ;;
    --all) SYNC_ALL=1; shift ;;
    --notes-only) NOTES_ONLY=1; shift ;;
    --version) VERSION="$2"; shift 2 ;;
    -h | --help) sed -n '2,15p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done

VERSION="${VERSION#v}"

github_release_exists() {
  gh release view "$1" >/dev/null 2>&1
}

print_plan() {
  local version="$1"
  local tag notes
  tag="$(release_github_release_title "${version}")"
  notes="$(release_changelog_section "${CHANGELOG}" "${version}")"
  if [[ "${NOTES_ONLY}" -eq 1 ]]; then
    printf '%s\n' "${notes}"
    return 0
  fi
  echo "tag: ${tag}"
  echo "title: ${tag}"
  echo "notes:"
  printf '%s\n' "${notes}"
}

apply_release() {
  local version="$1"
  local tag notes_file workspace
  tag="$(release_github_release_title "${version}")"
  notes_file="$(mktemp)"
  workspace="$(release_read_workspace_version "${ROOT}")"
  release_changelog_section "${CHANGELOG}" "${version}" >"${notes_file}"
  if github_release_exists "${tag}"; then
    echo "==> gh release edit ${tag} --title ${tag}"
    gh release edit "${tag}" --title "${tag}" --notes-file "${notes_file}"
  elif [[ "${version}" == "${workspace}" ]]; then
    echo "==> gh release create ${tag} --title ${tag} --verify-tag"
    gh release create "${tag}" --title "${tag}" --notes-file "${notes_file}" --verify-tag
  else
    echo "==> gh release create ${tag} --title ${tag} --verify-tag --latest=false"
    gh release create "${tag}" --title "${tag}" --notes-file "${notes_file}" --verify-tag --latest=false
  fi
  rm -f "${notes_file}"
}

collect_versions() {
  if [[ "${SYNC_ALL}" -eq 0 ]]; then
    echo "${VERSION}"
    return 0
  fi
  release_changelog_versions "${CHANGELOG}"
}

if [[ "${NOTES_ONLY}" -eq 1 && "${SYNC_ALL}" -eq 1 ]]; then
  echo "--notes-only cannot be combined with --all" >&2
  exit 2
fi

cd "${ROOT}"
[[ -f "${CHANGELOG}" ]] || { echo "missing ${CHANGELOG}" >&2; exit 2; }

if [[ "${NOTES_ONLY}" -eq 1 ]]; then
  print_plan "${VERSION}"
  exit 0
fi

if [[ "${MODE}" == "publish" || "${SYNC_ALL}" -eq 1 ]]; then
  release_require_command gh
fi

echo "mode: ${MODE}"

planned=()
while IFS= read -r version; do
  [[ -n "${version}" ]] || continue
  tag="$(release_github_release_title "${version}")"
  if [[ "${SYNC_ALL}" -eq 1 ]] && ! github_release_exists "${tag}"; then
    echo "skip ${tag} (no GitHub Release)"
    continue
  fi
  planned+=("${version}")
  echo "==> ${tag}"
  print_plan "${version}"
  echo
done < <(collect_versions)

if [[ "${MODE}" != "publish" ]]; then
  echo "github release dry-run finished"
  exit 0
fi

if [[ ${#planned[@]} -eq 0 ]]; then
  echo "no GitHub Releases to update" >&2
  exit 2
fi

if ! release_confirm "Write GitHub Release notes for ${#planned[@]} tag(s)?"; then
  echo "skipped github release publish"
  exit 0
fi

for version in "${planned[@]}"; do
  apply_release "${version}"
done

echo "github release publish finished"
