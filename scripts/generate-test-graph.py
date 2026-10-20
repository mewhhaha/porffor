#!/usr/bin/env python3
"""Generate dated README diagrams from committed evidence, without rerunning tests."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
START = '<!-- scriptw-test-graph:start -->'
END = '<!-- scriptw-test-graph:end -->'


def read_committed_or_local(root, name):
    path = root / name
    return path.read_bytes() if path.exists() else subprocess.check_output(
        ['git', 'show', ':' + name], cwd=root)


def render(inventory, publication):
    native = inventory['current_af']
    counts = native['native_tests']
    pending = native['expected_native_tests'] - sum(counts.values())
    if pending != native['remaining_uncompleted_native_tests'] or pending < 0:
        raise ValueError('native totals disagree with the saved inventory')
    real = publication['real_suite']
    outcomes = {row['label']: row['count'] for row in real['counts_per_outcome']}
    if set(outcomes) != {'Success', 'NotImplemented', 'Crash', 'Bug'}:
        raise ValueError('unknown published outcomes need an explicit diagram category')
    if sum(outcomes.values()) != real['total'] or outcomes['Success'] != real['passed']:
        raise ValueError('published totals disagree')
    return f'''{START}
### Test results at their measured revisions

These diagrams use **different dated checkpoints**. They do not establish
results for the renamed compiler or a current full-suite pass rate.

Latest saved all-feature Rust workspace run: source `{inventory['tested_source'][:9]}`,
2026-10-20, before the scriptw rename. The red case is an original-budget
5-second Wasm-worker timeout, not an established JavaScript semantic mismatch.

```mermaid
---
config:
  look: handDrawn
  theme: neutral
---
flowchart LR
  AF["Rust workspace tests<br/>{native['expected_native_tests']:,} native tests"]
  AF --> PASS["Succeeding<br/>{counts['passed']:,}"]
  AF --> FAIL["Failing: worker timeout<br/>{counts['failed']:,}"]
  AF --> WAIT["Not run yet<br/>{pending:,}"]
  AF --> IGNORED["Ignored<br/>{counts['ignored']:,}"]
  DOC["Documentation tests<br/>{native['expected_documentation_tests']} not run"]
  classDef pass fill:#e0f1df,stroke:#34714b,color:#193d27
  classDef fail fill:#ffe4df,stroke:#a63d32,color:#702a23
  classDef wait fill:#f2eee4,stroke:#8b8170,color:#514a3d
  class PASS pass
  class FAIL fail
  class WAIT,DOC,IGNORED wait
```

Scopes: {native['green_scopes']} green, {native['red_scopes']} red and
{native['remaining_uncompleted_scopes']} unclosed of {native['expected_scopes']}.

Latest **published historical Test262** snapshot: `{real['refresh_date']}`,
pin `{real['pinned_revisions']['test262'][:9]}`, {real['total']:,} path-based entries.
These older counts are preserved for reference; the current mode-aware
102,994-execution families have not completed.

```mermaid
---
config:
  look: handDrawn
  theme: neutral
---
flowchart LR
  OLD["Historical Test262 publication<br/>{real['refresh_date']}"]
  OLD --> OK["Success<br/>{outcomes['Success']:,}"]
  OLD --> MISSING["NotImplemented<br/>{outcomes['NotImplemented']:,}"]
  OLD --> BUG["Bug<br/>{outcomes['Bug']:,}"]
  OLD --> CRASH["Crash<br/>{outcomes['Crash']:,}"]
  classDef pass fill:#e0f1df,stroke:#34714b,color:#193d27
  classDef fail fill:#ffe4df,stroke:#a63d32,color:#702a23
  classDef missing fill:#fff0cc,stroke:#94742e,color:#614b1e
  class OK pass
  class BUG,CRASH fail
  class MISSING missing
```

**Unrun is not the same as unimplemented.** Current capability gaps include
unmatched runtime-generated source and selected host/weak-reachability
operations; see the [inventory](docs/rust-rewrite/current-inventory.md).
The diagrams use Mermaid's hand-drawn look, rendered directly by GitHub.
Refresh/check: `python3 scripts/generate-test-graph.py [--check]`.
Inputs: [workspace evidence](docs/rust-rewrite/current-inventory-20261020.json)
and [historical publication](test262/snapshots/published-status-wasm-aot.json).
{END}'''


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    inventory = json.loads(read_committed_or_local(ROOT, 'docs/rust-rewrite/current-inventory-20261020.json'))
    publication = json.loads(read_committed_or_local(ROOT, 'test262/snapshots/published-status-wasm-aot.json'))
    graph = render(inventory, publication)
    path = ROOT / 'README.md'
    before = path.read_text()
    if START in before:
        if before.count(START) != 1 or before.count(END) != 1:
            raise ValueError('expected one generated diagram block')
        after = re.sub(re.escape(START) + '.*?' + re.escape(END), lambda _: graph, before, flags=re.S)
    else:
        after = before.replace('## Historical development checkpoints', graph + '\n\n## Historical development checkpoints', 1)
        if after == before:
            raise ValueError('missing README insertion point')
    if args.check:
        if after != before:
            raise ValueError('README diagrams are stale; regenerate them')
    else:
        path.write_text(after)
    print('README test diagrams: current with recorded evidence')


if __name__ == '__main__':
    main()
