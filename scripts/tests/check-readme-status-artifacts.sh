#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
guard="$repo_root/scripts/check-readme-status-artifacts.sh"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

fail() {
  printf 'check-readme-status-artifacts regression: %s\n' "$*" >&2
  exit 1
}

test_repo="$tmp_dir/repo"
git init -q "$test_repo"
cd "$test_repo"
git config user.email test@example.invalid
git config user.name 'README status guard test'
cp "$repo_root/.gitignore" .gitignore

write_base_readme() {
  printf '%s\n' \
    '# Fixture' \
    'outside=base' \
    '## Current Status' \
    '<!-- lila-status:start -->' \
    'status=base' \
    '- `./scripts/publish-real-status-low-ram.sh wasm-aot codex-published-real`' \
    '<!-- lila-status:end -->' \
    'tail=base' > README.md
}

write_actual_legacy_readme() {
  printf '%s\n' \
    '# Fixture' \
    'outside=base' \
    '## Current Status' \
    '<!-- porffor-status:start -->' \
    'Rust rewrite status must be read in layers, not one vanity number:' \
    '- Fake wasm-safe Test262 subset: `187/187` green' \
    '- Fake full Rust rewrite suite: `190/190` green' \
    '- Full pinned real Test262 for Rust rewrite: **not green / current pinned aggregate not yet fully republished**' \
    '- Current real-suite pin: `ecma262=ecma262-current-draft` `test262=e9d582d6b8b13afc5ba9a676664741592b5c7f69`' \
    '- Last complete cached `spec-exec` publish is stale for the current pin and must not be reported as current progress.' \
    '' \
    'As of `2026-04-30`, Rust Wasm-AOT path is at 100% of repo fake coverage, not 100% ECMAScript. Project is still off literal 100% until the full pinned real Test262 run is green for Rust path and the status artifact is republished.' \
    '' \
    'Status refresh commands:' \
    '- `cargo test -p porffor-engine --quiet`' \
    '- `cargo test -p porffor-cli --quiet`' \
    '- `./target/debug/porf test262 run language/wasm/pass --suite-root crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262 --execution-backend wasm`' \
    '- `./target/debug/porf test262 run --suite-root crates/porffor-test262/tests/fixtures/fake_test262/vendor/test262`' \
    '- `./scripts/publish-real-status-low-ram.sh spec-exec codex-published-real`' \
    '' \
    'When counts move, update this block in same change. Do not claim full Test262 `100%` from fake-suite numbers.' \
    '<!-- porffor-status:end -->' \
    'tail=base' > README.md
}

write_migrated_legacy_readme() {
  write_actual_legacy_readme
  sed \
    -e 's/porffor-status/lila-status/g' \
    -e 's/porffor-engine/lila-engine/g' \
    -e 's/porffor-cli/lila-cli/g' \
    -e 's#target/debug/porf #target/debug/lila #g' \
    -e 's#crates/porffor-test262#crates/lila-test262#g' \
    -e 's/publish-real-status-low-ram.sh spec-exec/publish-real-status-low-ram.sh wasm-aot/' \
    README.md > README.md.next
  mv README.md.next README.md
}

rewrite_readme() {
  local expression="$1"
  sed "$expression" README.md > README.md.next
  mv README.md.next README.md
}

change_generated_status() {
  rewrite_readme 's/status=base/status=changed/'
}

change_outside_status() {
  rewrite_readme 's/outside=base/outside=changed/'
}

change_refresh_to_spec_exec() {
  rewrite_readme 's/publish-real-status-low-ram.sh wasm-aot/publish-real-status-low-ram.sh spec-exec/'
}

change_refresh_to_wasm_alias() {
  rewrite_readme 's/publish-real-status-low-ram.sh wasm-aot/publish-real-status-low-ram.sh wasm/'
}

add_artifact() {
  local path="$1"
  mkdir -p "$(dirname "$path")"
  printf '{"fixture":true}\n' > "$path"
}

setup_outside_only() {
  change_outside_status
}

setup_readme_only() {
  change_generated_status
}

setup_node_snapshot() {
  change_generated_status
  add_artifact test262/snapshots/current-built-ins-123.json
}

setup_aggregate_snapshot() {
  change_generated_status
  add_artifact test262/snapshots/current-aggregate-123.json
}

setup_fake_snapshot() {
  change_generated_status
  add_artifact crates/lila-test262/tests/fixtures/fake_test262/snapshots/fake.json
}

setup_spec_exec_status() {
  change_generated_status
  add_artifact test262/snapshots/published-status-spec-exec.json
}

setup_canonical_json_only() {
  change_generated_status
  add_artifact test262/snapshots/published-status-wasm-aot.json
}

setup_canonical_txt_only() {
  change_generated_status
  add_artifact test262/snapshots/published-status-wasm-aot.txt
}

setup_canonical_pair() {
  change_generated_status
  add_artifact test262/snapshots/published-status-wasm-aot.json
  add_artifact test262/snapshots/published-status-wasm-aot.txt
}

setup_canonical_pair_with_node() {
  setup_canonical_pair
  add_artifact test262/snapshots/current-built-ins-123.json
}

setup_canonical_pair_with_spec_exec() {
  setup_canonical_pair
  add_artifact test262/snapshots/published-status-spec-exec.json
}

setup_deleted_canonical_pair() {
  change_generated_status
  rm test262/snapshots/published-status-wasm-aot.json
  rm test262/snapshots/published-status-wasm-aot.txt
}

setup_canonical_symlink_pair() {
  change_generated_status
  rm test262/snapshots/published-status-wasm-aot.json
  rm test262/snapshots/published-status-wasm-aot.txt
  ln -s ../../README.md test262/snapshots/published-status-wasm-aot.json
  ln -s ../../README.md test262/snapshots/published-status-wasm-aot.txt
}

setup_reverse_t27_token_change() {
  change_refresh_to_spec_exec
}

setup_wasm_alias_t27_token_change() {
  change_refresh_to_wasm_alias
}

setup_large_changed_file_list() {
  local index

  change_generated_status
  mkdir -p bulk
  for index in $(seq 1 3000); do
    printf 'fixture\n' > "bulk/changed-file-$index"
  done
}

write_base_readme
mkdir -p test262/snapshots
printf '{"fixture":"base"}\n' > test262/snapshots/published-status-wasm-aot.json
printf 'fixture=base\n' > test262/snapshots/published-status-wasm-aot.txt
git add -A
git commit -qm base
base_commit="$(git rev-parse HEAD)"

run_case() {
  local name="$1"
  local expected_status="$2"
  local setup="$3"
  local output
  local status

  git checkout -q -B "case-$name" "$base_commit"
  "$setup"
  git add -A
  git commit -qm "$name"

  set +e
  output="$("$guard" "$base_commit" 2>&1)"
  status=$?
  set -e

  if [ "$status" -ne "$expected_status" ]; then
    fail "$name returned $status, expected $expected_status: $output"
  fi
  printf 'check-readme-status-artifacts regression: %s ok\n' "$name"
}

run_case outside-generated-block 0 setup_outside_only
run_case readme-only 1 setup_readme_only
run_case node-snapshot 1 setup_node_snapshot
run_case aggregate-snapshot 1 setup_aggregate_snapshot
run_case fake-snapshot 1 setup_fake_snapshot
run_case spec-exec-status 1 setup_spec_exec_status
run_case canonical-json-only 1 setup_canonical_json_only
run_case canonical-txt-only 1 setup_canonical_txt_only
run_case canonical-pair 0 setup_canonical_pair
run_case canonical-pair-with-node 0 setup_canonical_pair_with_node
run_case canonical-pair-with-spec-exec 1 setup_canonical_pair_with_spec_exec
run_case deleted-canonical-pair 1 setup_deleted_canonical_pair
run_case canonical-symlink-pair 1 setup_canonical_symlink_pair
for canonical_path in \
  test262/snapshots/published-status-wasm-aot.json \
  test262/snapshots/published-status-wasm-aot.txt; do
  canonical_entry="$(git ls-tree HEAD -- "$canonical_path")"
  canonical_mode="${canonical_entry%% *}"
  if [ "$canonical_mode" != '120000' ]; then
    fail "$canonical_path symlink fixture has tree mode $canonical_mode, expected 120000"
  fi
done
printf 'check-readme-status-artifacts regression: canonical-symlink-tree-modes ok\n'
run_case reverse-t27-token-change 1 setup_reverse_t27_token_change
run_case wasm-alias-t27-token-change 1 setup_wasm_alias_t27_token_change
run_case large-changed-file-list 1 setup_large_changed_file_list

# Preserve the sole historical policy/identity exception using the real
# Porffor-era generated block shape from origin/main.
git checkout -q -B legacy-t27-t29-base "$base_commit"
write_actual_legacy_readme
git add README.md
git commit -qm legacy-t27-t29-base
legacy_t27_t29_base="$(git rev-parse HEAD)"
write_migrated_legacy_readme
git add README.md
git commit -qm t27-t29-identity-and-policy-migration

set +e
t27_t29_output="$("$guard" "$legacy_t27_t29_base" 2>&1)"
t27_t29_status=$?
set -e
if [ "$t27_t29_status" -ne 0 ]; then
  fail "exact T27/T29 migration returned $t27_t29_status: $t27_t29_output"
fi
printf 'check-readme-status-artifacts regression: exact-t27-t29-migration ok\n'

run_legacy_negative() {
  local name="$1"
  local expression="$2"
  local output
  local status

  git checkout -q -B "legacy-$name" "$legacy_t27_t29_base"
  write_migrated_legacy_readme
  rewrite_readme "$expression"
  git add README.md
  git commit -qm "$name"

  set +e
  output="$("$guard" "$legacy_t27_t29_base" 2>&1)"
  status=$?
  set -e
  if [ "$status" -ne 1 ]; then
    fail "$name returned $status, expected 1: $output"
  fi
  printf 'check-readme-status-artifacts regression: %s ok\n' "$name"
}

run_legacy_negative partial-t29-identity 's#target/debug/lila #target/debug/porf #g'
run_legacy_negative wrong-t29-identity 's#crates/lila-test262#crates/lila-test263#g'
run_legacy_negative wasm-alias-policy 's/publish-real-status-low-ram.sh wasm-aot/publish-real-status-low-ram.sh wasm/'

git checkout -q -B legacy-spec-exec-artifact "$legacy_t27_t29_base"
write_migrated_legacy_readme
add_artifact test262/snapshots/published-status-spec-exec.json
git add -A
git commit -qm legacy-spec-exec-artifact
set +e
legacy_spec_output="$("$guard" "$legacy_t27_t29_base" 2>&1)"
legacy_spec_status=$?
set -e
if [ "$legacy_spec_status" -ne 1 ]; then
  fail "legacy spec-exec artifact returned $legacy_spec_status, expected 1: $legacy_spec_output"
fi
printf 'check-readme-status-artifacts regression: legacy-spec-exec-artifact ok\n'

# Exercise the real repository comparison. At the migration boundary this has
# a changed-file list large enough to reproduce the former pipefail/SIGPIPE
# false negative; after the boundary lands it remains a normal smoke check.
set +e
actual_output="$(cd "$repo_root" && "$guard" origin/main 2>&1)"
actual_status=$?
set -e
if [ "$actual_status" -ne 0 ]; then
  fail "actual origin/main comparison returned $actual_status: $actual_output"
fi
origin_main_readme="$(git -C "$repo_root" show origin/main:README.md)"
if grep -Fq '<!-- porffor-status:start -->' <<<"$origin_main_readme" \
  && ! grep -Fq 'exact T27/T29 identity and wasm-aot policy migration' <<<"$actual_output"; then
  fail "actual legacy origin/main comparison did not take the exact T27/T29 migration route: $actual_output"
fi
printf 'check-readme-status-artifacts regression: actual-origin-main-comparison ok\n'

printf 'check-readme-status-artifacts regression: all cases passed\n'
