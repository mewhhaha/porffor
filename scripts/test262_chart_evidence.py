"""Reconcile the immutable full baseline, exact failure replay and product scope.

This is a display projection. It neither republishes native suite results nor
claims that successes from an older compiler were rerun on the repaired one.
"""
import hashlib
import json
from pathlib import Path
import re
import tarfile

from check_conformance_source_pins import verify as verify_source_pins
from status_publication import load_publication

EVIDENCE = 'docs/rust-rewrite/test262-chart-reconciliation-20261021.json'
BASE_RECEIPT = 'test262/snapshots/current-test262-20261020-receipt.json'
REPLAY_RECEIPT = 'docs/rust-rewrite/failed-case-replay-20261021.json'
BACKLOG = 'test262/backlog/6a4268a9354a545d41c3b62efebc478ed8c521f4/wasm-aot.json'
EVAL_ESIDS = {'sec-performeval', 'sec-performeval-rules-in-initializer'}
EVAL_NON_SOURCE_CASES = {
    'language/eval-code/direct/non-string-object.js',
    'language/eval-code/direct/non-string-primitive.js',
    'language/eval-code/indirect/non-string-object.js',
    'language/eval-code/indirect/non-string-primitive.js',
}
FUNCTION_CASES = {
    'annexB/built-ins/Function/createdynfn-html-close-comment-body.js',
    'annexB/built-ins/Function/createdynfn-html-close-comment-params.js',
    'annexB/built-ins/Function/createdynfn-html-open-comment-body.js',
    'annexB/built-ins/Function/createdynfn-html-open-comment-params.js',
    'annexB/built-ins/Function/createdynfn-no-line-terminator-html-close-comment-body.js',
    'annexB/built-ins/Function/createdynfn-no-line-terminator-html-close-comment-params.js',
    'built-ins/Function/StrictFunction_reservedwords_with.js',
    'built-ins/Function/private-identifiers-not-empty.js',
}


def sha256(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def require(condition, message):
    if not condition:
        raise ValueError(message)


def classify(path, source):
    frontmatter = re.search(r'/\*---(.*?)---\*/', source, re.S)
    if not frontmatter:
        return None
    match = re.search(r'^esid:\s*([^\s#]+)\s*$', frontmatter[1], re.M)
    esid = match[1] if match else None
    if esid in EVAL_ESIDS and path not in EVAL_NON_SOURCE_CASES:
        return esid, 'dynamic-eval-primary-target'
    if path in FUNCTION_CASES:
        require(esid == 'sec-createdynamicfunction', 'reviewed Function case changed its specification target')
        return esid, 'dynamic-function-source-parser'
    return None


def checked_replay(base_ids, base_failures, snapshot, pins):
    completed = snapshot['completed_test_ids']
    require(snapshot['pinned_revisions'] == pins and snapshot['execution_backend'] == 'wasm-aot', 'replay pin/backend differs')
    require(len(completed) == len(set(completed)) == snapshot['total'], 'replay is duplicated or incomplete')
    require(set(completed) == set(base_failures) <= set(base_ids), 'replay must cover every original failure exactly')
    failures = {row['test_id']: row['outcome'] for row in snapshot['failures']}
    require(len(failures) == len(snapshot['failures']) and set(failures) <= set(completed), 'invalid replay failures')
    require(snapshot['passed'] + len(failures) == snapshot['total'], 'replay counts disagree')
    rows = {key: failures.get(key, 'Success') for key in completed}
    require({kind: sum(v == kind for v in rows.values()) for kind in snapshot['counts_per_outcome']} == snapshot['counts_per_outcome'], 'replay outcome census disagrees')
    return rows


def scoped_rows(root, execution_ids):
    paths = {}
    for execution in execution_ids:
        mode, separator, path = execution.partition(':')
        require(separator and mode in ('sloppy-script', 'strict-script', 'raw-script', 'module', 'raw-module')
                and not Path(path).is_absolute() and '..' not in Path(path).parts, 'invalid archived execution ID')
        paths.setdefault(path, []).append(execution)
    rows = []
    for name, executions in sorted(paths.items()):
        path = root / 'test262/vendor/test262/test' / name
        source = path.read_bytes()
        decision = classify(name, source.decode('utf-8'))
        if decision:
            esid, reason = decision
            rows.append(dict(path=name, source_sha256=hashlib.sha256(source).hexdigest(),
                             esid=esid, reason=reason, execution_ids=sorted(executions)))
    require(FUNCTION_CASES <= {row['path'] for row in rows}, 'reviewed Function scope inventory is incomplete')
    return rows


def projection(outcomes, scope_ids):
    require(scope_ids <= outcomes.keys(), 'scope refers to unmeasured execution IDs')
    require(all(outcomes[key] == 'Success' for key in scope_ids), 'scope may not hide a measured non-success')
    counts = {name: sum(value == name for key, value in outcomes.items() if key not in scope_ids)
              for name in ('Success', 'Bug', 'Crash', 'NotImplemented')}
    counts['OutOfScope'] = len(scope_ids)
    require(sum(counts.values()) == len(outcomes), 'scope projection changed the full denominator')
    return counts


def build(root):
    publication, published_path = load_publication(root)
    real = publication['real_suite']
    base_receipt_path = root / BASE_RECEIPT
    base_receipt = json.loads(base_receipt_path.read_text())
    base_archive = root / base_receipt['archive']
    require(base_receipt['complete'] and sha256(base_archive) == base_receipt['archive_sha256'], 'full baseline archive identity differs')
    with tarfile.open(base_archive, 'r:gz') as archive:
        snapshots = [json.load(archive.extractfile(member)) for member in archive.getmembers()
                     if member.isfile() and member.name.startswith('snapshots/scriptw-current-20261020-') and member.name.endswith('.json')]
    aggregates = [s for s in snapshots if s['run_kind'] == 'aggregate-matrix']
    require(len(aggregates) == 1, 'expected one archived complete aggregate')
    aggregate = aggregates[0]
    pins = aggregate['pinned_revisions']
    expected_counts = {row['label']: row['count'] for row in real['counts_per_outcome']}
    require(aggregate['counts_per_outcome'] == expected_counts and aggregate['total'] == real['total']
            and aggregate['passed'] == real['passed'] and aggregate['compiler_identity'] == real['compiler_identity']
            and pins == real['pinned_revisions'] and aggregate['manifest_hash'] == real['manifest_hash'], 'full archive differs from published baseline')
    nodes = {s['manifest_hash']: s for s in snapshots if s['run_kind'] != 'aggregate-matrix'}
    outcomes = {}
    for entry in aggregate['aggregate_entries']:
        node = nodes[entry['manifest_hash']]
        require(node['compiler_identity'] == aggregate['compiler_identity'] and node['pinned_revisions'] == pins
                and node['counts_per_outcome'] == entry['counts_per_outcome'], 'baseline leaf identity differs')
        ids = node['completed_test_ids']
        require(len(ids) == len(set(ids)) == node['total'] == entry['total'] and not outcomes.keys() & set(ids), 'baseline leaf incomplete or overlapping')
        failures = {row['test_id']: row['outcome'] for row in node['failures']}
        require(len(failures) == len(node['failures']) and set(failures) <= set(ids)
                and node['passed'] + len(failures) == node['total'], 'baseline leaf outcomes incomplete')
        outcomes.update({key: failures.get(key, 'Success') for key in ids})
    require(len(outcomes) == real['total'] and {k: sum(v == k for v in outcomes.values()) for k in expected_counts} == expected_counts, 'full baseline execution census differs')
    base_failures = {key: value for key, value in outcomes.items() if value != 'Success'}
    backlog_path = root / BACKLOG
    backlog_bytes = backlog_path.read_bytes()
    backlog = json.loads(backlog_bytes)
    require({row['test_id']: row['outcome'] for row in backlog['records']} == base_failures, 'backlog differs from native baseline failures')
    replay_receipt_path = root / REPLAY_RECEIPT
    receipt = json.loads(replay_receipt_path.read_text())
    replay_archive = root / receipt['evidence_archive']
    require(receipt['complete'] and receipt['source_equivalence']['original_backlog_sha256'] == hashlib.sha256(backlog_bytes).hexdigest()
            and sha256(replay_archive) == receipt['evidence_archive_sha256'], 'replay archive/baseline binding differs')
    with tarfile.open(replay_archive, 'r:gz') as archive:
        originals = [m for m in archive.getmembers() if m.isfile() and m.name.startswith('original-snapshots/') and m.name.endswith('.json')]
        require(len(originals) == 1, 'expected one original-fixture replay; errata are excluded')
        replay = json.load(archive.extractfile(originals[0]))
    replayed = checked_replay(outcomes, base_failures, replay, pins)
    require(replay['compiler_identity'] == receipt['compiler_identity']
            and replay['counts_per_outcome'] == receipt['original_fixture_replay']['counts_per_outcome'], 'replay receipt differs from original native snapshot')
    reconciled = dict(outcomes, **replayed)
    pin_check = verify_source_pins(root / 'crates/scriptw-test262/src/source-pins.json', root / 'test262/vendor/test262')
    require(pin_check['vendored_tree_verified'], 'scope sources are not the pinned suite')
    rows = scoped_rows(root, outcomes)
    scope_ids = {key for row in rows for key in row['execution_ids']}
    counts = projection(reconciled, scope_ids)
    return dict(schema='scriptw-test262-chart-reconciliation-v1', date='2026-10-21',
        interpretation='Full published baseline successes plus exact original-failure replay; not a fresh full run of the repaired compiler.',
        source_pins=pins, baseline_compiler_identity=aggregate['compiler_identity'],
        replay_compiler_identity=replay['compiler_identity'], total=len(outcomes),
        baseline_counts=expected_counts, replay_counts=replay['counts_per_outcome'],
        reconciled_counts={k: sum(v == k for v in reconciled.values()) for k in expected_counts},
        scoped_chart_counts=counts,
        remaining_failures=[dict(execution_id=k, outcome=v) for k, v in sorted(reconciled.items()) if v != 'Success'],
        replayed_execution_ids=sorted(replayed),
        scope_policy=dict(version=1, meaning='Primary-target dynamic-source semantics outside the standalone Wasm product promise; native-host passes remain in historical outcomes.',
            coverage='Conservative audited subset; not an exhaustive count of every dynamic-source use.',
            eval_esids=sorted(EVAL_ESIDS), eval_non_source_exclusions=sorted(EVAL_NON_SOURCE_CASES), reviewed_function_paths=sorted(FUNCTION_CASES),
            preserves='API properties, call/apply/bind, constructor setup, prepared-source support and unrelated mentions remain ordinary outcomes.'),
        scope_source_paths=len(rows), scope_execution_count=len(scope_ids), scope_rows=rows,
        evidence=[dict(path=str(p.relative_to(root)), sha256=sha256(p)) for p in
                  (root / published_path, base_receipt_path, base_archive, backlog_path, replay_receipt_path, replay_archive)],
        full_current_source_rerun=False)


def load(root, publication):
    path = root / EVIDENCE
    if not path.exists():
        return None
    record = json.loads(path.read_text())
    require(record['schema'] == 'scriptw-test262-chart-reconciliation-v1', 'invalid chart reconciliation schema')
    require(record == build(root), 'stale chart reconciliation; regenerate from retained native evidence')
    require(record['baseline_compiler_identity'] == publication['real_suite']['compiler_identity'], 'chart baseline compiler differs')
    return record
