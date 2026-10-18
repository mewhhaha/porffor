"""Bound workspace linking while retaining one exact full-suite denominator.

Run from the repository root under limited_verification.py --cloud. A proven
zero-test library keeps the normal Cargo --lib feature roots present between
integration batches; its auxiliary executions never add to suite counts.
"""
import argparse
import collections
import fcntl
import hashlib
import json
from pathlib import Path
import subprocess
import sys


def key(case):
    return (case['package'], case['target'], case['kind'])


def qualified(case):
    return case['package'] + ':' + case['target']


def cohorts(cases, isolated, size):
    """Keep same-selector siblings together and every original target once."""
    if size < 1:
        raise ValueError('cohort size must be positive')
    groups = []
    for isolation in [None, *sorted(isolated)]:
        selected = [c for c in cases if
                    (c['package'] == isolation if isolation else c['package'] not in isolated)]
        libraries = [c for c in selected if c['kind'] == 'lib']
        if libraries:
            groups.append(libraries)
        selectors = collections.defaultdict(list)
        for case in selected:
            if case['kind'] not in ('lib', 'doc'):
                selectors[tuple(case['selector'])].append(case)
        batch = []
        for siblings in selectors.values():
            if len(siblings) > size:
                raise ValueError('same-selector siblings exceed the cohort size')
            if len(batch) + len(siblings) > size:
                groups.append(batch)
                batch = []
            batch.extend(siblings)
        if batch:
            groups.append(batch)
        docs = [c for c in selected if c['kind'] == 'doc']
        if docs:
            groups.append(docs)
    flattened = [key(c) for group in groups for c in group]
    if len(flattened) != len(set(flattened)) or set(flattened) != {key(c) for c in cases}:
        raise ValueError('cohort plan changed the complete target inventory')
    return groups


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def retire_integrations(root, receipt, selected, record):
    """Only retire sealed, completed exact outputs after the child has exited."""
    profile = Path(receipt['target_directory']) / receipt['profile']
    wanted = {key(c) for c in selected if c['kind'] in ('test', 'bin')}
    rows = [c for c in receipt['cases'] if key(c) in wanted]
    if {key(c) for c in rows} != wanted or len(rows) != len(wanted):
        raise ValueError('retirement lacks an exact completed target')
    pending = []
    for case in rows:
        if case['state'] != 'passed' or not case['inventory_matches'] or case['exit_code']:
            raise ValueError('retirement requires a passed exact native scope')
        for field, digest in [('inventory_log', 'inventory_sha256'),
                              ('transcript_log', 'transcript_sha256')]:
            if sha(root / case[field]) != case[digest]:
                raise ValueError('native evidence changed before retirement')
        group = receipt['build_groups'][case['build_group']]
        # Actual metadata owns the exact package identity, including same-name
        # targets in different packages; the caller supplies it from that run.
        matches = [x for x in group['executables'] if
                   x['target'] == case['target'] and x['kind'] == case['kind'] and
                   x['package_id'] == case['package_id']]
        if len(matches) != 1:
            raise ValueError('retirement lacks an exact Cargo artifact identity')
        path = root / matches[0]['path']
        if path.is_symlink() or path.resolve().parent != (profile / 'deps').resolve():
            raise ValueError('retirement output escaped the native profile')
        if sha(path) != case['executable_sha256'] or path.stat().st_size != case['executable_bytes']:
            raise ValueError('native executable changed before retirement')
        pending.append({'path': str(path), 'sha256': case['executable_sha256'],
                        'bytes': case['executable_bytes'], 'target': qualified(case)})
    if len({r['path'] for r in pending}) != len(pending):
        raise ValueError('duplicate retirement path')
    with (profile / '.cargo-lock').open('a+b') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        record(pending, 'sealed-before-unlink')
        for row in pending:
            path = Path(row['path'])
            if sha(path) != row['sha256']:
                raise ValueError('native executable changed under Cargo lock')
            path.unlink()
            row['retired'] = True
            record(pending, 'retiring')
        record(pending, 'complete')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--cohort-size', type=int, default=8)
    parser.add_argument('--test-processes', type=int, default=4)
    parser.add_argument('--all-features', action='store_true')
    parser.add_argument('--plan-only', action='store_true')
    parser.add_argument('--partition-native-target', action='append', default=[])
    parser.add_argument('--isolate-test-package', action='append', default=[])
    parser.add_argument('--anchor', default='lila-backend-native:lila_backend_native')
    args = parser.parse_args()
    root = Path.cwd()
    out = args.output.resolve()
    out.mkdir(parents=True, exist_ok=False)
    base = [sys.executable, '-B', 'scripts/run_workspace_tests.py', '--include-ignored',
            '--test-processes', str(args.test_processes), '--partition-cli',
            '--build-selected-together', '--preserve-test-artifacts',
            '--shared-runtime-linker', 'scripts/test-shared-runtime-images/cc',
            '--strip-test-debug', '--no-test-incremental', '--stop-on-build-failure']
    if args.all_features:
        base.append('--all-features')
    for package in args.isolate_test_package:
        base.extend(['--isolate-test-package', package])

    def execute(label, selection=None, planning=False):
        # The child uses its output basename in watched-log names. Retain the
        # unique outer run name so another full run cannot overwrite old logs.
        child_output = out / (out.name + '-' + label)
        command = [*base, '--output', str(child_output)]
        if selection is not None:
            command.extend(['--only-targets', *map(qualified, selection)])
        names = set(map(qualified, selection)) if selection is not None else None
        for target in args.partition_native_target:
            if names is None or target in names:
                command.extend(['--partition-native-target', target])
        if planning:
            command.append('--plan-only')
        with (out / (label + '.log')).open('xb') as log:
            code = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT).returncode
        path = child_output / 'receipt.json'
        return code, json.loads(path.read_text()), path

    code, plan, plan_path = execute('inventory', planning=True)
    if code or not plan['source_unchanged']:
        raise ValueError('complete native planning failed')
    source = json.loads(plan_path.with_name('source-start.json').read_text())
    groups = cohorts(plan['cases'], args.isolate_test_package, args.cohort_size)
    anchor = next(c for c in plan['cases'] if qualified(c) == args.anchor and c['kind'] == 'lib')
    if anchor['package'] in args.isolate_test_package:
        raise ValueError('auxiliary library anchor must belong to the normal graph')
    result = {'schema': 'lila-bounded-workspace-cohorts-v1', 'source': source,
              'inventory_receipt': str(plan_path), 'inventory_sha256': sha(plan_path),
              'targets': plan['selected_target_members'], 'cohorts': [],
              'all_features': args.all_features, 'native_accepted': False,
              'auxiliary_anchor': args.anchor, 'complete': False}

    def save():
        temporary = out / 'receipt.pending'
        temporary.write_text(json.dumps(result, indent=2) + '\n')
        temporary.replace(out / 'receipt.json')

    anchor_proven = False
    accepted = []
    for index, group in enumerate(groups):
        normal_native = any(c['kind'] != 'doc' and
                            c['package'] not in args.isolate_test_package for c in group)
        auxiliary = normal_native and not any(c['kind'] == 'lib' for c in group)
        selected = [*group, *([anchor] if auxiliary else [])]
        row = {'index': index, 'targets': [qualified(c) for c in group],
               'auxiliary_targets': [args.anchor] if auxiliary else []}
        result['cohorts'].append(row)
        save()
        if args.plan_only:
            continue
        if auxiliary and not anchor_proven:
            raise ValueError('auxiliary library anchor has not passed its zero-test inventory')
        code, child, path = execute(f'cohort-{index:03}', selected)
        row.update(receipt=str(path), receipt_sha256=sha(path), exit_code=code)
        if json.loads(path.with_name('source-start.json').read_text()) != source or not child['source_unchanged']:
            raise ValueError('source changed between native cohorts')
        if {key(c) for c in child['cases']} != {key(c) for c in selected}:
            raise ValueError('child changed the exact target denominator')
        passed = code == 0 and all(c.get('state') == 'passed' for c in child['cases'])
        row['passed'] = passed
        save()
        if not passed:
            return 1
        for case in child['cases']:
            if qualified(case) == args.anchor:
                if case['inventoried_tests'] != 0 or case['outcomes'] != [{'passed': 0, 'failed': 0, 'ignored': 0}]:
                    raise ValueError('auxiliary anchor has acquired tests; replan instead of duplicating them')
                anchor_proven = True
        metadata = json.loads(path.with_name('metadata.json').read_text())
        package_ids = {p['name']: p['id'] for p in metadata['packages']}
        for case in child['cases']:
            case['package_id'] = package_ids[case['package']]

        def record(rows, state):
            row['retirement'] = {'state': state, 'outputs': rows}
            save()

        retire_integrations(root, child, group, record)
        accepted.extend(key(c) for c in group)
        row['accepted_targets'] = len(group)
        save()
        print('END COHORT', index, len(group), 'targets', flush=True)
    if args.plan_only:
        result['prepared_only'] = True
    else:
        assert len(accepted) == len(set(accepted)) == len(plan['cases'])
        assert set(accepted) == {key(c) for c in plan['cases']}
        result.update(complete=True, native_accepted=True, accepted_targets=len(accepted))
    save()
    return 0


if __name__ == '__main__':
    sys.exit(main())
