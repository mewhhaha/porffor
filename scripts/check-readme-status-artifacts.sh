#!/usr/bin/env bash
set -euo pipefail

base_ref="${1:-${GITHUB_BASE_REF:-origin/main}}"

changed_files="$(git diff --name-only "$base_ref"...HEAD)"
if ! printf '%s\n' "$changed_files" | grep -qx 'README.md'; then
  printf 'check-readme-status-artifacts: README.md unchanged\n'
  exit 0
fi

base_status="$(git show "$base_ref:README.md" | sed -n '/<!-- porffor-status:start -->/,/<!-- porffor-status:end -->/p')"
head_status="$(git show HEAD:README.md | sed -n '/<!-- porffor-status:start -->/,/<!-- porffor-status:end -->/p')"

if [ "$base_status" = "$head_status" ]; then
  printf 'check-readme-status-artifacts: generated README status block unchanged\n'
  exit 0
fi

if printf '%s\n' "$changed_files" | grep -Eq '^(test262/snapshots/|crates/porffor-test262/tests/fixtures/.*/snapshots/)'; then
  printf 'check-readme-status-artifacts: README status change has snapshot artifact changes\n'
  exit 0
fi

cat >&2 <<'EOF'
check-readme-status-artifacts: README generated status block changed without Test262 snapshot artifact changes.
Use `porf test262 publish-status` or `scripts/publish-real-status-low-ram.sh`, or keep documentation-only edits outside the generated block.
EOF
exit 1
