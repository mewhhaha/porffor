#!/usr/bin/env bash
set -euo pipefail

contract="${1:-test262/backlog/host-abi.tsv}"

failures=0

fail() {
  printf 'check-test262-host-abi: %s\n' "$*" >&2
  failures=$((failures + 1))
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

valid_failure_class() {
  case "$1" in
    HostHarness|Unsupported) return 0 ;;
    *) return 1 ;;
  esac
}

required_operations() {
  cat <<'EOF'
global
getGlobal
createRealm
realm.evalScript
realm.destroy
detachArrayBuffer
gc
IsHTMLDDA
AbstractModuleSource
agent.start
agent.broadcast
agent.receiveBroadcast
agent.report
agent.getReport
agent.sleep
agent.leaving
agent.monotonicNow
async.$DONE
EOF
}

if [ ! -f "$contract" ]; then
  fail "missing contract: $contract"
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
seen_ops="$tmp_dir/seen-ops"
: > "$seen_ops"

line_no=0
record_count=0
while IFS= read -r line; do
  line_no=$((line_no + 1))
  case "$line" in
    ''|\#*) continue ;;
  esac

  field_count="$(printf '%s\n' "$line" | awk -F '\t' '{print NF}')"
  if [ "$field_count" -ne 7 ]; then
    fail "$contract:$line_no expected 7 tab-separated fields, found $field_count"
    continue
  fi

  IFS=$'\t' read -r operation surface spec_exec wasm_aot failure_class owner_task notes <<< "$line"
  record_count=$((record_count + 1))

  for field_name in operation surface spec_exec wasm_aot failure_class owner_task notes; do
    field_value="${!field_name}"
    if [ -z "$field_value" ]; then
      fail "$contract:$line_no has empty $field_name"
    fi
  done

  if ! valid_failure_class "$failure_class"; then
    fail "$contract:$line_no has invalid failure class: $failure_class"
  fi
  if ! valid_task_id "$owner_task"; then
    fail "$contract:$line_no has invalid owner task: $owner_task"
  fi
  if grep -Fxq "$operation" "$seen_ops"; then
    fail "$contract:$line_no duplicates operation: $operation"
  fi
  printf '%s\n' "$operation" >> "$seen_ops"
done < "$contract"

if [ "$record_count" -eq 0 ]; then
  fail "$contract has no ABI rows"
fi

while IFS= read -r required; do
  if ! grep -Fxq "$required" "$seen_ops"; then
    fail "$contract is missing required operation: $required"
  fi
done < <(required_operations)

if [ "$failures" -ne 0 ]; then
  exit 1
fi

printf 'check-test262-host-abi: ok\n'
