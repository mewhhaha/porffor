#!/usr/bin/env python3
"""Verify one exact CLI53 checkpoint entirely offline; never run a native program."""
import argparse
import ast
import base64
from collections import Counter
import gzip
import hashlib
import io
import json
from pathlib import Path
import re

HEAD = '8e9a8c8210f99d8d14b4267461ff42079b603c00'
SCHEMA = 'ordinary-cli-probe53-retention-publication-v1'
INPUT_SHA = '890f1d24817916a9c466282ac81407c3241d6bc101ad893b3ee53c8bb0e398c7'
D2_SHA = 'e45103a7be6ddeef5034d2b7851ab73b33c0e83f83227b89527c0ed890cd02e5'
D2_READER_SHA = '60eda8e977fdc4ca26b0e533c2e2c098c023b03ec6a5613780a3a270a87d3112'
POLICY_SHA = '545cb629f53089ccc0a098ea2aa47d635d3b0af9cf3ce6224b1d13f217a319c7'
SOURCE_PINS_SHA = 'ab47a6da190c07bc54ac2cc752580345f89fce9c31b739e7987be78b5ffea480'
PINNED_REVISIONS = {'ecma262': '5345883164f463e87f8b40aca4956157ecba8783',
                    'test262': '6a4268a9354a545d41c3b62efebc478ed8c521f4'}
PINNED_COMMITS = {'ecma262': '5345883164f463e87f8b40aca4956157ecba8783',
                  'test262': '2e0a56762801e275a9fdf96dc49d90ba0cddcf63'}
PACKET_SHA = 'e6a4ef5706f4c64c57f851f89a3ab3529b8a5770faa1353296ce1cb2234ea89d'
QUEUE_SHA = 'f6f6b4e5a1abaaddae1a287d92cc1ed0de148384f36f0ce3356b75e90d0b8454'
BUILDER_SHA = 'ae64177ba061223d61018bd11af284a66f7eb12dad06930c633a4384b6411cf6'
V3_PACKET_SHA = '97d68e3f5ccec072b8232e364442d43ccb41d12951ea5faaa212b4bc76d73626'
V3_ADOPTION_SHA = '55a8b3677e447a39a434fc484c60caf7ef3a9db44db7a9eca2ad9465479300d1'
V3_QUEUE_SHA = 'bd14d9ade7470d0d4316a55ffd76b6c9a7e133069e736f00d8653e7a9fce048b'
LIMIT = 64 * 1024 * 1024
OUTCOMES = ('Success', 'NotImplemented', 'Crash', 'Bug', 'InfrastructureError')
NI2 = {mode + ':staging/sm/generators/runtime.js' for mode in ('sloppy-script', 'strict-script')}
EMPTY = hashlib.sha256(b'').hexdigest()


def require(value, message):
    if not value:
        raise ValueError(message)


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def decode_assets(report):
    assets = {}
    for key, value in report['raw_evidence'].items():
        require(value['encoding'] == 'gzip-base64' and value['raw_bytes'] <= LIMIT,
                'Evidence encoding/read bound differs')
        packed = base64.b64decode(value['data'], validate=True)
        require(digest(packed) == value['archive_sha256'] and len(packed) == value['archive_bytes'],
                'Embedded archive bytes differ')
        with gzip.GzipFile(fileobj=io.BytesIO(packed), mode='rb') as stream:
            raw = stream.read(LIMIT + 1)
        require(len(raw) <= LIMIT and digest(raw) == key == value['raw_sha256']
                and len(raw) == value['raw_bytes'], 'Embedded raw bytes differ')
        assets[key] = raw
    return assets


def _verify_common(report, expected_schema):
    require(report['schema'] == expected_schema and report['complete'] is True
            and report['execution_evidence_verified'] is True and report['native_cli_replay_executed'] is True,
            'Require actual closed 53-ID publication')
    require(not any(report[key] for key in ('current_source_full_historical_replay_executed',
            'full_test262_acceptance', 'all_features_acceptance', 'task_closure')), 'Scope was overstated')
    assets = decode_assets(report)
    ids = assets[report['input']['sha256']].decode().splitlines()
    rows = report['results']
    require(len(ids) == len(set(ids)) == report['total'] == len(rows) == 53
            and [r['execution_id'] for r in rows] == ids, 'Every exact requested ID must remain once')
    counts = Counter(row['outcome'] for row in rows)
    require({key: counts[key] for key in report['outcomes']} == report['outcomes']
            and sum(counts.values()) == 53 and set(counts) <= set(report['outcomes'])
            and report['ordinary_cli_probe53_passed'] == (counts['Success'] == 53), 'Literal counts differ')
    before = {row['execution_id']: row for row in rows}
    for name, group in report['groups'].items():
        require(len(group['ids']) == len(set(group['ids'])) == group['count']
                and set(group['ids']) <= set(ids)
                and dict(Counter(before[x]['outcome'] for x in group['ids'])) == group['outcomes']
                and set(group['ids']) == {r['execution_id'] for r in rows if name in r['groups']},
                'Group projection differs')
    build = report['ordinary_build_preparation']
    messages = [json.loads(line) for line in assets[build['stdout_sha256']].decode().splitlines()
                if line.startswith('{')]
    require([m['success'] for m in messages if m.get('reason') == 'build-finished'] == [True],
            'Embedded actual Cargo stream did not finish successfully')
    require(all(build[key] in messages for key in ('cli_event', 'engine_event', 'runtime_event')),
            'Selected producer event differs from raw Cargo bytes')
    require(build['exit_code'] == 0 and build['cli_event']['profile']['test'] is False
            and build['cli_event']['features'] == [] and build['cli_event']['target']['kind'] == ['bin']
            and build['cli_event']['target']['name'] == 'lila' and build['ordinary_cli_acceptance'] is False,
            'Ordinary build preparation graph differs')
    identity = build['worker']['identity']
    env = dict(build['engine_event']['env'])
    require(identity['source_revision'] == {'kind': 'git-commit', 'commit': report['head']}
            and env['LILA_COMPILER_SOURCE_REVISION'] == 'git:' + report['head']
            and identity['source_fingerprint'] == env['LILA_COMPILER_FINGERPRINT']
            and identity['source_fingerprint_scheme'] == env['LILA_COMPILER_FINGERPRINT_SCHEME']
            and identity['executable_sha256'] == build['worker']['sha256'] == build['replay_compiler_sha256'],
            'Portable worker/producer identities differ')
    require(report['outer']['exit_code'] == (2 if counts['InfrastructureError'] else int(counts['Success'] != 53))
            and report['outer']['source_before'] == report['outer']['source_after'], 'Actual outer closure differs')
    for name in ('stdout', 'stderr'):
        require(len(assets[report['outer'][name]['raw_sha256']]) == report['outer'][name]['raw_bytes'],
                'Embedded outer raw stream differs')
    snapshots = {}
    for row in rows:
        reference = row['native_snapshot_sha256']
        require(row['raw_transcript_sha256'] in assets, 'Actual native transcript is absent')
        if reference is None:
            require(row['outcome'] == 'InfrastructureError' and row['reason'], 'Missing native row became green')
            continue
        if reference not in snapshots:
            snapshot = json.loads(assets[reference])
            require(snapshot['snapshot_version'] == 8 and snapshot['execution_backend'] == 'wasm-aot'
                    and snapshot['run_kind'] == 'full' and snapshot['compiler_identity'] == identity
                    and snapshot['pinned_revisions'] == report['pinned_revisions'], 'Native snapshot identity differs')
            completed = snapshot['completed_test_ids']
            verdicts = {x: 'Success' for x in completed}
            details = {}
            require(len(completed) == len(set(completed)) == snapshot['total'], 'Native snapshot duplicate IDs')
            for failure in snapshot['failures']:
                execution = failure['test_id']
                require(execution in verdicts and execution not in details and failure['outcome'] != 'Success',
                        'Native failure identities differ')
                verdicts[execution] = failure['outcome']
                details[execution] = failure['detail']
            require({key: sum(value == key for value in verdicts.values()) for key in snapshot['counts_per_outcome']}
                    == snapshot['counts_per_outcome'], 'Native snapshot counts differ')
            snapshots[reference] = (verdicts, details)
        verdicts, details = snapshots[reference]
        require(verdicts[row['execution_id']] == row['outcome']
                and details.get(row['execution_id']) == row['reason'], 'Published native verdict/reason differs')
    for invocation in report['native_invocations']:
        command = invocation['command']
        require(command[command.index('--threads') + 1] == '1'
                and command[command.index('--timeout-ms') + 1] == '60000'
                and command[-1] == '--reuse-case-workers'
                and command[1:3] == ['--jobs', '1']
                and invocation['exit_code'] in (0, 1)
                and invocation['stdout_sha256'] in assets and invocation['stderr_sha256'] in assets,
                'Actual native command or raw stream differs')
    prior_verdicts, prior_details = {}, {}
    for reference in report['baseline']['native_snapshot_sha256']:
        snapshot = json.loads(assets[reference])
        require(snapshot['compiler_identity'] == report['baseline']['compiler_identity']
                and snapshot['compiler_identity']['source_revision'] == report['baseline']['source_revision'],
                'Embedded prior native identity differs')
        completed = snapshot['completed_test_ids']
        require(len(completed) == len(set(completed)) == snapshot['total'], 'Prior native IDs differ')
        values = {x: 'Success' for x in completed}
        for failure in snapshot['failures']:
            require(failure['test_id'] in values and failure['test_id'] not in prior_details,
                    'Prior native failure IDs differ')
            values[failure['test_id']] = failure['outcome']
            prior_details[failure['test_id']] = failure['detail']
        require({key: sum(value == key for value in values.values()) for key in snapshot['counts_per_outcome']}
                == snapshot['counts_per_outcome'], 'Prior native counts differ')
        prior_verdicts.update(values)
    absent = set(report['baseline']['native_absent_execution_ids'])
    require(not set(prior_verdicts).intersection(absent)
            and len(set(prior_verdicts).union(absent)) == report['baseline']['total'] == 5365
            and not set(ids).intersection(absent), 'Actual prior denominator differs')
    for row in rows:
        require(row['prior_outcome'] == prior_verdicts.get(row['execution_id'])
                and row['prior_reason'] == (prior_details.get(row['execution_id'])
                    if row['execution_id'] in prior_verdicts else 'Absent from the actual historical 5365 denominator'),
                'Prior transition or reason differs from embedded actual snapshot')
    return {'verified': True, 'total': 53, 'outcomes': report['outcomes'],
            'probe_passed': report['ordinary_cli_probe53_passed'], 'embedded_raw_streams': len(assets)}


def predecessor(assets):
    """Only execute the exact previously reviewed Python reader, with its main disabled."""
    require(D2_SHA in assets and D2_READER_SHA in assets, 'Exact D2 proof and reader missing')
    source = assets[D2_READER_SHA]
    require(digest(source) == D2_READER_SHA, 'Predecessor reader identity differs')
    namespace = {'__name__': 'embedded_hash_pinned_D2_reader'}
    exec(compile(source, '<hash-pinned D2 offline verifier>', 'exec'), namespace)
    prior = json.loads(assets[D2_SHA])
    verdict = namespace['verify'](prior)
    require(verdict['source_batch_objective_met'] is True, 'Prior D2 proof did not verify')
    oldassets = namespace['decode_assets'](prior)
    f5 = json.loads(oldassets[namespace['F5_SHA']])
    return prior, f5


def compare_prior(report, prior, f5, policy):
    rows = {row['execution_id']: row for row in report['results']}
    ids = [row['execution_id'] for row in report['results']]
    oldids = [row['execution_id'] for row in prior['results']]
    shadow = policy['shadow8_ids']
    require(ids == oldids + shadow and oldids == policy['old45_ids']
            and len(shadow) == len(set(shadow)) == 8 and not set(oldids).intersection(shadow),
            'Preserve the exact ordered old45 prefix and eight new pinned modes')
    old43 = sorted(row['execution_id'] for row in prior['results'] if row['outcome'] == 'Success')
    old29 = sorted(row['execution_id'] for row in f5['results'] if row['outcome'] == 'Success')
    known = set(old43) | set(shadow)
    require(len(old43) == 43 and len(old29) == 29 and set(old29) <= set(old43) and len(known) == 51
            and all(rows[name]['outcome'] == 'Success' for name in known), 'A prior or new required passing mode regressed')
    require(all(rows[name]['outcome'] in ('Success', 'NotImplemented') for name in NI2),
            'Generator modes must be literal Success or typed NotImplemented')
    unsupported = sorted(name for name in NI2 if rows[name]['outcome'] == 'NotImplemented')
    require(all('unsupported dynamic-source operation `GeneratorFunction`' in (rows[name]['reason'] or '')
                for name in unsupported), 'An unsupported generator diagnostic was relabelled')
    require(all(report['fixture_sha256'].get(name) == value for name, value in prior['fixture_sha256'].items())
            and report['fixture_sha256'] == policy['fixture_sha256']
            and report['pinned_revisions'] == prior['pinned_revisions'] == PINNED_REVISIONS
            and policy['pinned_revisions'] == PINNED_COMMITS
            and report['native_inventory']['sha256'] == prior['native_inventory']['sha256']
            and report['native_inventory']['count'] == prior['native_inventory']['count'] == 102994
            and report['baseline'] == prior['baseline'], 'Original fixtures/pins/historical baseline differ')
    oldrows = {row['execution_id']: row for row in prior['results']}
    expected_groups = {name: value['ids'] for name, value in prior['groups'].items()}
    expected_groups['shadowRealm8'] = shadow
    require({name: value['ids'] for name, value in report['groups'].items()} == expected_groups,
            'Original group memberships or ShadowRealm8 registration differ')
    transitions = dict(Counter(oldrows[name]['outcome'] + '->' + rows[name]['outcome'] for name in oldids))
    return {'prior_d2_portable_sha256': D2_SHA, 'prior_f5_portable_sha256': prior['prior_f5_portable']['sha256'],
            'retained_d2_success_ids': old43, 'retained_d2_success_count': 43,
            'retained_f5_success_ids': old29, 'retained_f5_success_count': 29,
            'shadow8_success_ids': shadow, 'shadow8_success_count': 8,
            'known_success_count': 51, 'regressions': [], 'transitions_from_D2': transitions,
            'remaining_typed_Generator_NI_ids': unsupported,
            'generator_success_ids': sorted(NI2 - set(unsupported)),
            'actual_outer_exit': report['outer']['exit_code'],
            'all53_passed': report['ordinary_cli_probe53_passed'],
            'full5365_or_full_pin_acceptance': False}


def normal_provenance(report, assets):
    build = report['ordinary_build_preparation']
    for path, expected in report['fixture_sha256'].items():
        require(expected in assets and digest(assets[expected]) == expected,
                'Actual pinned fixture bytes absent or changed: ' + path)
    build = report['ordinary_build_preparation']
    stage = json.loads(assets[build['owner_raw_sha256']])
    require(stage['schema'] == 'ordinary-default-cli-owner-v2' and stage['head'] == HEAD
            and all(stage[key] is True for key in ('complete', 'passed', 'source_unchanged', 'protected_images_unchanged'))
            and stage['ordinary_cli_acceptance'] is stage['oracle'] is False
            and stage['frozen_sources_before'] == stage['frozen_sources_after']
            and len(stage['frozen_sources_before']) == 3
            and all(row['patch_sha256'] == hashlib.sha256(b'').hexdigest() and not row['untracked']
                    for row in stage['frozen_sources_before'].values()),
            'Actual ordinary source/build ownership is not clean and closed')
    require(build['command'] == ['python3', '-B', 'scripts/limited_verification.py', '--cloud', '--cloud-cpus', 'auto', '--',
            'cargo', 'build', '--locked', '--offline', '--release', '-p', 'lila-cli', '--bin', 'lila',
            '--message-format=json-render-diagnostics', '--timings'], 'Actual normal Cargo build argv differs')
    messages = [json.loads(line) for line in assets[build['stdout_sha256']].splitlines() if line.startswith(b'{')]
    artifacts = [event for event in messages if event.get('reason') == 'compiler-artifact']
    require(stage['compiler_artifacts'] == artifacts and not any(event['profile']['test']
            or event['target']['name'] == 'boa_engine' or 'spec-exec-oracle' in event.get('features', []) for event in artifacts),
            'Test/oracle graph cannot own ordinary CLI results')
    paths = {str(Path(name)) for event in artifacts for name in event.get('filenames', [])}
    require(paths == {row['path'] for row in stage['dependency_artifacts']}
            and all(row['sha256'] == stage['protected_inputs'][row['path']]['sha256']
                    for row in stage['dependency_artifacts'] + stage['runtime_bundle']),
            'Actual normal dependency artifact/static runtime coverage differs')
    require(stage['actual_cli_artifact'] == build['cli_event'] and stage['actual_engine_producer'] == build['engine_event']
            and stage['actual_runtime_producer'] == build['runtime_event'] and stage['runtime_bundle'] == build['runtime_bundle'],
            'Published actual selected producers/static R/P differ')
    runtime = build['runtime_event']
    require('static:-bundle=lila_runtime_images' in runtime['linked_libs']
            and 'native=' + runtime['out_dir'] in runtime['linked_paths']
            and {Path(row['path']).name for row in stage['runtime_bundle']} == {
                'liblila_runtime_images.a', 'lila-runtime-package.bin', 'lila-runtime-native.bin',
                'lila-runtime-manifest.bin', 'image_bindings.rs', 'image_data.rs'}
            and all(str(Path(row['path']).parent) == runtime['out_dir'] for row in stage['runtime_bundle']),
            'Actual static archive/package/native producer bundle differs')
    worker, ordinary = stage['native_cli_worker'], stage['ordinary_cli']
    require(worker['sha256'] == ordinary['sha256'] == build['worker']['sha256'] == build['replay_compiler_sha256']
            and worker['identity'] == ordinary['identity'] == build['worker']['identity']
            and worker['path'] != ordinary['path'] and ordinary['path'] == build['cli_event']['executable'],
            'Actual ordinary CLI and distinct identical worker copy differ')
    for command in stage['commands']:
        require(command['complete'] is True and command['exit_code'] == 0
                and all(command[name + '_sha256'] in assets for name in ('stdout', 'stderr')),
                'Actual ordinary build/identity/readelf stream is missing or failed')
    linkage = assets[stage['readelf_run']['stdout_sha256']].decode()
    require(not re.search(r'\(NEEDED\).*\[liblila_runtime_images_', linkage),
            'A shared-runtime test leaf cannot own ordinary CLI')
    require(json.loads(assets[stage['worker_identity_run']['stdout_sha256']]) == worker['identity'],
            'Actual worker compiler-identity raw bytes differ')
    inventory = assets[report['native_inventory']['embedded_raw_sha256']].decode().splitlines()
    require(inventory and inventory[0] == 'count: ' + str(report['native_inventory']['count'])
            and len(inventory) - 1 == len(set(inventory[1:])) == report['native_inventory']['count']
            and all(re.fullmatch(r'(?:sloppy-script|strict-script|raw-script|module|raw-module):[^\s]+\.js', execution)
                    for execution in inventory[1:])
            and set(current['execution_id'] for current in report['results']) <= set(inventory[1:])
            and report['native_inventory']['present_requested'] == 53
            and report['native_inventory']['absent_requested'] == [],
            'Actual complete list-all inventory/requested53 availability differs')


def actual_replay(report, assets, policy):
    """Bind presentation rows to original manifest, summary, saved rows and native snapshot."""
    refs = report['actual_replay']
    run = json.loads(assets[refs['run_sha256']])
    summary = json.loads(assets[refs['summary_sha256']])
    outer = json.loads(assets[report['outer']['owner_sha256']])
    stage = json.loads(assets[report['ordinary_build_preparation']['owner_raw_sha256']])
    worker = stage['native_cli_worker']
    ids = assets[INPUT_SHA].decode().splitlines()
    require({k: v for k, v in summary.items() if k not in ('total', 'outcomes', 'results')} == run,
            'Embedded summary and run manifest differ')
    require(run['execution_list_sha256'] == INPUT_SHA and assets[refs['executions_sha256']] == assets[INPUT_SHA]
            and refs['batch_executions_sha256'] == run['batch_execution_list_sha256'] == INPUT_SHA and run['batch_executions'] == ids
            and run['absent_executions'] == [] and run['fixture_sha256'] == report['fixture_sha256']
            and run['binary_sha256'] == worker['sha256'] and run['compiler_identity'] == worker['identity']
            and run['binary_source'] == worker['path'] and run['execution_list_source'] == policy['input']['path']
            and run['batch_threads'] == 1 and run['replay_mode'] == 'native-execution-list-v1'
            and run['suite_root'] == policy['suite_root'], 'Actual replay source/IDs/worker/fixtures differ')
    replayroot = Path(run['binary']).parent
    require(run['binary'] == str(replayroot / 'compiler')
            and run['execution_list'] == str(replayroot / 'executions')
            and run['batch_execution_list'] == str(replayroot / 'batch-executions'), 'Actual replay owned paths differ')
    saved_rows = {row['execution_id']: row for row in summary['results']}
    require(summary['total'] == len(summary['results']) == len(saved_rows) == 53 and set(saved_rows) == set(ids)
            and summary['outcomes'] == {name: report['outcomes'][name] for name in OUTCOMES[:-1]},
            'Original summary rows/counts differ')
    require(outer['complete'] is True and outer['source_unchanged'] is True and outer['head'] == HEAD
            and outer['native_acceptance'] is False and outer['source_before'] == outer['source_after']
            and outer['source_before'] == {'head': HEAD, 'tracked_patch_sha256': EMPTY, 'untracked': {}}
            and outer['driver_sha256'] == policy['outer_driver']['sha256']
            and outer['cwd'] == '/workspace/porffor-next' and outer['cargo_target_directory'] == '/workspace/porffor/target',
            'Original outer source/driver identity differs')
    for name in ('command', 'exit_code', 'seconds', 'source_before', 'source_after'):
        require(outer[name] == report['outer'][name], 'Outer presentation differs from actual owner')
    for name in ('stdout', 'stderr'):
        require({k: v for k, v in outer[name].items() if k != 'path'} == report['outer'][name],
                'Outer transcript metadata differs')
    command = ['taskset', '-c', '3', 'python3', '-B', 'scripts/limited_verification.py', '--cloud', '--',
               'python3', '-B', 'scripts/replay-test262-executions.py', policy['input']['path'],
               '--binary', worker['path'], '--suite-root', policy['suite_root'],
               '--output-dir', str(replayroot), '--workers', '1', '--reuse-case-workers']
    require(outer['command'] == command, 'Actual outer replay argv differs')
    require(len(report['native_invocations']) == 1, 'Need one actual native run-list invocation')
    call = report['native_invocations'][0]
    expected = [run['binary'], '--jobs', '1', 'test262', 'run-list', run['batch_execution_list'],
                '--execution-backend', 'wasm-aot', '--suite-root', run['suite_root'],
                '--snapshot-dir', str(replayroot / 'snapshots'), '--snapshot-name', 'exact-batch',
                '--threads', '1', '--timeout-ms', '60000', '--reuse-case-workers']
    require(call['command'] == expected and call['exit_code'] == outer['exit_code'],
            'Native command/actual exit differs')
    snapshots = set()
    details = {}
    for row in report['results']:
        execution = row['execution_id']
        saved = saved_rows[execution]
        require(json.loads(assets[refs['saved_result_sha256'][execution]]) == saved
                and saved['execution_id'] == execution and saved['protocol'] == 'native-execution-list-v1'
                and saved['outcome'] == row['outcome'] and saved['exit_code'] == row['result_exit_code']
                and type(saved['exit_code']) is int and saved['exit_code'] == int(saved['outcome'] != 'Success')
                and not saved.get('infrastructure_error')
                and saved['transcript_sha256'] == row['raw_transcript_sha256'] == call['stdout_sha256']
                and saved['transcript'] == call['stdout'], 'Saved native result or presentation differs')
        ref = saved['native_snapshot']['sha256']
        require(ref == row['native_snapshot_sha256'] and ref in assets
                and json.loads(assets[refs['invocation_sha256'][saved['invocation']]]) == call,
                'Original saved snapshot/invocation reference differs')
        snapshots.add(ref)
    require(len(snapshots) == 1, 'Need one actual native53 snapshot')
    snapshot = json.loads(assets[next(iter(snapshots))])
    require(len(snapshot['completed_test_ids']) == len(set(snapshot['completed_test_ids'])) == snapshot['total'] == 53
            and set(snapshot['completed_test_ids']) == set(ids),
            'Native completed inventory differs from requested exact53')
    for failure in snapshot['failures']:
        details[failure['test_id']] = failure
    unsupported = {row['execution_id'] for row in report['results'] if row['outcome'] == 'NotImplemented'}
    require(set(details) == unsupported and unsupported <= NI2
            and all(details[name]['kind'] == 'Unsupported' and details[name]['outcome'] == 'NotImplemented'
                    and 'unsupported dynamic-source operation `GeneratorFunction`' in details[name]['detail']
                    for name in unsupported), 'Native unsupported kind/reason differs')
    inventory = assets[report['native_inventory']['embedded_raw_sha256']].decode().splitlines()
    require(report['native_inventory']['count'] == 102994
            and run['native_inventory']['stdout_sha256'] == report['native_inventory']['sha256']
            == report['native_inventory']['embedded_raw_sha256']
            and run['native_inventory']['command'] == [run['binary'], '--jobs', '1', 'test262', 'list-all',
                '--suite-root', run['suite_root'], '--execution-backend', 'wasm-aot']
            and len({name.split(':', 1)[1] for name in inventory[1:]}) == 53616,
            'Require the complete pinned list-all inventory, without denominator reductions')


def queue_closure(report, assets, policy):
    queue = json.loads(assets[report['queue_owner_sha256']])
    exported = report['export_source_index_fences']
    require(exported['before'] == exported['after'], 'Export source or real index changed')
    source = {path: {key: value for key, value in row.items() if key != 'real_index'}
              for path, row in exported['before'].items()}
    require(queue['schema'] == 'finite-shadow-runtime-cli53-owner-v1' and queue['complete'] is True
            and queue['head'] == HEAD and queue['stage'] == 'closed-actual-probe53'
            and queue['source_unchanged'] is True and queue['driver_sha256'] == V3_ADOPTION_SHA
            and queue['packet_sha256'] == V3_PACKET_SHA and not queue.get('error') and not queue.get('final_guard_error')
            and queue['frozen_sources_before'] == queue['frozen_sources_after'] == source,
            'CLI53 queue must be actually closed with unchanged frozen sources')
    require(not any(queue[key] for key in ('historical_replay_launched', 'full_pin_replay_launched',
            'full_pin_execution_verified', 'full_pinned_suite_or_task_closure')),
            'CLI53 scoped proof cannot assert long-run acceptance')
    build = report['ordinary_build_preparation']
    require(queue['ordinary_build_owner']['sha256'] == build['owner_raw_sha256']
            and queue['predecessor_probe45']['sha256'] == D2_SHA
            and queue['probe53_gate'] == report['probe53_gate'], 'Actual queue owners/gate differ')
    gate = report['probe53_gate']
    unsupported = report['comparison_to_D2_f5_and_original0a18']['remaining_typed_Generator_NI_ids']
    require(gate['passed_launch_gate'] is True
            and all(gate[key] is True for key in ('all_51_known_successes_retained', 'all_43_d2_successes_retained',
                    'all_29_f5_successes_retained', 'all_8_shadowrealm_modes_passed'))
            and gate['ordinary_cli_probe53_all_passed'] is report['ordinary_cli_probe53_passed']
            and gate['outcomes'] == report['outcomes'] and gate['literal_native_and_outer_exit'] == report['outer']['exit_code']
            and gate['typed_generator_unsupported_ids'] == unsupported
            and gate['generator_success_ids'] == sorted(NI2 - set(unsupported))
            and gate['outer_owner']['sha256'] == report['outer']['owner_sha256']
            and gate['summary']['sha256'] == report['actual_replay']['summary_sha256']
            and gate['run_manifest']['sha256'] == report['actual_replay']['run_sha256'],
            'Published literal gate differs from actual rows/owners')
    corrected_pin_closure(queue, assets, source)


def corrected_pin_closure(queue, assets, source):
    """Preserve the failed V2 gate and independently check the narrow read-only correction."""
    require(V3_PACKET_SHA in assets and V3_ADOPTION_SHA in assets and V3_QUEUE_SHA in assets
            and QUEUE_SHA in assets and PACKET_SHA in assets,
            'Corrected closure source/manifest evidence missing')
    require(queue['queue_driver']['sha256'] == V3_QUEUE_SHA
            and queue['original_v2_queue_driver']['sha256'] == QUEUE_SHA
            and queue['original_v2_packet']['sha256'] == PACKET_SHA
            and queue['commands'] == [] and queue['native_executed'] is False
            and type(queue['cargo_or_native_invocations']) is int and queue['cargo_or_native_invocations'] == 0
            and queue['original_v2_pin_error_independently_reproduced'] is True,
            'The corrected closure must preserve original provenance and execute no native commands')
    original = json.loads(assets[queue['predecessor_v2_failure']['sha256']])
    require(original['schema'] == 'finite-shadow-runtime-cli53-owner-v1' and original['complete'] is True
            and original['stage'] == 'closed-error' and original['head'] == HEAD
            and original['driver_sha256'] == QUEUE_SHA and original['packet_sha256'] == PACKET_SHA
            and original['source_unchanged'] is True and not original.get('final_guard_error')
            and original['frozen_sources_before'] == original['frozen_sources_after'] == source
            and original['error'].endswith('ValueError: Actual53 native snapshot pin differs\n')
            and 'probe53_gate' in original['error'] and 'probe_gate' in original['error']
            and 'probe53_gate' not in original
            and original['historical_replay_launched'] is original['full_pin_replay_launched'] is False,
            'Original V2 failure must remain the precise closed pin-comparison RED')
    for key in ('plan', 'ordinary_build_owner', 'accepted_default_audit', 'accepted_all_features_audit',
                'predecessor_probe45', 'native_audit_admission'):
        require(queue[key] == original[key], 'Read-only correction changed existing actual evidence')
    commands = original['commands']
    require([row['label'] for row in commands] == ['producer_snapshot', 'normal_build', 'normal_build_verify', 'probe53'],
            'Original V2 must retain its exact four actual commands')
    for row in commands:
        require(row['complete'] is True and row['command'] == original['plan']['commands'][row['label']]
                and type(row['exit_code']) is int
                and row['exit_code'] == (queue['probe53_gate']['literal_native_and_outer_exit']
                                        if row['label'] == 'probe53' else 0),
                'Original completed command or actual exit differs')
        for name in ('stdout', 'stderr'):
            ref = row[name]
            require(len(assets[ref['sha256']]) == ref['bytes'], 'Original V2 raw command stream absent or changed')
    binding = queue['pin_binding']
    require(binding['schema'] == 'cli53-native-tree-pin-binding-v3' and binding['verified'] is True
            and binding['source_pins']['sha256'] == SOURCE_PINS_SHA
            and binding['source_pins']['git_blob'] == 'ced333f13fb3adf0303a8f7a024781e869dd1020'
            and binding['source_pins']['value'] == json.loads(assets[SOURCE_PINS_SHA])
            and binding['compiler_head'] == HEAD
            and binding['compiler_vendored_tree'] == PINNED_REVISIONS['test262']
            and binding['native_pinned_revisions'] == PINNED_REVISIONS
            and binding['upstream_commit'] == PINNED_COMMITS['test262']
            and binding['suite_or_input_changed'] is False,
            'Commit/content-tree correction lost its exact compiler/pin provenance')
    suite = binding['execution_suite']
    require(suite == {'path': '/workspace/porffor/test262/vendor/test262', 'repo': '/workspace/porffor',
            'head': source['/workspace/porffor']['head'], 'prefix': 'test262/vendor/test262',
            'tree': PINNED_REVISIONS['test262'], 'source_clean': True},
            'Execution-suite Git mapping differs from the frozen actual tree')


def verify(report):
    require(report['input']['sha256'] == INPUT_SHA and report['head'] == HEAD, 'Current source/input identity differs')
    result = _verify_common(report, SCHEMA)
    assets = decode_assets(report)
    require(report['full_workspace_native_acceptance'] is False and report['binary_payloads_embedded'] is False
            and report['source_batch_objective_met'] is True, 'Scope or binary-presence claim was widened')
    policy = json.loads(assets[POLICY_SHA])
    pins = json.loads(assets[SOURCE_PINS_SHA])
    require(pins == {'ecma262': PINNED_COMMITS['ecma262'], 'test262_upstream': PINNED_COMMITS['test262'],
                    'test262_tree': PINNED_REVISIONS['test262']},
            'Embedded source pins must distinguish the upstream commit from the actual tree')
    require(policy['input']['sha256'] == INPUT_SHA and report['outcomes'].keys() == dict.fromkeys(OUTCOMES).keys(),
            'Exact sealed input/outcome categories differ')
    prior, f5 = predecessor(assets)
    require(report['prior_d2_portable'] == {'sha256': D2_SHA, 'bytes': len(assets[D2_SHA])}
            and report['comparison_to_D2_f5_and_original0a18'] == compare_prior(report, prior, f5, policy),
            'Published retention/transitions differ from actual predecessor')
    normal_provenance(report, assets)
    stage = json.loads(assets[report['ordinary_build_preparation']['owner_raw_sha256']])
    build = report['ordinary_build_preparation']
    require(stage['driver_sha256'] == BUILDER_SHA and report['ordinary_build_preparation']['driver_sha256'] == BUILDER_SHA
            and not stage.get('controller_error') and not stage.get('final_guard_error'),
            'Normal builder identity/guard differs')
    require(all(stage['build'][key] == build[key] for key in ('command', 'exit_code', 'seconds', 'stdout_sha256', 'stderr_sha256'))
            and stage['build']['complete'] is True
            and build['worker'] == {key: stage['native_cli_worker'][key] for key in ('path', 'sha256', 'bytes', 'identity')},
            'Published build/worker details differ from actual normal owner')
    require(build['cli_event']['profile']['opt_level'] == '3'
            and build['cli_event']['target']['src_path'] == '/workspace/porffor-next/crates/lila-cli/src/bin/lila.rs'
            and build['cli_event']['executable'] == '/workspace/porffor/target/release/lila'
            and Path(build['runtime_event']['out_dir']).parent.parent == Path('/workspace/porffor/target/release/build'),
            'Actual optimized default official CLI or runtime producer path differs')
    actual_replay(report, assets, policy)
    queue_closure(report, assets, policy)
    return {**result, 'current_source': HEAD, 'retained_d2_successes': 43, 'retained_f5_successes': 29,
            'shadow8_successes': 8, 'typed_Generator_NI_preserved': report['outcomes']['NotImplemented'],
            'actual_outer_exit': report['outer']['exit_code'], 'all53_passed': report['ordinary_cli_probe53_passed'],
            'source_batch_objective_met': True, 'binary_future_presence_claimed': False,
            'full5365_or_full_pin_acceptance': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('checkpoint', type=Path)
    args = parser.parse_args()
    require(args.checkpoint.stat().st_size <= 2 * LIMIT, 'Portable file size exceeds bound')
    print(json.dumps(verify(json.loads(args.checkpoint.read_bytes())), indent=2))
