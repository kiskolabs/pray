#!/usr/bin/env bash
# Prove CHANGELOG notes cover one version heading and its bullets only.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=changelog.sh
source "${SCRIPT_DIR}/changelog.sh"

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

fixture="$(mktemp)"
trap 'rm -f "${fixture}"' EXIT

cat >"${fixture}" <<'EOF'
# CHANGELOG

## 1.10.0 (2026-09-02)

- Newer bullet.

## 1.2.0 (2026-07-14)

- Middle bullet with `code`.
- Second middle bullet.

## 1.1.0 (2026-07-14)

- Older bullet.
EOF

section="$(release_changelog_section "${fixture}" "1.2.0")"
printf '%s\n' "${section}" | grep -q 'Middle bullet with `code`.' || fail "missing requested bullet"
printf '%s\n' "${section}" | grep -q 'Second middle bullet.' || fail "missing second bullet"
printf '%s\n' "${section}" | grep -q 'Newer bullet' && fail "included later version"
printf '%s\n' "${section}" | grep -q 'Older bullet' && fail "included earlier version"
printf '%s\n' "${section}" | grep -q '^# CHANGELOG' && fail "included document title"

first_line="$(printf '%s\n' "${section}" | head -1)"
[[ "${first_line}" == "## 1.2.0 (2026-09-02)" ]] && fail "wrong heading date"
[[ "${first_line}" == "## 1.2.0 (2026-07-14)" ]] || fail "expected version heading, got ${first_line}"

latest="$(release_changelog_section "${fixture}" "1.10.0")"
printf '%s\n' "${latest}" | grep -q 'Newer bullet.' || fail "missing latest bullet"
printf '%s\n' "${latest}" | grep -q 'Middle bullet' && fail "latest included middle version"

oldest="$(release_changelog_section "${fixture}" "1.1.0")"
printf '%s\n' "${oldest}" | grep -q 'Older bullet.' || fail "missing oldest bullet"
printf '%s\n' "${oldest}" | grep -q 'Middle bullet' && fail "oldest included middle version"

if release_changelog_section "${fixture}" "9.9.9" >/dev/null 2>&1; then
  fail "missing version must fail"
fi

if release_changelog_section "${fixture}" "1.1" >/dev/null 2>&1; then
  fail "partial version must not match 1.1.0"
fi

title="$(release_github_release_title "1.17.0")"
[[ "${title}" == "v1.17.0" ]] || fail "expected v1.17.0 title, got ${title}"
[[ -n "${title}" ]] || fail "title must not be empty"

first_listed="$(release_changelog_versions "${fixture}" | head -1)"
listed_count="$(release_changelog_versions "${fixture}" | wc -l | tr -d ' ')"
[[ "${first_listed}" == "1.10.0" ]] || fail "expected first listed version 1.10.0"
[[ "${listed_count}" -eq 3 ]] || fail "expected three version headings"

echo "changelog notes tests passed"
