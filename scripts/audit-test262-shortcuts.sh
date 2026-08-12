#!/usr/bin/env bash
set -euo pipefail

mode="report"
ledger="test262/backlog/shortcut-allowlist.tsv"
inventory="test262/backlog/shortcut-inventory.md"
target="crates/lila-test262/src/lib.rs"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --check)
      mode="check"
      shift
      ;;
    --observations)
      mode="observations"
      shift
      ;;
    --allowlist|--ledger)
      ledger="${2:?$1 needs a path}"
      shift 2
      ;;
    --inventory)
      inventory="${2:?--inventory needs a path}"
      shift 2
      ;;
    --target)
      target="${2:?--target needs a path}"
      shift 2
      ;;
    *)
      target="$1"
      shift
      ;;
  esac
done

if [ ! -f "$target" ]; then
  printf 'audit-test262-shortcuts: missing source file: %s\n' "$target" >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  sha256() {
    sha256sum | cut -d ' ' -f 1
  }
elif command -v shasum >/dev/null 2>&1; then
  sha256() {
    shasum -a 256 | cut -d ' ' -f 1
  }
else
  printf 'audit-test262-shortcuts: sha256sum or shasum is required\n' >&2
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

raw_observations="$tmp_dir/observations.raw.tsv"
observations="$tmp_dir/observations.tsv"
sorted_observations="$tmp_dir/observations.sorted.tsv"
sorted_ledger="$tmp_dir/ledger.sorted.tsv"
joined="$tmp_dir/joined.tsv"

# Emit one record for every production occurrence. Rewrite entrypoints use the
# called rewrite function as their stable identity, so deleting one entrypoint
# does not renumber every later row in the dispatcher. Other observations use
# their enclosing Rust declaration and ordinal. Source lines are display-only;
# the SHA-256 fingerprint remains the drift identity.
awk '
BEGIN {
  anchor = "module"
  production = 1
}

/^mod tests \{/ {
  production = 0
}

!production {
  next
}

/^[[:space:]]*(pub(\([^)]*\))?[[:space:]]+)?((const|async|unsafe)[[:space:]]+)*fn[[:space:]]+[A-Za-z0-9_]+/ {
  declaration = $0
  sub(/^.*fn[[:space:]]+/, "", declaration)
  sub(/[^A-Za-z0-9_].*$/, "", declaration)
  anchor = declaration
}

/^(pub(\([^)]*\))?[[:space:]]+)?const[[:space:]]+[A-Za-z0-9_]+[[:space:]]*:/ {
  declaration = $0
  sub(/^.*const[[:space:]]+/, "", declaration)
  sub(/[^A-Za-z0-9_].*$/, "", declaration)
  anchor = declaration
}

/^(pub(\([^)]*\))?[[:space:]]+)?(struct|enum|union|trait|type)[[:space:]]+[A-Za-z0-9_]+/ {
  declaration = $0
  sub(/^.*(struct|enum|union|trait|type)[[:space:]]+/, "", declaration)
  sub(/[^A-Za-z0-9_].*$/, "", declaration)
  anchor = declaration
}

function emit(category, source, base, key) {
  source = $0
  gsub(/\t/, "\\t", source)
  base = category "/" anchor
  count[base] += 1
  key = sprintf("%s/%03d", base, count[base])
  print key "\t" NR "\t" anchor "\t" category "\t" source
}

function emit_named(category, identity, source, base, key) {
  source = $0
  gsub(/\t/, "\\t", source)
  base = category "/" identity
  count[base] += 1
  key = sprintf("%s/%03d", base, count[base])
  print key "\t" NR "\t" anchor "\t" category "\t" source
}

{
  if ($0 ~ /rewrite_[A-Za-z0-9_]+\(&case\.path\)/) {
    entrypoint = $0
    sub(/^.*rewrite_/, "rewrite_", entrypoint)
    sub(/\(.*/, "", entrypoint)
    emit_named("path-rewrite-entrypoint", entrypoint)
  }
  if ($0 ~ /(case\.path|path)\.(starts_with|ends_with|contains|as_str\(\))/ ||
      $0 ~ /path == "built-ins/ || $0 ~ /match path/) {
    emit("direct-path-predicate")
  }
  if ($0 ~ /(case\.)?original_source\.(contains|replace)/ ||
      $0 ~ /source\.contains/) {
    emit("source-text-predicate")
  }
  if ($0 ~ /prelude\.contents|used_preludes|helper used|assert\.sameValue = function|assert\.throws = function|skips_test_typed_array/) {
    emit("harness-helper-reduction")
  }
}
' "$target" > "$raw_observations"

while IFS=$'\t' read -r key line anchor category source; do
  fingerprint="$(printf '%s\n%s\n%s\n' "$category" "$anchor" "$source" | sha256)"
  printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$key" "$fingerprint" "$category" "$line" "$anchor" "$source"
done < "$raw_observations" > "$observations"

if [ "$mode" = "observations" ]; then
  printf '# key\tfingerprint\tcategory\tline\tanchor\tsource\n'
  cat "$observations"
  exit 0
fi

if [ ! -f "$ledger" ]; then
  printf 'audit-test262-shortcuts: missing ledger: %s\n' "$ledger" >&2
  exit 1
fi

failures=0
entries=0

while IFS=$'\t' read -r key fingerprint classification owner_task removal_task reason_code extra; do
  case "$key" in
    ''|\#*) continue ;;
  esac
  entries=$((entries + 1))
  if [ -n "${extra:-}" ]; then
    printf 'audit-test262-shortcuts: %s has more than six TSV fields\n' "$key" >&2
    failures=$((failures + 1))
  fi
  if ! printf '%s\n' "$key" | grep -Eq '^(path-rewrite-entrypoint|direct-path-predicate|source-text-predicate|harness-helper-reduction)/[A-Za-z0-9_]+/[0-9]{3}$'; then
    printf 'audit-test262-shortcuts: invalid stable key: %s\n' "$key" >&2
    failures=$((failures + 1))
  fi
  if ! printf '%s\n' "$fingerprint" | grep -Eq '^[0-9a-f]{64}$'; then
    printf 'audit-test262-shortcuts: %s has invalid SHA-256 fingerprint: %s\n' \
      "$key" "$fingerprint" >&2
    failures=$((failures + 1))
  fi
  case "$classification" in
    legitimate-harness-adaptation|diagnostic-instrumentation|semantic-shortcut) ;;
    *)
      printf 'audit-test262-shortcuts: %s has invalid classification: %s\n' \
        "$key" "$classification" >&2
      failures=$((failures + 1))
      ;;
  esac
  for task in "$owner_task" "$removal_task"; do
    if ! printf '%s\n' "$task" | grep -Eq '^T[0-2][0-9]$'; then
      printf 'audit-test262-shortcuts: %s has invalid concrete task id: %s\n' \
        "$key" "$task" >&2
      failures=$((failures + 1))
    fi
  done
  case "$classification:$reason_code" in
    legitimate-harness-adaptation:upstream-harness-materialization \
      |legitimate-harness-adaptation:suite-selection \
      |diagnostic-instrumentation:artifact-routing \
      |diagnostic-instrumentation:unsupported-feature-routing \
      |diagnostic-instrumentation:vendored-contract-guard \
      |semantic-shortcut:test-specific-materialization \
      |semantic-shortcut:reduced-harness-helper \
      |semantic-shortcut:source-shape-rewrite \
      |semantic-shortcut:dynamic-source-substitution) ;;
    *)
      printf 'audit-test262-shortcuts: %s has invalid classification/reason pair: %s/%s\n' \
        "$key" "$classification" "$reason_code" >&2
      failures=$((failures + 1))
      ;;
  esac
done < "$ledger"

if [ "$entries" -eq 0 ]; then
  printf 'audit-test262-shortcuts: ledger is empty: %s\n' "$ledger" >&2
  exit 1
fi

awk -F '\t' '$1 !~ /^#/ && $1 != "" { print }' "$ledger" \
  | LC_ALL=C sort -t $'\t' -k1,1 > "$sorted_ledger"
LC_ALL=C sort -t $'\t' -k1,1 "$observations" > "$sorted_observations"

duplicate_ledger_keys="$(cut -f 1 "$sorted_ledger" | uniq -d)"
if [ -n "$duplicate_ledger_keys" ]; then
  printf 'audit-test262-shortcuts: duplicate ledger keys:\n%s\n' \
    "$duplicate_ledger_keys" >&2
  failures=$((failures + 1))
fi

duplicate_observation_keys="$(cut -f 1 "$sorted_observations" | uniq -d)"
if [ -n "$duplicate_observation_keys" ]; then
  printf 'audit-test262-shortcuts: duplicate generated keys:\n%s\n' \
    "$duplicate_observation_keys" >&2
  failures=$((failures + 1))
fi

cut -f 1 "$sorted_observations" > "$tmp_dir/observation.keys"
cut -f 1 "$sorted_ledger" > "$tmp_dir/ledger.keys"

new_entries="$(comm -23 "$tmp_dir/observation.keys" "$tmp_dir/ledger.keys")"
if [ -n "$new_entries" ]; then
  printf 'audit-test262-shortcuts: new source observations need classification:\n%s\n' \
    "$new_entries" >&2
  failures=$((failures + 1))
fi

missing_entries="$(comm -13 "$tmp_dir/observation.keys" "$tmp_dir/ledger.keys")"
if [ -n "$missing_entries" ]; then
  printf 'audit-test262-shortcuts: ledger entries disappeared from the source:\n%s\n' \
    "$missing_entries" >&2
  failures=$((failures + 1))
fi

if [ -z "$new_entries" ] && [ -z "$missing_entries" ] \
  && [ -z "$duplicate_ledger_keys" ] && [ -z "$duplicate_observation_keys" ]; then
  join -t $'\t' -1 1 -2 1 "$sorted_observations" "$sorted_ledger" > "$joined"
  drifted_entries="$(awk -F '\t' '$2 != $7 { print $1 }' "$joined")"
  if [ -n "$drifted_entries" ]; then
    printf 'audit-test262-shortcuts: source fingerprints drifted:\n%s\n' \
      "$drifted_entries" >&2
    failures=$((failures + 1))
  fi
fi

if [ "$failures" -ne 0 ]; then
  exit 1
fi

render_inventory() {
  printf '# Test262 Shortcut Inventory\n'
  printf '\nSource: `%s`\n' "$target"
  printf '\nLedger: `%s`\n' "$ledger"
  printf '\nThis file is generated by `scripts/audit-test262-shortcuts.sh`. '
  printf 'Rewrite-entrypoint keys use the called function; other stable keys use the '
  printf 'enclosing Rust declaration and an occurrence ordinal; '
  printf 'line numbers are display-only. A SHA-256 fingerprint covers the category, '
  printf 'declaration and exact matched source line, so source drift cannot inherit an '
  printf 'old classification. Test-only assertions are excluded.\n'

  printf '\n## Classification summary\n\n'
  printf '| Classification | Count |\n'
  printf '| --- | ---: |\n'
  for classification in \
    legitimate-harness-adaptation \
    diagnostic-instrumentation \
    semantic-shortcut
  do
    count="$(awk -F '\t' -v value="$classification" '$8 == value { count += 1 } END { print count + 0 }' "$joined")"
    printf '| `%s` | %s |\n' "$classification" "$count"
  done

  printf '\n## Removal-task summary\n\n'
  printf '| Task | Count |\n'
  printf '| --- | ---: |\n'
  awk -F '\t' '{ count[$10] += 1 } END { for (task in count) print task "\t" count[task] }' "$joined" \
    | LC_ALL=C sort \
    | while IFS=$'\t' read -r task count; do
        printf '| `%s` | %s |\n' "$task" "$count"
      done

  printf '\n## Reason codes\n\n'
  printf '| Code | Meaning |\n'
  printf '| --- | --- |\n'
  printf '| `upstream-harness-materialization` | Loads, combines or records the upstream Test262 shell contract without supplying ECMAScript product semantics. |\n'
  printf '| `suite-selection` | Selects a requested suite path or case boundary without changing the selected case. |\n'
  printf '| `artifact-routing` | Gives manifests, snapshots or failure records deterministic identity and ownership; it does not change execution. |\n'
  printf '| `unsupported-feature-routing` | Reports a known unsupported feature explicitly instead of executing or counting it as a pass. |\n'
  printf '| `vendored-contract-guard` | Pins vendored source before a narrower transformation can be applied; the owning transformation still has to be removed. |\n'
  printf '| `test-specific-materialization` | Replaces a named Test262 case with handwritten source instead of exercising the general compiler/runtime path. |\n'
  printf '| `reduced-harness-helper` | Selects, omits or rewrites a Test262 helper from path/source knowledge rather than compiling the full helper contract. |\n'
  printf '| `source-shape-rewrite` | Recognizes source text or syntax and substitutes a compiler-friendly shape. |\n'
  printf '| `dynamic-source-substitution` | Replaces dynamic source generation with static source for selected harness/cases. |\n'

  printf '\n## Exact entries\n\n'
  printf '| Stable key | Fingerprint | Classification | Owner | Removal | Line | Reason code | Matched source |\n'
  printf '| --- | --- | --- | --- | --- | ---: | --- | --- |\n'
  LC_ALL=C sort -t $'\t' -k4,4n -k1,1 "$joined" \
    | awk -F '\t' '
      function escape(value) {
        gsub(/\\/, "\\\\", value)
        gsub(/\|/, "\\|", value)
        gsub(/`/, "\\`", value)
        return value
      }
      {
        printf "| `%s` | `%s` | `%s` | `%s` | `%s` | %s | %s | `%s` |\n", \
          $1, substr($2, 1, 16), $8, $9, $10, $4, escape($11), escape($6)
      }
    '
}

if [ "$mode" = "report" ]; then
  render_inventory
  exit 0
fi

if [ ! -f "$inventory" ]; then
  printf 'audit-test262-shortcuts: missing inventory: %s\n' "$inventory" >&2
  exit 1
fi

generated_inventory="$tmp_dir/shortcut-inventory.md"
render_inventory > "$generated_inventory"
if ! cmp -s "$generated_inventory" "$inventory"; then
  printf 'audit-test262-shortcuts: inventory is stale: %s\n' "$inventory" >&2
  printf 'regenerate it with: %s --target %s > %s\n' \
    "$0" "$target" "$inventory" >&2
  exit 1
fi

printf 'audit-test262-shortcuts: ok (%s exact entries)\n' "$entries"
