"""Verify the retained historical result projection offline; executes no JS or Cargo."""
import collections
import hashlib
import json
from pathlib import Path
import sys

def unpack(entry):
    raw = entry['raw'].encode()
    assert len(raw) == entry['bytes']
    assert hashlib.sha256(raw).hexdigest() == entry['sha256']
    return json.loads(raw)

def verify(path):
    proof = json.loads(path.read_text())
    assert proof['schema'] == 'closed-original-history-and-diagnostics-eb1ca63c9-v1'
    assert proof['source_head'] == 'eb1ca63c9dbf09148703ccfbd0ca35d9629753e7'
    assert proof['complete'] and proof['evidence_verified']
    assert proof['literal_conformance_passed'] is False and proof['strict_retention_passed'] is False
    assert proof['full_pin_or_task_acceptance'] is False
    audit, summary, snapshot = [unpack(proof[key]) for key in ('audit', 'summary', 'native_snapshot')]
    assert audit['complete'] and audit['evidence_verified'] and not audit['native_executed_by_auditor']
    assert audit['task_acceptance'] is False and audit['literal_all_success'] is False
    source = proof['historical_input']
    assert len(source['raw'].encode()) == source['bytes']
    assert hashlib.sha256(source['raw'].encode()).hexdigest() == source['sha256'] == '95d46445f4c3fda986c7cdf390dccdddfb8ab370b8a831e3f77ea7f387a1a43e'
    expected = source['raw'].splitlines()
    assert len(expected) == len(set(expected)) == 5365
    rows = summary['results']
    mapping = {row['execution_id']: row['outcome'] for row in rows}
    assert len(rows) == len(mapping) == summary['total'] == snapshot['total'] == 5365
    assert set(mapping) == set(expected) == set(snapshot['completed_test_ids'])
    assert len(snapshot['completed_test_ids']) == 5365
    failures = {row['test_id']: row['outcome'] for row in snapshot['failures']}
    assert len(failures) == len(snapshot['failures']) and set(failures) <= set(expected)
    assert mapping == {key: failures.get(key, 'Success') for key in expected}
    counts = dict(collections.Counter(mapping.values()))
    assert counts == {'Success': 5194, 'NotImplemented': 167, 'Bug': 4}
    assert snapshot['passed'] == 5194 and not snapshot['timeout_test_ids']
    for key, value in snapshot['counts_per_outcome'].items():
        assert counts.get(key, 0) == value
    for key, value in audit['historical']['counts'].items():
        assert counts.get(key, 0) == value
    assert summary['binary_sha256'] == summary['compiler_identity']['executable_sha256'] == 'c2f50a36a481394171b568d52440257fa259554e10cc290ce5301aecec3c1246'
    assert snapshot['compiler_identity'] == summary['compiler_identity']
    assert snapshot['pinned_revisions'] == audit['historical']['native_pins'] == {
        'test262': 'aa55200d1310384c5cf69ea95b2a2ecba457007b', 'ecma262': '5345883164f463e87f8b40aca4956157ecba8783'}
    assert proof['summary']['sha256'] == audit['historical']['summary']['sha256']
    for row in rows:
        assert row['native_snapshot']['sha256'] == proof['native_snapshot']['sha256']
    current = {key for key, value in mapping.items() if value == 'Success'}
    previous = proof['previous_5142_positive_ids']
    strict = proof['strict_prior_positive_ids']
    assert len(previous) == len(set(previous)) == 5142 and len(strict) == len(set(strict)) == 5173
    assert len(set(previous) & current) == 5141 and len(set(strict) & current) == 5172
    missing = ['module:language/import/import-bytes/bytes-from-txt.js']
    assert sorted(set(strict) - current) == sorted(set(previous) - current) == missing
    assert sorted(current - set(previous)) == proof['newly_passing_previous_nonpasses']
    assert len(proof['newly_passing_previous_nonpasses']) == 53
    assert audit['retention_5173'] == {'required': 5173, 'retained': 5172, 'missing': [],
        'regressions': [{'execution_id': missing[0], 'outcome': 'Bug'}], 'passed': False}
    diagnostics, owner = unpack(proof['diagnostics']), unpack(proof['diagnostic_owner'])
    assert diagnostics['complete'] and diagnostics['acceptance'] is False
    assert owner['complete'] and owner['source_unchanged'] and owner['exit_code'] == 0
    assert [row['exit_code'] for row in diagnostics['commands']] == [0, 1, 0]
    for row in diagnostics['commands']:
        for stream in ('stdout', 'stderr'):
            raw = proof['diagnostic_logs'][row['name']][stream].encode()
            assert len(raw) == row[stream]['bytes'] and hashlib.sha256(raw).hexdigest() == row[stream]['sha256']
    assert 'fixture line 33: ReferenceError' in proof['diagnostic_logs']['annex-label-stage']['stderr']
    observed = set()
    for entry in proof['diagnostic_snapshots']:
        value = unpack(entry)
        assert value['total'] == value['passed'] == 1 and value['failures'] == []
        assert value['compiler_identity'] == summary['compiler_identity']
        observed.update(value['completed_test_ids'])
    assert observed == {missing[0], 'sloppy-script:staging/sm/RegExp/unicode-ignoreCase.js'}
    unpack(proof['missing_dependency_analysis'])
    print('Verified original history: 5365 IDs; 5194 Success, 167 NotImplemented, 4 Bug, 0 Crash; strict retention 5172/5173. Full-pin/task acceptance remains false.')

if __name__ == '__main__':
    default = Path(__file__).with_name('original-history-eb1ca63c9-20261015.json')
    verify(Path(sys.argv[1]) if len(sys.argv) > 1 else default)
