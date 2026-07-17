#!/usr/bin/env bash
set -euo pipefail

BACKEND="${1:-spec-exec}"
SNAPSHOT_NAME="${2:-codex-published-real}"
PORF_BIN="${PORF_BIN:-./target/release/porf}"
SUITE_ROOT="${SUITE_ROOT:-test262/vendor/test262}"
SNAPSHOT_DIR="${SNAPSHOT_DIR:-test262/snapshots}"
THREADS="${THREADS:-1}"
JOBS="${JOBS:-1}"
ISOLATE_CASES="${ISOLATE_CASES:-1}"
MAX_MATRIX_NODES="${MAX_MATRIX_NODES:-1}"
README_PATH="${README_PATH:-}"
MATRIX_TOTAL=""
MATRIX_COMPLETED=0

if [[ "$ISOLATE_CASES" == "1" ]]; then
  export PORFFOR_TEST262_FORCE_CASE_RUNNER=1
fi

if [[ ! -x "$PORF_BIN" ]]; then
  echo "missing executable: $PORF_BIN" >&2
  echo "build first: cargo build --release -p porffor-cli" >&2
  exit 1
fi

matrix_progress() {
  local aggregate_glob aggregate_path
  aggregate_glob="$SNAPSHOT_DIR/${SNAPSHOT_NAME}-aggregate-"'*.json'
  aggregate_path=""
  if compgen -G "$aggregate_glob" > /dev/null; then
    aggregate_path="$(ls "$SNAPSHOT_DIR"/"${SNAPSHOT_NAME}"-aggregate-*.json | head -n 1)"
  fi

  if [[ -z "$MATRIX_TOTAL" ]]; then
    local inventory
    inventory="$(AGGREGATE_PATH="$aggregate_path" SUITE_ROOT="$SUITE_ROOT" node - <<'NODE'
const fs = require('fs');
const path = require('path');

const root = path.join(process.cwd(), process.env.SUITE_ROOT, 'test');
const aggregatePath = process.env.AGGREGATE_PATH || '';
const TOP_LEVEL_FILTERS = ['annexB', 'built-ins', 'harness', 'intl402', 'language', 'staging'];
const MATRIX_SPLIT_FILTERS = new Set(['built-ins', 'intl402', 'language', 'staging']);
const MATRIX_RECURSION_THRESHOLD = 500;
const MATRIX_CHUNK_SIZE = 250;

function scan(dir, relBase = '') {
  let out = [];
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    const full = path.join(dir, ent.name);
    const rel = relBase ? path.posix.join(relBase, ent.name) : ent.name;
    if (ent.isDirectory()) out = out.concat(scan(full, rel));
    else if (ent.isFile() && ent.name.endsWith('.js')) out.push(rel);
  }
  return out;
}

function groupCasesBySegment(cases, segmentIndex) {
  const groups = new Map();
  for (const entry of cases) {
    const segs = entry.path.split('/');
    const seg = segs[segmentIndex];
    if (seg === undefined) continue;
    if (!groups.has(seg)) groups.set(seg, []);
    groups.get(seg).push(entry);
  }
  return [...groups.entries()].sort((a, b) => a[0].localeCompare(b[0]));
}

function groupCasesByDirectorySegment(cases, segmentIndex) {
  const groups = new Map();
  for (const entry of cases) {
    const segs = entry.path.split('/');
    if (segs.length <= segmentIndex + 1) continue;
    const seg = segs[segmentIndex];
    if (seg === undefined) continue;
    if (!groups.has(seg)) groups.set(seg, []);
    groups.get(seg).push(entry);
  }
  return [...groups.entries()].sort((a, b) => a[0].localeCompare(b[0]));
}

function finalize(filter, cases) {
  const ordered = [...cases].sort((a, b) => a.path.localeCompare(b.path));
  if (ordered.length > MATRIX_RECURSION_THRESHOLD) {
    const totalChunks = Math.ceil(ordered.length / MATRIX_CHUNK_SIZE);
    const nodes = [];
    for (let i = 0; i < totalChunks; i += 1) {
      nodes.push(
        `${filter}@chunk-${String(i + 1).padStart(4, '0')}-of-${String(totalChunks).padStart(4, '0')}`,
      );
    }
    return nodes;
  }
  return [filter];
}

function buildForRoot(rootName, cases) {
  if (!MATRIX_SPLIT_FILTERS.has(rootName) || cases.length === 0) {
    return finalize(rootName, cases);
  }
  const childGroups = groupCasesBySegment(cases, 1);
  if (childGroups.length === 0) return finalize(rootName, cases);

  let nodes = [];
  for (const [child, childCases] of childGroups) {
    const childFilter = `${rootName}/${child}`;
    if (childCases.length > MATRIX_RECURSION_THRESHOLD) {
      const grandchildGroups = groupCasesByDirectorySegment(childCases, 2);
      if (grandchildGroups.length > 0) {
        const covered = new Set(
          grandchildGroups.flatMap(([, group]) => group.map(entry => entry.path)),
        );
        const residual = childCases.filter(entry => !covered.has(entry.path));
        if (residual.length > 0) nodes = nodes.concat(finalize(childFilter, residual));
        for (const [grandchild, grandchildCases] of grandchildGroups) {
          nodes = nodes.concat(finalize(`${childFilter}/${grandchild}`, grandchildCases));
        }
        continue;
      }
    }
    nodes = nodes.concat(finalize(childFilter, childCases));
  }
  return nodes;
}

const cases = scan(root).sort().map(testPath => ({ path: testPath }));
const topGroups = groupCasesBySegment(cases, 0);
let nodes = [];
for (const top of TOP_LEVEL_FILTERS) {
  const found = topGroups.find(([name]) => name === top);
  nodes = nodes.concat(buildForRoot(top, found ? found[1] : []));
}

let completed = 0;
if (aggregatePath && fs.existsSync(aggregatePath)) {
  const aggregate = JSON.parse(fs.readFileSync(aggregatePath, 'utf8'));
  completed = Array.isArray(aggregate.completed_nodes) ? aggregate.completed_nodes.length : 0;
}

console.log(`${completed} ${nodes.length}`);
NODE
    )"
    read -r MATRIX_COMPLETED MATRIX_TOTAL <<<"$inventory"
    return 0
  fi

  MATRIX_COMPLETED=0
  if [[ -n "$aggregate_path" && -f "$aggregate_path" ]]; then
    MATRIX_COMPLETED="$(AGGREGATE_PATH="$aggregate_path" python3 - <<'PY'
import json
import os
from pathlib import Path

aggregate = json.loads(Path(os.environ["AGGREGATE_PATH"]).read_text())
completed = aggregate.get("completed_nodes", [])
print(len(completed) if isinstance(completed, list) else 0)
PY
    )"
  fi
}

while true; do
  matrix_progress
  completed="$MATRIX_COMPLETED"
  total="$MATRIX_TOTAL"
  echo "matrix_progress: ${completed}/${total}"

  if [[ "$total" -gt 0 && "$completed" -ge "$total" ]]; then
    cmd=(
      "$PORF_BIN" test262 publish-status
      --execution-backend "$BACKEND"
      --suite-root "$SUITE_ROOT"
      --snapshot-dir "$SNAPSHOT_DIR"
      --snapshot-name "$SNAPSHOT_NAME"
    )
    if [[ -n "$README_PATH" ]]; then
      cmd+=(--readme-path "$README_PATH")
    fi
    exec "${cmd[@]}"
  fi

  "$PORF_BIN" test262 report-all \
    --jobs "$JOBS" \
    --execution-backend "$BACKEND" \
    --suite-root "$SUITE_ROOT" \
    --snapshot-dir "$SNAPSHOT_DIR" \
    --snapshot-name "$SNAPSHOT_NAME" \
    --resume \
    --threads "$THREADS" \
    --max-matrix-nodes "$MAX_MATRIX_NODES"
done
