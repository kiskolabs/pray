#!/usr/bin/env bash
# Warn at 150 LOC; fail above 300 unless path is ratcheted in loc-limits.allowlist.
# Allowlisted files may not grow past their recorded max. Blank lines count.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WARN_MAX=150
FAIL_MAX=300
ALLOWLIST="${ROOT}/scripts/loc-limits.allowlist"

cd "$ROOT"

allowed_max_for() {
  local path="$1"
  if [[ ! -f "$ALLOWLIST" ]]; then
    return 1
  fi
  awk -v path="$path" '
    /^[[:space:]]*#/ || /^[[:space:]]*$/ { next }
    $1 == path { print $2; found = 1; exit }
    END { exit found ? 0 : 1 }
  ' "$ALLOWLIST"
}

warnings=0
failures=0

while IFS= read -r file; do
  [[ -f "$file" ]] || continue
  lines="$(wc -l <"$file" | tr -d ' ')"
  rel="${file#./}"

  if ((lines <= WARN_MAX)); then
    continue
  fi

  if ((lines <= FAIL_MAX)); then
    printf 'WARN  %5d/%d  %s\n' "$lines" "$WARN_MAX" "$rel"
    warnings=$((warnings + 1))
    continue
  fi

  if allowed="$(allowed_max_for "$rel")"; then
    if ((lines > allowed)); then
      printf 'FAIL  %5d/%d  %s  (grew past allowlist ratchet)\n' "$lines" "$allowed" "$rel"
      failures=$((failures + 1))
    else
      printf 'WARN  %5d/%d  %s  (allowlisted; hard limit %d)\n' "$lines" "$WARN_MAX" "$rel" "$FAIL_MAX"
      warnings=$((warnings + 1))
    fi
    continue
  fi

  printf 'FAIL  %5d/%d  %s\n' "$lines" "$FAIL_MAX" "$rel"
  failures=$((failures + 1))
done < <(
  find \
    crates/*/src \
    rubygems/pray-cli/lib \
    npmjs/pray-cli/src \
    -type f \( -name '*.rs' -o -name '*.rb' -o -name '*.ts' \) \
    ! -name '*.test.ts' \
    | LC_ALL=C sort
)

printf '\nloc-limits: %d warning(s) (>=%d), %d failure(s) (>%d without ratchet)\n' \
  "$warnings" "$WARN_MAX" "$failures" "$FAIL_MAX"

if ((failures > 0)); then
  exit 1
fi
