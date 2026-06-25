#!/usr/bin/env bash
set -euo pipefail

mode="report"
allowlist="test262/backlog/shortcut-allowlist.tsv"
target="crates/porffor-test262/src/lib.rs"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --check)
      mode="check"
      shift
      ;;
    --allowlist)
      allowlist="${2:?--allowlist needs a path}"
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

count_matches() {
  pattern="$1"
  matches="$(grep -nE "$pattern" "$target" || true)"
  if [ -z "$matches" ]; then
    printf '0\n'
  else
    printf '%s\n' "$matches" | wc -l | tr -d ' '
  fi
}

pattern_for_key() {
  case "$1" in
    path_rewrite_entrypoints)
      printf '%s\n' 'rewrite_[A-Za-z0-9_]+\(&case\.path\)'
      ;;
    direct_path_predicates)
      printf '%s\n' '(case\.path|path)\.(starts_with|ends_with|contains|as_str\(\))|path == "built-ins|match path'
      ;;
    source_text_predicates)
      printf '%s\n' '(case\.)?original_source\.(contains|replace)|source\.contains'
      ;;
    harness_helper_reductions)
      printf '%s\n' 'prelude\.contents|used_preludes|helper used|assert\.sameValue = function|assert\.throws = function|skips_test_typed_array'
      ;;
    *)
      return 1
      ;;
  esac
}

valid_task_id() {
  case "$1" in
    T[0-2][0-9]|T26-unclassified) ;;
    *) return 1 ;;
  esac
  case "$1" in
    T27|T28|T29) return 1 ;;
  esac
}

if [ "$mode" = "check" ]; then
  if [ ! -f "$allowlist" ]; then
    printf 'audit-test262-shortcuts: missing allowlist: %s\n' "$allowlist" >&2
    exit 1
  fi
  failures=0
  seen=0
  seen_path_rewrite_entrypoints=0
  seen_direct_path_predicates=0
  seen_source_text_predicates=0
  seen_harness_helper_reductions=0
  while IFS=$'\t' read -r key max_count owner_task removal_task reason; do
    case "$key" in
      ''|\#*) continue ;;
    esac
    seen=$((seen + 1))
    case "$key" in
      path_rewrite_entrypoints) seen_path_rewrite_entrypoints=1 ;;
      direct_path_predicates) seen_direct_path_predicates=1 ;;
      source_text_predicates) seen_source_text_predicates=1 ;;
      harness_helper_reductions) seen_harness_helper_reductions=1 ;;
    esac
    pattern="$(pattern_for_key "$key" || true)"
    if [ -z "$pattern" ]; then
      printf 'audit-test262-shortcuts: unknown allowlist key: %s\n' "$key" >&2
      failures=$((failures + 1))
      continue
    fi
    if ! printf '%s\n' "$max_count" | grep -Eq '^[0-9]+$'; then
      printf 'audit-test262-shortcuts: %s has non-numeric max_count: %s\n' "$key" "$max_count" >&2
      failures=$((failures + 1))
      continue
    fi
    if ! valid_task_id "$owner_task"; then
      printf 'audit-test262-shortcuts: %s has invalid owner task: %s\n' "$key" "$owner_task" >&2
      failures=$((failures + 1))
    fi
    if ! valid_task_id "$removal_task"; then
      printf 'audit-test262-shortcuts: %s has invalid removal task: %s\n' "$key" "$removal_task" >&2
      failures=$((failures + 1))
    fi
    if [ -z "$reason" ]; then
      printf 'audit-test262-shortcuts: %s is missing a reason\n' "$key" >&2
      failures=$((failures + 1))
    fi
    current_count="$(count_matches "$pattern")"
    if [ "$current_count" -gt "$max_count" ]; then
      printf 'audit-test262-shortcuts: %s count grew from allowlisted max %s to %s\n' \
        "$key" "$max_count" "$current_count" >&2
      failures=$((failures + 1))
    fi
  done < "$allowlist"
  if [ "$seen" -eq 0 ]; then
    printf 'audit-test262-shortcuts: allowlist is empty: %s\n' "$allowlist" >&2
    exit 1
  fi
  for required_key in \
    path_rewrite_entrypoints \
    direct_path_predicates \
    source_text_predicates \
    harness_helper_reductions
  do
    seen_var="seen_${required_key}"
    if [ "${!seen_var}" -eq 0 ]; then
      printf 'audit-test262-shortcuts: allowlist is missing required key: %s\n' \
        "$required_key" >&2
      failures=$((failures + 1))
    fi
  done
  if [ "$failures" -ne 0 ]; then
    exit 1
  fi
  printf 'audit-test262-shortcuts: ok\n'
  exit 0
fi

emit_section() {
  title="$1"
  pattern="$2"
  printf '\n## %s\n\n' "$title"
  matches="$(grep -nE "$pattern" "$target" || true)"
  if [ -z "$matches" ]; then
    printf 'Count: 0\n'
    return
  fi
  count="$(printf '%s\n' "$matches" | wc -l | tr -d ' ')"
  printf 'Count: %s\n\n' "$count"
  printf '```text\n'
  printf '%s\n' "$matches"
  printf '```\n'
}

printf '# Test262 Shortcut Inventory\n'
printf '\nSource: `%s`\n' "$target"
printf '\nThis report is deterministic and intentionally mechanical. Classify each match as a legitimate harness adaptation, temporary diagnostic instrumentation, or semantic shortcut before closing T03.\n'

emit_section \
  'Path-Based Rewrite Entrypoints' \
  'rewrite_[A-Za-z0-9_]+\(&case\.path\)'

emit_section \
  'Direct Path Predicates' \
  '(case\.path|path)\.(starts_with|ends_with|contains|as_str\(\))|path == "built-ins|match path'

emit_section \
  'Source-Text Predicates' \
  '(case\.)?original_source\.(contains|replace)|source\.contains'

emit_section \
  'Harness And Helper Reductions' \
  'prelude\.contents|used_preludes|helper used|assert\.sameValue = function|assert\.throws = function|skips_test_typed_array'
