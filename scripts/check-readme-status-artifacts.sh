#!/usr/bin/env bash
set -euo pipefail

base_ref="${1:-${GITHUB_BASE_REF:-origin/main}}"
canonical_status_json='test262/snapshots/published-status-wasm-aot.json'
canonical_status_txt='test262/snapshots/published-status-wasm-aot.txt'

changed_files="$(git diff --name-only "$base_ref"...HEAD)"
changed_file() {
  grep -Fqx -- "$1" <<<"$changed_files"
}

canonical_status_blob() {
  local path="$1"
  local entry
  local metadata
  local recorded_path
  local mode
  local object_type
  local object_id

  [ -f "$path" ] && [ ! -L "$path" ] || return 1
  entry="$(git ls-tree HEAD -- "$path")"
  IFS=$'\t' read -r metadata recorded_path <<<"$entry"
  read -r mode object_type object_id <<<"$metadata"
  [ "$recorded_path" = "$path" ] \
    && [ "$mode" = '100644' ] \
    && [ "$object_type" = 'blob' ] \
    && [ -n "$object_id" ]
}

if ! changed_file 'README.md'; then
  printf 'check-readme-status-artifacts: README.md unchanged\n'
  exit 0
fi

extract_status_block() {
  local ref="$1"
  local readme
  local start_marker
  local end_marker

  readme="$(git show "$ref:README.md")"
  if grep -Fq '<!-- lila-status:start -->' <<<"$readme"; then
    start_marker='<!-- lila-status:start -->'
    end_marker='<!-- lila-status:end -->'
  elif grep -Fq '<!-- porffor-status:start -->' <<<"$readme"; then
    start_marker='<!-- porffor-status:start -->'
    end_marker='<!-- porffor-status:end -->'
  else
    printf 'check-readme-status-artifacts: missing generated status markers in %s:README.md\n' "$ref" >&2
    return 1
  fi

  sed -n "\\|$start_marker|,\\|$end_marker|p" <<<"$readme"
}

base_status="$(extract_status_block "$base_ref")"
head_status="$(extract_status_block HEAD)"

if [ "$base_status" = "$head_status" ]; then
  printf 'check-readme-status-artifacts: generated README status block unchanged\n'
  exit 0
fi

if grep -Eq '^test262/snapshots/published-status-spec-exec\.(json|txt)$' <<<"$changed_files"; then
  cat >&2 <<'EOF'
check-readme-status-artifacts: spec-exec is oracle-only and cannot authorize the generated README status block.
Use `lila test262 publish-status` or `scripts/publish-real-status-low-ram.sh` with the wasm-aot product backend.
EOF
  exit 1
fi

# T27/T29 changed publication policy and repository identity without changing
# conformance evidence. Normalize only the exact historical markers, command
# tokens and fake-suite path; every status/count/content byte remains compared.
normalize_t27_t29_status() {
  sed \
    -e 's/<!-- porffor-status:start -->/<!-- lila-status:start -->/' \
    -e 's/<!-- porffor-status:end -->/<!-- lila-status:end -->/' \
    -e 's/-p porffor-engine/-p lila-engine/' \
    -e 's/-p porffor-cli/-p lila-cli/' \
    -e 's#\./target/debug/porf #./target/debug/lila #g' \
    -e 's#crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262#crates/lila-test262/tests/fixtures/fake_test262/vendor/test262#g' \
    -e 's#`\./scripts/publish-real-status-low-ram\.sh spec-exec codex-published-real`#`./scripts/publish-real-status-low-ram.sh wasm-aot codex-published-real`#'
}

if grep -Fqx -- '<!-- porffor-status:start -->' <<<"$base_status" \
  && grep -Fqx -- '<!-- lila-status:start -->' <<<"$head_status" \
  && grep -Fqx -- '- `./scripts/publish-real-status-low-ram.sh spec-exec codex-published-real`' <<<"$base_status" \
  && grep -Fqx -- '- `./scripts/publish-real-status-low-ram.sh wasm-aot codex-published-real`' <<<"$head_status" \
  && ! grep -Fq './scripts/publish-real-status-low-ram.sh spec-exec' <<<"$head_status" \
  && [ "$(normalize_t27_t29_status <<<"$base_status")" = "$head_status" ]; then
  printf 'check-readme-status-artifacts: status evidence unchanged; exact T27/T29 identity and wasm-aot policy migration\n'
  exit 0
fi

if changed_file "$canonical_status_json" \
  && changed_file "$canonical_status_txt" \
  && canonical_status_blob "$canonical_status_json" \
  && canonical_status_blob "$canonical_status_txt"; then
  printf 'check-readme-status-artifacts: README status change has the canonical wasm-aot JSON/text publication pair\n'
  exit 0
fi

cat >&2 <<EOF
check-readme-status-artifacts: README generated status block changed without both canonical wasm-aot publication artifacts:
  $canonical_status_json
  $canonical_status_txt
Matrix nodes, aggregate snapshots, focused/fake snapshots, oracle artifacts, either artifact alone, and non-regular/symlink outputs do not authorize this block.
Use \`lila test262 publish-status\` or \`scripts/publish-real-status-low-ram.sh\`, or keep documentation-only edits outside the generated block.
EOF
exit 1
