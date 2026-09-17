#!/usr/bin/env bash
# Extract one CHANGELOG.md version section for GitHub Release notes.
#
# Headings are `## X.Y.Z` or `## X.Y.Z (YYYY-MM-DD)`. The document title
# `# CHANGELOG` is never part of a release body.
set -euo pipefail

release_changelog_versions() {
  local file="$1"
  awk '$1 == "##" { print $2 }' "${file}"
}

release_changelog_section() {
  local file="$1"
  local version="$2"
  awk -v version="${version}" '
    $1 == "##" && $2 == version { found = 1 }
    found && $1 == "##" && $2 != version { exit }
    found { lines[++n] = $0 }
    END {
      if (!found) {
        printf "no CHANGELOG section for %s\n", version > "/dev/stderr"
        exit 1
      }
      while (n > 0 && lines[n] ~ /^[ \t]*$/) n--
      for (i = 1; i <= n; i++) print lines[i]
    }
  ' "${file}"
}

release_github_release_title() {
  local version="$1"
  printf 'v%s\n' "${version}"
}

release_changelog_has_version() {
  local file="$1"
  local version="$2"
  release_changelog_versions "${file}" | grep -Fxq "${version}"
}
