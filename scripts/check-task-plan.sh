#!/usr/bin/env bash
set -euo pipefail

tasks_dir="${TASKS_DIR:-tasks}"
readme_path="${README_PATH:-$tasks_dir/README.md}"

failures=0

fail() {
  printf 'check-task-plan: %s\n' "$*" >&2
  failures=$((failures + 1))
}

require_file() {
  if [ ! -f "$1" ]; then
    fail "missing file: $1"
    return 1
  fi
}

require_file "$readme_path" || exit 1
require_file "$tasks_dir/00-operating-contract.md" || exit 1

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT

ids_file="$tmp_dir/ids"
: > "$ids_file"

while IFS= read -r file; do
  base="$(basename "$file")"
  case "$base" in
    [0-9][0-9]-*.md) ;;
    README.md) continue ;;
    *) fail "unexpected task filename: $file"; continue ;;
  esac

  num="${base%%-*}"
  id="T$num"
  printf '%s %s\n' "$id" "$file" >> "$ids_file"

  first_heading="$(sed -n '1s/^# *//p' "$file")"
  case "$first_heading" in
    "$id — "*|"$id - "*) ;;
    *) fail "$file first heading must start with '$id — ' or '$id - '" ;;
  esac

  status_count="$(grep -c '^\*\*Status:\*\*' "$file" || true)"
  [ "$status_count" -eq 1 ] \
    || fail "$file must contain exactly one **Status:** field"
  grep -q '^\*\*Status:\*\*[[:space:]]*[^[:space:]]' "$file" \
    || fail "$file is missing a non-empty **Status:** field"
  depends_count="$(grep -c '^\*\*Depends on:\*\*' "$file" || true)"
  [ "$depends_count" -eq 1 ] \
    || fail "$file must contain exactly one **Depends on:** field"
  grep -q '^\*\*Depends on:\*\*[[:space:]]*' "$file" \
    || fail "$file is missing a **Depends on:** field"
  grep -q '^## Objective[[:space:]]*$' "$file" \
    || fail "$file is missing a ## Objective section"
  grep -Eq '^## (Acceptance criteria|Deliverables|Integrity requirements)[[:space:]]*$' "$file" \
    || fail "$file is missing acceptance criteria, deliverables, or integrity requirements"
  grep -Eq '^## (Required tests|Required final commands|Test instructions|Required validation)[[:space:]]*$' "$file" \
    || fail "$file is missing required test instructions"
done < <(find "$tasks_dir" -maxdepth 1 -type f -name '*.md' | sort)

if [ ! -s "$ids_file" ]; then
  fail "no task files found in $tasks_dir"
fi

while IFS= read -r duplicate; do
  [ -n "$duplicate" ] && fail "duplicate task ID: $duplicate"
done < <(sort "$ids_file" | awk '
  previous == $1 { print $1 }
  { previous = $1 }
')

task_ids="$(cut -d' ' -f1 "$ids_file" | tr '\n' ' ')"

while IFS= read -r file; do
  deps_line="$(sed -n 's/^\*\*Depends on:\*\*[[:space:]]*//p' "$file" | head -n 1)"
  [ -n "$deps_line" ] || continue
  case "$deps_line" in
    None|none|N/A|n/a) continue ;;
  esac

  while IFS= read -r dep; do
    [ -n "$dep" ] || continue
    case " $task_ids " in
      *" $dep "*) ;;
      *) fail "$file dependency points to missing task: $dep" ;;
    esac
  done < <(printf '%s\n' "$deps_line" | grep -o 'T[0-9][0-9]' || true)
done < <(find "$tasks_dir" -maxdepth 1 -type f -name '[0-9][0-9]-*.md' | sort)

while IFS= read -r target; do
  [ -n "$target" ] || continue
  case "$target" in
    http://*|https://*|'#'*) continue ;;
  esac
  path="${target%%#*}"
  [ -n "$path" ] || continue
  if [ ! -e "$tasks_dir/$path" ]; then
    fail "$readme_path has broken task link target: $target"
  fi
done < <(grep -oE '\[[^]]+\]\([^)]+' "$readme_path" | sed 's/.*(//' | grep -E '(^|/)[0-9][0-9]-[^)]*\.md' || true)

if [ "$failures" -ne 0 ]; then
  exit 1
fi

printf 'check-task-plan: ok\n'
