#!/usr/bin/env bash
set -euo pipefail

BACKEND="${1:-spec-exec}"
SNAPSHOT_NAME="${2:-codex-published-real}"
PORF_BIN="${PORF_BIN:-./target/debug/porf}"
SUITE_ROOT="${SUITE_ROOT:-test262/vendor/test262}"
SNAPSHOT_DIR="${SNAPSHOT_DIR:-test262/snapshots}"
THREADS="${THREADS:-1}"
MAX_MATRIX_NODES="${MAX_MATRIX_NODES:-1}"
README_PATH="${README_PATH:-}"

if [[ ! -x "$PORF_BIN" ]]; then
  echo "missing executable: $PORF_BIN" >&2
  echo "build first: cargo build -p porffor-cli" >&2
  exit 1
fi

matrix_progress() {
  local aggregate_glob aggregate_path
  aggregate_glob="$SNAPSHOT_DIR/${SNAPSHOT_NAME}-aggregate-"'*.json'
  aggregate_path=""
  if compgen -G "$aggregate_glob" > /dev/null; then
    aggregate_path="$(ls "$SNAPSHOT_DIR"/"${SNAPSHOT_NAME}"-aggregate-*.json | head -n 1)"
  fi

  AGGREGATE_PATH="$aggregate_path" SUITE_ROOT="$SUITE_ROOT" node - <<'NODE'
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
}

reset_stale_failed_nodes_once() {
  local aggregate_glob aggregate_path
  aggregate_glob="$SNAPSHOT_DIR/${SNAPSHOT_NAME}-aggregate-"'*.json'
  aggregate_path=""
  if compgen -G "$aggregate_glob" > /dev/null; then
    aggregate_path="$(ls "$SNAPSHOT_DIR"/"${SNAPSHOT_NAME}"-aggregate-*.json | head -n 1)"
  fi

  if [[ -z "$aggregate_path" ]]; then
    return 0
  fi

  AGGREGATE_PATH="$aggregate_path" SNAPSHOT_DIR="$SNAPSHOT_DIR" SNAPSHOT_NAME="$SNAPSHOT_NAME" python3 - <<'PY'
import json
import os
from pathlib import Path

aggregate_path = Path(os.environ["AGGREGATE_PATH"])
snapshot_dir = Path(os.environ["SNAPSHOT_DIR"])
snapshot_name = os.environ["SNAPSHOT_NAME"]

if not aggregate_path.exists():
    raise SystemExit(0)

aggregate = json.loads(aggregate_path.read_text())
entries = aggregate.get("aggregate_entries", [])
stale_entries = [entry for entry in entries if entry.get("failed", 0) > 0]
if not stale_entries:
    raise SystemExit(0)

stale_node_ids = {entry["node_id"] for entry in stale_entries}
aggregate["completed_nodes"] = [
    node_id for node_id in aggregate.get("completed_nodes", []) if node_id not in stale_node_ids
]
aggregate["aggregate_entries"] = [
    entry for entry in entries if entry["node_id"] not in stale_node_ids
]

aggregate["total"] = sum(entry.get("total", 0) for entry in aggregate["aggregate_entries"])
aggregate["passed"] = sum(entry.get("passed", 0) for entry in aggregate["aggregate_entries"])
counts = {}
for entry in aggregate["aggregate_entries"]:
    for kind, count in entry.get("counts_per_kind", {}).items():
        counts[kind] = counts.get(kind, 0) + count
aggregate["aggregate_counts_so_far"] = counts

for entry in stale_entries:
    safe_node_id = entry["node_id"].replace("/", "_")
    stem = f"{snapshot_name}-{safe_node_id}-{entry['manifest_hash']}"
    for ext in ("json", "txt"):
        snapshot_path = snapshot_dir / f"{stem}.{ext}"
        if snapshot_path.exists():
            snapshot_path.unlink()

aggregate_path.write_text(json.dumps(aggregate, indent=2) + "\n")
PY
}

did_reset_stale_failed_nodes=0

while true; do
  read -r completed total < <(matrix_progress)
  echo "matrix_progress: ${completed}/${total}"

  if [[ "$did_reset_stale_failed_nodes" -eq 0 ]]; then
    reset_stale_failed_nodes_once
    did_reset_stale_failed_nodes=1
    read -r completed total < <(matrix_progress)
    echo "matrix_progress_after_stale_reset: ${completed}/${total}"
  fi

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
    --execution-backend "$BACKEND" \
    --suite-root "$SUITE_ROOT" \
    --snapshot-dir "$SNAPSHOT_DIR" \
    --snapshot-name "$SNAPSHOT_NAME" \
    --resume \
    --threads "$THREADS" \
    --max-matrix-nodes "$MAX_MATRIX_NODES"
done
