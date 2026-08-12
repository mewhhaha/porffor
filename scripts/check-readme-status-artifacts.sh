#!/usr/bin/env bash
set -euo pipefail

base_ref="${1:-${GITHUB_BASE_REF:-origin/main}}"

changed_files="$(git diff --name-only "$base_ref"...HEAD)"
if ! printf '%s\n' "$changed_files" | grep -qx 'README.md'; then
  printf 'check-readme-status-artifacts: README.md unchanged\n'
  exit 0
fi

base_status="$(git show "$base_ref:README.md" | sed -n '/<!-- lila-status:start -->/,/<!-- lila-status:end -->/p')"
head_status="$(git show HEAD:README.md | sed -n '/<!-- lila-status:start -->/,/<!-- lila-status:end -->/p')"

if [ "$base_status" = "$head_status" ]; then
  printf 'check-readme-status-artifacts: generated README status block unchanged\n'
  exit 0
fi

# T27 changed publication policy without changing conformance evidence: the
# stale status block used to point its refresh command at the oracle backend.
# Permit only that exact command-token correction, and only in the direction
# that leaves a wasm-aot command and no spec-exec publisher invocation.
normalize_refresh_backend() {
  sed -E 's#(\./scripts/publish-real-status-low-ram\.sh) (spec-exec|wasm|wasm-aot)( codex-published-real)#\1 <product-backend>\3#'
}

if printf '%s\n' "$head_status" | grep -Fqx -- '- `./scripts/publish-real-status-low-ram.sh wasm-aot codex-published-real`' \
  && ! printf '%s\n' "$head_status" | grep -Fq './scripts/publish-real-status-low-ram.sh spec-exec' \
  && [ "$(printf '%s\n' "$base_status" | normalize_refresh_backend)" = "$(printf '%s\n' "$head_status" | normalize_refresh_backend)" ]; then
  printf 'check-readme-status-artifacts: status evidence unchanged; refresh command is wasm-aot-only\n'
  exit 0
fi

if printf '%s\n' "$changed_files" | grep -Eq '^(test262/snapshots/|crates/lila-test262/tests/fixtures/.*/snapshots/)'; then
  printf 'check-readme-status-artifacts: README status change has snapshot artifact changes\n'
  exit 0
fi

cat >&2 <<'EOF'
check-readme-status-artifacts: README generated status block changed without Test262 snapshot artifact changes.
Use `lila test262 publish-status` or `scripts/publish-real-status-low-ram.sh`, or keep documentation-only edits outside the generated block.
EOF
exit 1
