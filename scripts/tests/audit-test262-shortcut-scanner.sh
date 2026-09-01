#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
scanner_source="$repo_root/scripts/audit-test262-shortcuts.rs"
audit_driver="$repo_root/scripts/audit-test262-shortcuts.sh"
tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

fail() {
  printf 'audit-test262 shortcut scanner regression: %s\n' "$*" >&2
  exit 1
}

if [ ! -f "$scanner_source" ]; then
  fail "missing scanner source: $scanner_source"
fi
if [ ! -x "$audit_driver" ]; then
  fail "missing executable audit driver: $audit_driver"
fi

scanner="$tmp_dir/audit-test262-shortcuts"
scanner_tests="$tmp_dir/audit-test262-shortcuts-tests"
fixture="$tmp_dir/shortcut-observations.rs"
actual="$tmp_dir/actual.tsv"
driver_observations="$tmp_dir/driver-observations.tsv"
ledger="$tmp_dir/shortcut-allowlist.tsv"
inventory="$tmp_dir/shortcut-inventory.md"
drift_stdout="$tmp_dir/drift.stdout"
drift_stderr="$tmp_dir/drift.stderr"

rustfmt --edition 2021 --check "$scanner_source"
rustc --edition=2021 -D warnings --test "$scanner_source" -o "$scanner_tests"
"$scanner_tests"
rustc --edition=2021 -D warnings "$scanner_source" -o "$scanner"

cat > "$fixture" <<'RUST'
struct FixtureCase {
    path: String,
    original_source: String,
}

struct Failure {
    test_path: String,
}

struct Artifact {
    path: String,
}

fn rewrite_fixture(_: &String) -> Option<()> {
    None
}

fn first_anchor(
    case: &FixtureCase,
    path: &str,
    source: &str,
    failure: &Failure,
    artifact: &Artifact,
) {
    let _inline_equivalent = path.ends_with("built-ins/equivalent.js");
    let _multiline_equivalent = path
        .ends_with("built-ins/equivalent.js");
    let _same_line = path.starts_with("built-ins/first.js") || path.contains("same-line");
    let _rewrite = rewrite_fixture(
        &case
            .path
    );
    let _equality = path
        == "built-ins/equality.js";
    let _match = match
        path {
        _ => (),
    };
    let _contains = case.original_source.contains("needle");
    let _replace = case
        .original_source
        .replace(
            "old",
            "new",
        );
    let _source_contains = source.contains("generic-needle");
    let _source_replace = source.replace("generic-old", "generic-new");

    let _normal_string_decoy = "path.ends_with(\"built-ins/normal-string-decoy.js\")";
    let _raw_string_decoy = r#"
case
    .original_source
    .replace("raw", "string");
path.contains("built-ins/raw-string-decoy.js");
"#;

    /*
    path.ends_with("built-ins/block-comment-decoy.js");
    /* case.original_source.contains("nested-block-comment-decoy"); */
    */
    // source.contains("line-comment-decoy");

    let _failure_path = failure.test_path.ends_with("built-ins/failure-path-negative.js");
    let _artifact_path = artifact.path.contains("built-ins/artifact-path-negative.js");
    let _source_name_control = source.len();
}

fn second_anchor(path: &str) {
    let _second = path.starts_with("built-ins/second-anchor.js");
}

fn marker() {
    prelude.contents;
}

#[cfg(test)]
mod tests {
    fn excluded(case: &super::FixtureCase, path: &str) {
        let _excluded_path = path.ends_with("built-ins/excluded-path.js");
        let _excluded_source = case.original_source.replace("excluded", "source");
        let _excluded_rewrite = super::rewrite_fixture(&case.path);
    }
}
RUST

"$scanner" "$fixture" > "$actual"

if ! awk -F '\t' '
  NF != 5 || $1 == "" || $2 !~ /^[0-9]+$/ || $3 == "" || $4 == "" || $5 == "" {
    exit 1
  }
' "$actual"; then
  fail "scanner output is not key/start-line/anchor/category/escaped-evidence TSV"
fi

observation_count="$(wc -l < "$actual" | tr -d '[:space:]')"
if [ "$observation_count" -ne 13 ]; then
  fail "scanner emitted $observation_count observations, expected 13: $(tr '\n' '|' < "$actual")"
fi

duplicate_keys="$(cut -f 1 "$actual" | sort | uniq -d)"
if [ -n "$duplicate_keys" ]; then
  fail "scanner emitted duplicate stable keys: $duplicate_keys"
fi

line_of() {
  local marker="$1"
  local line

  line="$(awk -v marker="$marker" 'index($0, marker) { print NR; exit }' "$fixture")"
  if [ -z "$line" ]; then
    fail "fixture marker not found: $marker"
  fi
  printf '%s' "$line"
}

assert_observation() {
  local key="$1"
  local line="$2"
  local anchor="$3"
  local category="$4"
  local evidence="$5"
  local expected
  local matches

  expected="$(printf '%s\t%s\t%s\t%s\t%s' "$key" "$line" "$anchor" "$category" "$evidence")"
  matches="$(grep -Fxc -- "$expected" "$actual")" || matches=0
  if [ "$matches" -ne 1 ]; then
    fail "expected one row [$expected], found $matches: $(tr '\n' '|' < "$actual")"
  fi
}

run_audit_driver() {
  (
    cd "$repo_root"
    ./scripts/audit-test262-shortcuts.sh "$@"
  )
}

inline_line="$(line_of 'let _inline_equivalent')"
multiline_line="$(line_of 'let _multiline_equivalent')"
same_line="$(line_of 'let _same_line')"
rewrite_line="$(line_of 'let _rewrite')"
equality_line="$(line_of 'let _equality')"
match_line="$(line_of 'let _match')"
contains_line="$(line_of 'let _contains')"
replace_line="$(line_of 'let _replace')"
source_contains_line="$(line_of 'let _source_contains')"
source_replace_line="$(line_of 'let _source_replace')"
second_anchor_line="$(line_of 'let _second')"
marker_line="$(line_of '    prelude.contents;')"

assert_observation \
  'direct-path-predicate/first_anchor/001' \
  "$inline_line" \
  'first_anchor' \
  'direct-path-predicate' \
  'path.ends_with("built-ins/equivalent.js")'
assert_observation \
  'direct-path-predicate/first_anchor/002' \
  "$multiline_line" \
  'first_anchor' \
  'direct-path-predicate' \
  'path\n        .ends_with("built-ins/equivalent.js")'
assert_observation \
  'direct-path-predicate/first_anchor/003' \
  "$same_line" \
  'first_anchor' \
  'direct-path-predicate' \
  'path.starts_with("built-ins/first.js")'
assert_observation \
  'direct-path-predicate/first_anchor/004' \
  "$same_line" \
  'first_anchor' \
  'direct-path-predicate' \
  'path.contains("same-line")'
assert_observation \
  'path-rewrite-entrypoint/rewrite_fixture/001' \
  "$rewrite_line" \
  'first_anchor' \
  'path-rewrite-entrypoint' \
  'rewrite_fixture(\n        &case\n            .path\n    )'
assert_observation \
  'direct-path-predicate/first_anchor/005' \
  "$equality_line" \
  'first_anchor' \
  'direct-path-predicate' \
  'path\n        == "built-ins/equality.js"'
assert_observation \
  'direct-path-predicate/first_anchor/006' \
  "$match_line" \
  'first_anchor' \
  'direct-path-predicate' \
  'match\n        path {\n  _ => <body>\n}'
assert_observation \
  'source-text-predicate/first_anchor/001' \
  "$contains_line" \
  'first_anchor' \
  'source-text-predicate' \
  'case.original_source.contains("needle")'
assert_observation \
  'source-text-predicate/first_anchor/002' \
  "$replace_line" \
  'first_anchor' \
  'source-text-predicate' \
  'case\n        .original_source\n        .replace(\n            "old",\n            "new",\n        )'
assert_observation \
  'source-text-predicate/first_anchor/003' \
  "$source_contains_line" \
  'first_anchor' \
  'source-text-predicate' \
  'source.contains("generic-needle")'
assert_observation \
  'source-text-predicate/first_anchor/004' \
  "$source_replace_line" \
  'first_anchor' \
  'source-text-predicate' \
  'source.replace("generic-old", "generic-new")'
assert_observation \
  'direct-path-predicate/second_anchor/001' \
  "$second_anchor_line" \
  'second_anchor' \
  'direct-path-predicate' \
  'path.starts_with("built-ins/second-anchor.js")'
assert_observation \
  'harness-helper-reduction/marker/001' \
  "$marker_line" \
  'marker' \
  'harness-helper-reduction' \
  '    prelude.contents;'

for decoy in \
  normal-string-decoy \
  raw-string-decoy \
  block-comment-decoy \
  nested-block-comment-decoy \
  line-comment-decoy \
  failure-path-negative \
  artifact-path-negative \
  excluded-path \
  excluded-source \
  excluded-rewrite; do
  if grep -Fq -- "$decoy" "$actual"; then
    fail "scanner emitted decoy or excluded observation: $decoy"
  fi
done

run_audit_driver --observations --target "$fixture" > "$driver_observations"
expected_observation_header="$(printf '# key\tfingerprint\tcategory\tline\tanchor\tevidence')"
if [ "$(head -n 1 "$driver_observations")" != "$expected_observation_header" ]; then
  fail "audit driver emitted an unexpected observation header"
fi

if ! awk -F '\t' '
  BEGIN {
    OFS = "\t"
    print "# key", "fingerprint", "classification", "owner", "removal", "reason"
  }
  NR == 1 { next }
  NF != 6 { exit 1 }
  {
    print $1, $2, "semantic-shortcut", "T03", "T03", "source-shape-rewrite"
  }
' "$driver_observations" > "$ledger"; then
  fail "could not derive a six-field ledger from audit-driver observations"
fi

ledger_entry_count="$(awk -F '\t' '$1 !~ /^#/ && $1 != "" { count += 1 } END { print count + 0 }' "$ledger")"
if [ "$ledger_entry_count" -ne "$observation_count" ]; then
  fail "derived ledger has $ledger_entry_count entries, expected $observation_count"
fi

run_audit_driver --ledger "$ledger" --target "$fixture" > "$inventory"
clean_check_output="$(
  run_audit_driver \
    --check \
    --ledger "$ledger" \
    --inventory "$inventory" \
    --target "$fixture"
)"
expected_check_output="audit-test262-shortcuts: ok ($observation_count exact entries)"
if [ "$clean_check_output" != "$expected_check_output" ]; then
  fail "audit-driver check returned unexpected output: $clean_check_output"
fi

drifted_fixture="$tmp_dir/drifted-shortcut-observations.rs"
sed 's/source.contains("generic-needle")/source.contains("drifted-needle")/' \
  "$fixture" > "$drifted_fixture"
mv "$drifted_fixture" "$fixture"
if grep -Fq 'source.contains("generic-needle")' "$fixture" \
  || ! grep -Fq 'source.contains("drifted-needle")' "$fixture"; then
  fail "could not change the observed source literal"
fi

if run_audit_driver \
  --check \
  --ledger "$ledger" \
  --inventory "$inventory" \
  --target "$fixture" \
  > "$drift_stdout" 2> "$drift_stderr"; then
  fail "audit-driver check accepted a drifted source fingerprint"
fi
if [ -s "$drift_stdout" ]; then
  fail "drifted audit-driver check unexpectedly wrote stdout: $(tr '\n' '|' < "$drift_stdout")"
fi

drift_message="$(< "$drift_stderr")"
expected_drift_message="$(printf '%s\n%s' \
  'audit-test262-shortcuts: selector fingerprints drifted:' \
  'source-text-predicate/first_anchor/003')"
if [ "$drift_message" != "$expected_drift_message" ]; then
  fail "audit-driver check did not report only the expected fingerprint drift: $(tr '\n' '|' < "$drift_stderr")"
fi

printf 'audit-test262 shortcut scanner regression: ok (%s observations and ledger drift)\n' \
  "$observation_count"
