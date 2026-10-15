#!/usr/bin/env python3
"""Offline next-workspace proof reader; derived from the sealed V8b/V9 readers."""
import argparse
import base64
from collections import Counter
import gzip
import hashlib
import io
import json
import os
from pathlib import Path
import re
import zlib

HEAD = '8e9a8c8210f99d8d14b4267461ff42079b603c00'
ROOT = '/workspace/porffor-next/'
LIMIT = 64 * 1024 * 1024
DEFAULT_SCHEMA = 'real_workspace_actual_retention_current9127_v10'
AF_SCHEMA = 'real_workspace_actual_all_features_mode_retention_current9220_v11'
PACK_SCHEMA = 'portable-real-next-workspace-v1-20261015'
D2_DEFAULT_SHA = '16996d6fb06d322846729d31b81bb7683d18de375eb7f62354ae0dfb47afb5f5'
D2_AF_SHA = 'edae3a4478c61b5130dddb360b25b21a7257f027a475d6a7525e3996a0aff8c5'
D2_DEFAULT_READER_SHA = '2a71581d07ab80f207cfa66266290252fe3ec215fc6b2a438b5b10b41995e26a'
D2_AF_READER_SHA = '9cd5e5628420f505d3ba743d2bb900057321ddb930b3aa1a3cedf899c23988e2'
NEW_MANIFEST_SHA = '48beb7386057e6b8e43ffd3aaf7d542a3adbbd670bbec97eb0f0ea189973164c'
PARTITION_DRIVER_SHA = '9726e973b2fb94fbe98e3f8964925d25b5ce78374a3475b6a31a6df47bb4f39c'
SCHEDULE_SHA = '570c7694e32a9ef202ad7f5d01a48a4e14ff7012ea27ce54bd93aabfb542ad20'
SNAPSHOT_SHA = '8430fcba98e622ac3b4de98eed7c6b77aaae8a6855d1efd706011401787ed157'
MASTER_MANIFEST = '/workspace/porffor/target/combined-shadow-runtime-iteration-private-d2f4bc51-20261014/manifest-v4.json'
MASTER_MANIFEST_SHA = '8cdb204d4426566156bd8c0e03778f37422df325fd8648a192cc9f83b0b6c2de'
EXTRA_MANIFEST = '/workspace/porffor/target/finite-generator-global-hints-private-da180627-20261015/manifest-v2.json'
EXTRA_MANIFEST_SHA = 'c902bce81da08383f74090245240738c0c179624ec933ff31160fd315d1b3d0c'
CORRECTION_MANIFEST = '/workspace/porffor/target/finite-generator-reader-hypotheses-private-55ccda18-20261015/manifest.json'
CORRECTION_MANIFEST_SHA = 'c6aba80c455751db257e6e9f670059cfdc59171aa786bdbceaeb4db5e8f40bad'


def require(value, message):
    if not value:
        raise ValueError(message)


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def unpack(row):
    require(row['encoding'] == 'base64(zlib(original bytes))' and 0 <= row['bytes'] <= LIMIT,
            'Unknown pack encoding/read bound')
    packed = base64.b64decode(row['data'], validate=True)
    stream = zlib.decompressobj()
    raw = stream.decompress(packed, LIMIT + 1)
    require(len(raw) == row['bytes'] and stream.eof and not stream.unused_data
            and not stream.unconsumed_tail and sha(raw) == row['sha256'],
            'Embedded raw bytes are corrupt/truncated')
    return raw


def inventory(raw):
    names = re.findall(r'^(.+): test$', raw.decode(), re.M)
    require(len(names) == len(set(names)), 'Duplicate qualified inventory ID')
    return names


def outcomes(raw, names):
    text = raw.decode()
    starts = list(re.finditer(r'^test (.+?) \.\.\. ', text, re.M))
    values = {}
    for index, start in enumerate(starts):
        end = starts[index + 1].start() if index + 1 < len(starts) else len(text)
        block = text[start.end():end].split('\nfailures:', 1)[0]
        terminals = re.findall(r'^(ok|FAILED|ignored)(?:\s+[^\n]*)?$', block, re.M)
        name = start[1].removesuffix(' - should panic').removesuffix(' - compile fail')
        require(len(terminals) == 1 and name not in values, 'Missing/duplicate actual native terminal')
        values[name] = terminals[0]
    counts = {'passed': sum(value == 'ok' for value in values.values()),
              'failed': sum(value == 'FAILED' for value in values.values()),
              'ignored': sum(value == 'ignored' for value in values.values())}
    summaries = re.findall(r'^test result: .*? (\d+) passed; (\d+) failed; (\d+) ignored;', text, re.M)
    require(set(values) == set(names) and len(summaries) == 1
            and tuple(counts.values()) == tuple(map(int, summaries[0])), 'Raw full inventory/summary differs')
    return values, counts


def canonical(path):
    return path if path.startswith('/') else ROOT + path


class RawStore:
    """Exact path/SHA identity; inherited bytes never imply a new execution."""
    def __init__(self, document):
        self.payloads = document['content_addressed_raw_payloads']
        self.index = {}
        self.decoded = {}
        for row in document['raw_streams']:
            key = (row['original_path'], row['payload_sha256'])
            require(key not in self.index and key[1] in self.payloads, 'Duplicate/missing path-SHA raw stream')
            require(row['bytes'] == self.payloads[key[1]]['bytes'], 'Raw stream size differs')
            self.index[key] = row
        require(set(self.payloads) == {key[1] for key in self.index}, 'Unreferenced/missing content payload')

    def get(self, path, digest=None):
        path = canonical(path)
        if digest is None:
            keys = [key for key in self.index if key[0] == path]
            require(len(keys) == 1, 'Missing/ambiguous path without expected SHA: ' + path)
            digest = keys[0][1]
        key = (path, digest)
        require(key in self.index, 'Missing exact path-SHA raw proof: ' + str(key))
        if digest not in self.decoded:
            raw = unpack(self.payloads[digest])
            require(sha(raw) == digest, 'Content address differs')
            self.decoded[digest] = raw
        return self.decoded[digest]

    def verify_all(self):
        for path, digest in self.index:
            require(len(self.get(path, digest)) == self.index[path, digest]['bytes'], 'Raw size differs')


def source_cfg(row, raw, metadata):
    require(row['source_head'] == HEAD and sha(raw) == row['source_sha256']
            and len(raw) == row['source_bytes'], 'Mode-specific source/head differs')
    name = row['id'].split('::')[-1]
    pattern = re.compile(r'(?P<attrs>(?:\s*#\[[^\]]*\])+)(?:\s*)fn\s+' + re.escape(name) + r'\s*\(')
    matches = list(pattern.finditer(raw.decode()))
    require(len(matches) == 1 and matches[0].group(0) == row['attribute_and_function_prefix']
            and set(re.findall(r'#\[[^\]]*\]', matches[0]['attrs']))
                == {'#[test]', '#[cfg(not(feature = "spec-exec-oracle"))]'}
            and row['disabled_when_feature'] == 'spec-exec-oracle'
            and row['cfg_expression'] == 'not(feature = "spec-exec-oracle")',
            'Only the exact source-authoritative negated oracle gate qualifies')
    package = next(item for item in metadata['packages'] if item['name'] == row['scope'][0])
    require(package['id'] in metadata['workspace_members'] and 'spec-exec-oracle' in package['features']
            and row['source_path'].startswith('crates/' + package['name'] + '/'),
            'Source-gated test package feature differs')


def qualified_default_ids(audit, default_audit, registry, observations, metadata, raw):
    require(registry['head'] == HEAD and registry['schema'] == 'source_authoritative_default_only_oracle_cfg_registry_v1'
            and len(registry['rows']) == 4, 'Exact four-test source registry differs')
    current = {tuple(scope['scope']): scope for scope in audit['scopes']}
    prior = {tuple(scope['scope']): scope for scope in default_audit['scopes']}
    declared = {(*row['scope'], row['id']): row for row in audit['source_proven_default_only_ids']}
    require(len(declared) == 4, 'Duplicate/missing qualified default-only ID')
    qualified = {}
    for row in registry['rows']:
        source_cfg(row, raw(ROOT + row['source_path'], row['source_sha256']), metadata)
        scope, name = tuple(row['scope']), row['id']
        key = (*scope, name)
        require(key in declared and prior[scope]['verdicts'].get(name) == 'ok'
                and name not in observations[scope]
                and 'spec-exec-oracle' not in prior[scope]['actual_Cargo_artifact']['features']
                and 'spec-exec-oracle' in current[scope]['actual_Cargo_artifact']['features'],
                'Default-only ID lacks actual default PASS and enabled-feature all-features absence')
        expected = {'scope': list(scope), 'id': name, 'source_path': row['source_path'],
            'source_sha256': row['source_sha256'], 'cfg_expression': row['cfg_expression'],
            'actual_enabled_feature': row['disabled_when_feature'], 'actual_default_outcome': 'ok',
            'default_audit': audit['same_source_default_audit'],
            'default_raw_inventory': prior[scope]['raw_inventory'],
            'default_raw_transcript': prior[scope]['raw_transcript'],
            'all_features_registration': 'absent because this exact source test is cfg-disabled',
            'all_features_native_outcome': None, 'all_features_native_execution_claimed': False}
        require(declared[key] == expected, 'Source-proven default-only record was widened or changed')
        qualified[key] = expected
    require(set(qualified) == set(declared), 'Unrelated missing ID was treated as cfg-disabled')
    return qualified


def verify_retention(rows, prior, observations, disabled, expected_total, expected_disabled):
    reports = {tuple(row['scope']): row for row in rows}
    require(len(reports) == len(rows) and set(reports) == set(prior), 'Retention scope projection differs')
    total = retained = excluded = 0
    for scope, old in prior.items():
        ids = sorted(name for name, value in old['verdicts'].items() if value == 'ok')
        cfg_names = [name for name in ids if (*scope, name) in disabled]
        eligible = [name for name in ids if name not in cfg_names]
        report = reports[scope]
        require(report['prior_passes'] == ids and report['eligible_in_all_features'] == eligible
                and report['retained_passes'] == eligible and not report['observed_regressions']
                and not report['unobserved_prior_ids']
                and report['source_proven_cfg_disabled_default_only_ids'] == [disabled[(*scope, name)] for name in cfg_names]
                and report['all_features_native_passes_include_default_only_ids'] is False
                and all(observations[scope].get(name) == 'ok' for name in eligible),
                'An eligible prior PASS regressed/vanished or a default-only ID became a native claim')
        total += len(ids)
        retained += len(eligible)
        excluded += len(cfg_names)
    require(total == expected_total and excluded == expected_disabled and retained + excluded == total,
            'Exact mode-qualified retention total differs')
    return retained, excluded

def metadata_scopes(metadata, all_features):
    selected = set()
    for package in metadata['packages']:
        if package['id'] not in metadata['workspace_members']:
            continue
        enabled = set(package['features']) if all_features else set()
        pending = [] if all_features else ['default']
        while pending:
            feature = pending.pop()
            if feature not in enabled and feature in package['features']:
                enabled.add(feature)
                pending.extend(package['features'][feature])
        for target in package['targets']:
            kind = target['kind'][0]
            if set(target.get('required-features', [])) - enabled:
                continue
            if target.get('test') and kind in ('lib', 'bin', 'test', 'bench', 'example'):
                selected.add((package['name'], target['name'], kind))
        if any(target.get('doctest') and 'lib' in target['kind'] for target in package['targets']):
            selected.add((package['name'], 'doc', 'doc'))
    return selected



def read_current_scopes(audit, raw, binaries, all_features, head=HEAD, partition_sha=PARTITION_DRIVER_SHA):
    def obj(binding):
        return json.loads(raw(binding['path'], binding['sha256']))

    def archived(row, outer=False):
        digest = row['archive_sha256' if outer else 'gzip_sha256']
        with gzip.GzipFile(fileobj=io.BytesIO(raw(row['path'], digest)), mode='rb') as archive:
            value = archive.read(LIMIT + 1)
        require(len(value) <= LIMIT and len(value) == row['raw_bytes' if outer else 'uncompressed_bytes']
                and sha(value) == row['raw_sha256' if outer else 'uncompressed_sha256'], 'Compressed current raw proof differs')
        return value

    runs, complete_metadata, metadata = [], None, None
    for run in audit['closed_actual_runs']:
        owner, closed = obj(run['workspace_owner']), obj(run['outer_owner'])
        require(owner['head'] == closed['head'] == head
                and owner['source_unchanged'] is owner['shared_runtime_linker_unchanged'] is True
                and closed['complete'] is closed['source_unchanged'] is True
                and closed['exit_code'] == run['literal_outer_exit_code']
                and owner['all_features'] is all_features and owner['profile'] == 'release'
                and owner['include_ignored'] is owner['partition_cli'] is True
                and ('--all-features' in closed['command']) is all_features
                and not set(closed['command']).intersection({'--native-filter', '--plan-only', '--features',
                    '--no-default-features', '--packages', '--reuse-library-build-from'}),
                'Actual all-features source/closure/profile/whole-run policy differs')
        parent = str(Path(run['workspace_owner']['path']).parent)
        source = json.loads(raw(parent + '/source-start.json'))
        require(source == {'head': head, 'tracked_patch_sha256': sha(b''), 'untracked': {}}
                and closed['source_before'] == closed['source_after'] == source,
                'Actual clean all-features source map differs')
        current_metadata = json.loads(raw(parent + '/metadata.json'))
        require(current_metadata['workspace_root'] == ROOT[:-1]
                and current_metadata['target_directory'] == '/workspace/porffor/target'
                and (metadata is None or metadata == current_metadata), 'Actual Cargo metadata differs')
        metadata = current_metadata
        complete_metadata = metadata_scopes(metadata, all_features) if complete_metadata is None else complete_metadata
        require(metadata_scopes(metadata, all_features) == complete_metadata and len(complete_metadata) == (108 if all_features else 107),
                'Actual all-features 108 metadata inventory differs')
        for field in ['stdout', 'stderr']:
            row = closed[field]
            archived(row, outer=True) if 'archive_sha256' in row else raw(row['path'], row['sha256'])
        runs.append(owner)
    observations, decoded_groups = {}, {}
    for scope in audit['scopes']:
        key = tuple(scope['scope'])
        require(key not in observations and scope['recorded_state'] == 'passed'
                and scope['raw_coverage_verified'] is True, 'Duplicate/unpassed actual all-features scope')
        names = inventory(raw(scope['raw_inventory']['path'], scope['raw_inventory']['sha256']))
        values, counts = outcomes(raw(scope['raw_transcript']['path'], scope['raw_transcript']['sha256']), names)
        require(scope['inventory'] == names and scope['verdicts'] == values and scope['counts'] == counts
                and scope['exit_code'] == counts['failed'] == counts['ignored'] == 0, 'Actual all-features PASS IDs differ')
        require(isinstance(scope['evidence_run'], int) and 0 <= scope['evidence_run'] < len(runs), 'Invalid scope origin')
        owner = runs[scope['evidence_run']]
        matches = [case for case in owner['cases'] if (case['package'], case['target'], case['kind']) == key]
        require(len(matches) == 1 and matches[0]['state'] == 'passed' and not matches[0].get('native_filter'),
                'Actual all-features scope lacks one whole owner')
        case = matches[0]
        require(scope['raw_inventory'] == {'path': case['inventory_log'], 'sha256': case['inventory_sha256']}
                and scope['raw_transcript'] == {'path': case['transcript_log'], 'sha256': case['transcript_sha256']}
                and case['outcomes'] == [counts], 'Actual scope raw owner differs')
        if key[2] != 'doc':
            group = owner['build_groups'][case.get('build_group', case.get('reused_build_group'))]
            command = group['command']
            require({'cargo', 'test', '--locked', '--offline', '--release', '--no-run'} <= set(command)
                    and ('--all-features' in command) is all_features
                    and not set(command).intersection({'--features', '--no-default-features'}),
                    'Actual all-features Cargo flags/profile differ')
            rows = [row for row in group['sealed_build_logs'] if row['path'].endswith('.build.jsonl.gz')]
            require(len(rows) == 1, 'Actual Cargo stream missing/ambiguous')
            row = rows[0]
            if row['gzip_sha256'] not in decoded_groups:
                decoded_groups[row['gzip_sha256']] = [json.loads(line) for line in archived(row).splitlines()]
            events = decoded_groups[row['gzip_sha256']]
            require([event['success'] for event in events if event.get('reason') == 'build-finished'] == [True],
                    'Actual Cargo group lacks successful closure')
            artifact = scope['actual_Cargo_artifact']
            package = next(package for package in metadata['packages'] if package['name'] == key[0])
            target = next(target for target in package['targets']
                          if target['name'] == key[1] and target['kind'] == [key[2]])
            executable = (case['command'][case['command'].index('--executable') + 1]
                          if 'native_partitions' in case else case['command'][0])
            require(artifact in events and artifact['profile']['test'] and artifact['profile']['opt_level'] == '3'
                    and artifact['package_id'] == package['id'] and artifact['target']['name'] == key[1]
                    and artifact['target']['kind'] == [key[2]] and artifact['target']['src_path'] == target['src_path']
                    and artifact['executable'] == executable and Path(executable).parent == Path('/workspace/porffor/target/release/deps')
                    and scope['recorded_executable_sha256'] == case['executable_sha256']
                    and re.fullmatch('[a-f0-9]{64}', case['executable_sha256'])
                    and (not all_features or set(package['features']) == set(artifact['features'])),
                    'Actual selected all-features test artifact/features differ from Cargo raw bytes')
            for producer in group.get('build_script_artifacts', []):
                require(producer['path'] in binaries and binaries[producer['path']]['sha256'] == producer['sha256']
                        and any(event.get('reason') == 'compiler-artifact' and event['package_id'] == producer['package_id']
                                and event['target']['kind'] == ['custom-build'] and producer['path'] in event['filenames'] for event in events),
                        'Actual all-features Cargo producer/hash provenance differs')
            if 'shared_link_event' in case:
                image, event = case['shared_runtime_image'], case['shared_link_event']
                require(event['shared_image'] in binaries
                        and binaries[event['shared_image']]['sha256'] == image['shared_sha256']
                        and image['identity'] == event['identity']
                        and event['executable_sha256'] == case['executable_sha256'],
                        'Actual shared runtime image binding differs')
                environments = [dict(event['env']) for event in events if event.get('reason') == 'build-script-executed'
                                and '/crates/lila-engine#' in event.get('package_id', '')]
                require(environments and all(env['LILA_COMPILER_SOURCE_REVISION'] == 'git:' + head for env in environments),
                        'Actual Engine source environment differs')
        else:
            require(('--all-features' in case['command']) is all_features, 'Actual documentation feature mode differs')
            group = owner['build_groups'][case['build_group']]
            for command in (group['inventory_command'], group['command']):
                require({'cargo', 'test', '--locked', '--offline', '--release', '--doc'} <= set(command)
                        and ('--all-features' in command) is all_features
                        and not set(command).intersection({'--features', '--no-default-features', '--target'}),
                        'Actual documentation Cargo source/profile policy differs')
            require(case['cohort_exit_code'] == scope['doc_cohort_exit_code'] == 0,
                    'Selected documentation cohort failed')
            def doc_chunks(data):
                text = data.decode()
                starts = list(re.finditer(r'^\s*Doc-tests (\S+)\s*$', text, re.M))
                chunks = {}
                for index, start in enumerate(starts):
                    require(start[1] not in chunks, 'Duplicate documentation package')
                    end = starts[index + 1].start() if index + 1 < len(starts) else len(text)
                    chunks[start[1]] = text[start.start():end].encode()
                return chunks
            listed = doc_chunks(raw(group['inventory_log'], group['inventory_sha256']))
            executed = doc_chunks(raw(group['transcript_log'], group['transcript_sha256']))
            require(set(listed) == set(executed) and listed, 'Complete documentation cohort differs')
            for package in listed:
                _, repeated_counts = outcomes(executed[package], inventory(listed[package]))
                require(repeated_counts['failed'] == repeated_counts['ignored'] == 0,
                        'Actual documentation cohort/repeated sibling did not pass')
        if 'native_partitions' in case:
            binding = case['native_partitions']
            partition = obj(binding)
            require(partition == binding['evidence'] and partition['qualified_target'] == key[0] + ':' + key[1]
                    and partition['executable_sha256'] == case['executable_sha256']
                    and partition['driver_sha256'] == partition_sha
                    and partition['cpus'] == case['test_cpus'] and partition['complete_test_count'] == len(names)
                    and partition['stall_seconds'] == (3600 if key[0] == 'lila-cli' else 900),
                    'Actual partition target/driver/artifact/deadline differs')
            require(inventory(raw(partition['original_inventory'], partition['original_inventory_sha256'])) == names
                    and inventory(raw(partition['complete_inventory'], partition['complete_inventory_sha256'])) == names
                    and len(partition['partitions']) == len(partition['cpus'])
                    and partition['inherited_retained_modules']['entries_per_process'] == '2'
                    and partition['inherited_retained_modules']['image_bytes_per_process'] == '268435456',
                    'Actual repeated inventory/partition/cache policy differs')
            assigned = {}
            for index, part in enumerate(partition['partitions']):
                listed = inventory(raw(part['inventory_log'], part['inventory_sha256']))
                actual, part_counts = outcomes(raw(part['transcript_log'], part['transcript_sha256']), listed)
                require(listed == part['ids'] and actual == part['native_outcomes'] and part_counts == part['counts']
                        and part['command'] == [str(Path(partition['executable'])), '--exact', *listed, '--test-threads=1', '--include-ignored']
                        and not set(actual).intersection(assigned) and part['state'] == 'passed' and part['exit_code'] == 0,
                        'Actual disjoint exact partition argv/IDs/outcomes differ')
                output_owner = Path(binding['path']).parent
                prefix = re.sub(r'[^A-Za-z0-9_.-]', '-', output_owner.name)[:64] or 'native'
                label = (prefix + '-' + sha(os.fsencode(output_owner)) + '-partition-' + str(index)
                         if partition_sha == PARTITION_DRIVER_SHA else output_owner.name + '-partition-' + str(index))
                require(part['index'] == index and part['cpu'] == partition['cpus'][index]
                        and part['launch_command'] == ['taskset', '-c', str(part['cpu']), './scripts/run-watched.sh',
                            '--label', label, '--stall', str(partition['stall_seconds']), '--poll', '1', '--',
                            'python3', '-B', 'scripts/limited_verification.py', '--cloud', '--', *part['command']],
                        'Actual partition CPU/stock cloud/deadline launch differs')
                require(part['transcript_log'] == str(Path('target/watched') / (label + '.log')), 'Actual partition transcript owner differs')
                assigned.update(actual)
            require(assigned == values, 'Complete partition union differs from actual scope')
        elif key[2] != 'doc':
            require(case['command'] == [artifact['executable'], '--test-threads=1', '--include-ignored'],
                    'Whole all-features native command contains filters/changed policy')
        observations[key] = values
    require(set(observations) == complete_metadata and len(observations) == (108 if all_features else 107)
            and dict(Counter(value for values in observations.values() for value in values.values())) == audit['actual_native_counts'],
            'Actual complete all-features metadata/native union differs')
    return observations, metadata, runs


def accepted_audit(audit, all_features):
    require(audit['schema'] == (AF_SCHEMA if all_features else DEFAULT_SCHEMA)
            and audit['compiler_revision'] == HEAD
            and audit['feature_mode'] == ('all_features' if all_features else 'default')
            and audit['all_features_requested'] is all_features, 'Wrong actual source/mode/schema')
    required = ['closed_process', 'all_raw_scope_coverage_verified', 'all_70_required_controls_passed',
                'all_24_new_controls_passed', 'all_94_required_controls_passed',
                'all_rechecked_original_failed_ids_passed', 'all_whole_scope_rechecks_passed']
    required += (['whole_all_features_workspace_passed', 'full_baseline_accounted_for',
                  'all_eligible_9127_d2_passes_retained', 'full_d2_baseline_accounted_for',
                  'all_9220_prior_all_features_passes_retained'] if all_features else
                 ['whole_default_workspace_passed', 'whole_default_workspace_logical_union_passed',
                  'all_old_passes_retained', 'all_9120_current_passes_retained', 'all_9127_d2_passes_retained'])
    for field in required:
        require(audit[field] is True, 'Actual audit not accepted: ' + field)
    count = 108 if all_features else 107
    require(audit['metadata_target_count'] == len(audit['scopes']) == count
            and audit['recorded_state_counts'] == {'passed': count}
            and audit['required_control_count'] == 94 and audit['prior_d2_passing_count'] == 9127
            and audit['retained_d2_passes'] == (9123 if all_features else 9127)
            and set(audit['actual_native_counts']) == {'ok'}, 'Actual scope/retention/control policy differs')
    if all_features:
        require(audit['prior_all_features_passing_count'] == audit['retained_prior_all_features_passes'] == 9220
                and audit['cfg_disabled_9127_baseline_count'] == 4
                and audit['eligible_d2_baseline_count'] == 9123
                and audit['default_only_ids_counted_as_all_features_native_passes'] is False,
                'Actual prior AF/default-only retention differs')
    require(not any(audit[field] for field in ['ordinary_CLI_acceptance', 'historical5365_acceptance',
                'full_pinned_suite_or_task_closure', 'synthetic_16_scope_adapter']), 'Workspace acceptance widened')


def load_reader(source, expected):
    require(sha(source) == expected, 'Inherited standalone reader SHA differs')
    namespace = {'__name__': 'sealed_inherited_portable_reader'}
    exec(compile(source, '<SHA-pinned inherited portable reader>', 'exec'), namespace)
    return namespace


def raw_layers(document):
    """One copy of each inherited pack; resolve only exact path/SHA identities."""
    stores, audits, seen = [], {}, set()

    def add(pack):
        store = RawStore(pack)
        stores.append(store)
        ref = pack.get('actual_audit')
        if ref is None and pack.get('schema') == 'portable-real-default-workspace-v8b-20261014':
            ref = {'path': '/workspace/porffor/target/workspace-full-default-d2f4bc51-v8b-audit-20261014.json',
                   'sha256': pack['exact_audit']['sha256']}
        if ref:
            value = unpack(pack['exact_audit'])
            require(sha(value) == ref['sha256'], 'Inherited exact audit authority differs')
            audits[(ref['path'], ref['sha256'])] = value
        refs = pack.get('inherited_portables', [])
        if pack.get('schema') == 'portable-real-all-features-workspace-v9-20261014':
            refs = [pack['same_source_default_portable']]
        for parent in refs:
            if parent['sha256'] in seen:
                continue
            seen.add(parent['sha256'])
            data = store.get(parent['path'], parent['sha256'])
            require(len(data) == parent['bytes'] and sha(data) == parent['sha256'], 'Inherited portable bytes differ')
            add(json.loads(data))
        return store

    primary = add(document)

    def raw(path, digest=None):
        path = canonical(str(path))
        if digest is None:
            keys = {key for store in stores for key in store.index if key[0] == path}
            keys.update(key for key in audits if key[0] == path)
            require(len(keys) == 1, 'Missing/ambiguous inherited path without SHA: ' + path)
            digest = next(iter(keys))[1]
        key = (path, digest)
        if key in audits:
            return audits[key]
        for store in stores:
            if key in store.index:
                return store.get(*key)
        raise ValueError('No exact portable path/SHA authority: ' + str(key))

    return primary, stores, raw


def scope_maps(audit, raw):
    scopes = {}
    for row in audit['scopes']:
        key = tuple(row['scope'])
        require(key not in scopes and row['raw_coverage_verified'] is True,
                'Duplicate/incomplete historical scope')
        names = inventory(raw(row['raw_inventory']['path'], row['raw_inventory']['sha256']))
        values, counts = outcomes(raw(row['raw_transcript']['path'], row['raw_transcript']['sha256']), names)
        require(names == row['inventory'] and values == row['verdicts'] and counts == row['counts']
                and counts['failed'] == counts['ignored'] == 0, 'Historical raw scope differs')
        scopes[key] = {'verdicts': values}
    return scopes


def require_passes(prior, observations, expected_count, disabled=()):
    count = excluded = 0
    for scope, row in prior.items():
        for name, value in row['verdicts'].items():
            require(value == 'ok', 'Prior required ID was not actually PASS')
            count += 1
            if (*scope, name) in disabled:
                excluded += 1
            else:
                require(observations.get(scope, {}).get(name) == 'ok',
                        'Prior qualified PASS regressed or vanished: ' + str((*scope, name)))
    require(count == expected_count, 'Prior raw passing count differs')
    return count - excluded, excluded


def verify_controls(audit, observations, d2_default, raw):
    old = d2_default['v2_required_new_controls'] + d2_default['additional_28_required_controls'] + d2_default['seven_subclass_required_controls']
    manifest = json.loads(raw('/workspace/porffor/target/combined-shadow-runtime-iteration-private-d2f4bc51-20261014/manifest-v2.json', NEW_MANIFEST_SHA))
    current = audit['v2_required_new_controls'] + audit['additional_28_required_controls'] + audit['seven_subclass_required_controls']
    new = audit['twenty_four_next_required_controls']
    expected_old = {(*row['scope'], row['id']) for row in old}
    expected_new = set()
    for row in manifest['new_native_controls']:
        matches = [tuple(scope['scope']) for scope in d2_default['scopes']
                   if scope['scope'][:2] == [row['package'], row['target']] and scope['scope'][2] != 'doc']
        require(len(matches) == 1, 'New required control has no unique Cargo scope')
        expected_new.add((*matches[0], row['id']))
    require(len(current) == len(expected_old) == 70 and len(new) == len(expected_new) == 24
            and not expected_old.intersection(expected_new)
            and {(*row['scope'], row['id']) for row in current} == expected_old
            and {(*row['scope'], row['id']) for row in new} == expected_new,
            'Required exact 70+24 control registration differs')
    require(all(observations.get(tuple(row['scope']), {}).get(row['id']) == row['actual_outcome'] == 'ok'
                for row in current + new), 'A required control lacks actual raw PASS evidence')
    return len(current) + len(new)


def verify_additional_controls(audit, observations, d2_default, raw):
    """Keep the sealed94 gate and add exact generator IDs using current raw PASS."""
    manifest = json.loads(raw(EXTRA_MANIFEST, EXTRA_MANIFEST_SHA))
    require(manifest['schema'] == 'prepared-function-global-constructor-hints-private-v2',
            'Additional control source registration differs')
    rows = manifest['new_native_controls']
    ids = { (row['package'], row['target'], row['kind'], row['id']) for row in rows }
    base = audit['v2_required_new_controls'] + audit['additional_28_required_controls'] + audit['seven_subclass_required_controls'] + audit['twenty_four_next_required_controls']
    base_ids = {(*row['scope'], row['id']) for row in base}
    historical = {(*row['scope'], name) for row in d2_default['scopes'] for name in row['verdicts']}
    require(len(rows) == len(ids) == 10 and len(base_ids) == 94
            and not ids.intersection(base_ids) and not ids.intersection(historical),
            'Additional10/base94 exact disjoint control registration differs')
    controls = []
    for row in rows:
        scope = (row['package'], row['target'], row['kind'])
        actual = observations.get(scope, {}).get(row['id'])
        require(actual == 'ok', 'Additional required control lacks actual raw PASS: ' + str((*scope, row['id'])))
        controls.append({'scope': list(scope), 'id': row['id'], 'actual_outcome': actual})
    return controls


def verify_source_snapshot(snapshot, raw):
    require(snapshot['head'] == HEAD and len(snapshot['files']) == 30,
            'Current exact source snapshot differs')
    actual = {row['relative_path']: row['sha256'] for row in snapshot['files']}
    require(len(actual) == len(snapshot['files']) and all(
        row['path'] == ROOT + row['relative_path']
        and len(raw(row['path'], row['sha256'])) == row['bytes'] for row in snapshot['files']),
        'Current source path/bytes/unique identities differ')
    layers = [('master_manifest', MASTER_MANIFEST, MASTER_MANIFEST_SHA),
              ('additional_source_manifest', EXTRA_MANIFEST, EXTRA_MANIFEST_SHA),
              ('corrected_source_manifest', CORRECTION_MANIFEST, CORRECTION_MANIFEST_SHA)]
    require(snapshot['source_manifest_order'] == [role for role, _, _ in layers],
            'Source manifest application order differs')
    prepared, expected = [], {}
    for role, path, digest in layers:
        ref = snapshot[role]
        require(ref['path'] == path and ref['sha256'] == digest,
                'Exact source manifest authority differs')
        data = raw(path, digest)
        require(sha(data) == digest and len(data) == ref['bytes'],
                'Exact source manifest content differs')
        source = json.loads(data)
        require(len({row['path'] for row in source['files']}) == len(source['files']),
                'Duplicate source manifest file')
        expected.update({row['path']: row['after_sha256'] for row in source['files']})
        prepared.append(source)
    generator, correction = prepared[1:]
    key = lambda row: (row['package'], row['target'], row['kind'], row['id'])
    registered = {key(row) for row in generator['new_native_controls']}
    unchanged = {key(row) for row in correction['controls_to_validate']}
    require(correction['schema'] == 'prepared-generator-reader-source-hypotheses-v1'
            and correction['control_ids_unchanged'] is True and correction['added_control_ids'] == []
            and len(correction['controls_to_validate']) == len(unchanged) == len(registered) == 10
            and registered == unchanged and correction['required_control_total'] == 10
            and len(correction['modified_control_ids']) == len(set(correction['modified_control_ids'])) == 4
            and set(correction['modified_control_ids']) <= {row[-1] for row in registered},
            'Correction must retain the exact ten registered control IDs')
    adjustments = snapshot['source_adjustments']
    require(len(adjustments) == 1
            and adjustments[0]['relative_path'] == 'crates/lila-ir/tests/finite_indirect_eval.rs'
            and adjustments[0]['manifest_after_sha256'] == expected[adjustments[0]['relative_path']]
            and adjustments[0]['actual_source_sha256'] == 'bef8b3d41b02bc178f4331e5cf55d1eb939ce6a35fc335217eaee4c8299cdaa6',
            'Only the exact corrected NEW IR boundary assertion may adjust the layered source')
    expected[adjustments[0]['relative_path']] = adjustments[0]['actual_source_sha256']
    require(set(actual) == set(expected) | {'CONTINUE.md'}
            and all(actual[path] == digest for path, digest in expected.items())
            and snapshot['intentional_existing_structure_body_changes'] == prepared[0]['existing_structural_test_body_updates']
            and snapshot['intentional_existing_structure_body_changes']['count'] == 3
            and snapshot['status_files'] == ['README.md', 'CONTINUE.md'],
            'Exact applied source/strengthened existing bodies/status bindings differ')


def verify_closed_union(audit, runs, raw, all_features):
    keys = lambda owner: {(c['package'], c['target'], c['kind']): c for c in owner['cases']}
    original = keys(runs[0])
    require(len(original) == len(runs[0]['cases']) == (108 if all_features else 107), 'Original full metadata coverage differs')
    current, origins = dict(original), {key: 0 for key in original}
    require(audit['original_recorded_state_counts'] == dict(Counter(c.get('state', 'unrecorded/unstarted') for c in runs[0]['cases'])),
            'Original literal state counts were rewritten')
    for row, owner in zip(audit['closed_actual_runs'], runs):
        require(row['literal_scope_states'] == [{'scope': [c['package'], c['target'], c['kind']], 'state': c.get('state')}
                for c in owner['cases']], 'Literal actual run scope states differ')
    original_closed = audit['closed_actual_runs'][0]
    require(audit['outer_exit_code'] == original_closed['literal_outer_exit_code']
            and audit['single_original_process_passed'] == (audit['outer_exit_code'] == 0
                and all(c.get('state') == 'passed' for c in runs[0]['cases'])),
            'Original process state/exit was rewritten')
    schedule = load_reader(raw(ROOT + 'scripts/workspace_test_schedule.py', SCHEDULE_SHA), SCHEDULE_SHA)
    identity = ('head', 'profile', 'target_directory', 'all_features', 'include_ignored', 'source_map_sha256',
        'verification_driver_sha256', 'selected_workspace_packages', 'isolated_test_packages',
        'targets_disabled_by_cargo_features', 'shared_runtime_linker', 'test_incremental_policy',
        'scheduling', 'artifact_selection_policy', 'partition_cli')
    def command_policy(command):
        result, position = [], 0
        while position < len(command):
            value = command[position]
            if value in ('--output', '--retry-from', '--continuation-phase'):
                require(position + 1 < len(command), 'Missing continuation argument value')
                position += 2
            elif value == '--retry-interrupted':
                position += 1
            else:
                result.append(value)
                position += 1
        return result
    first_closed = json.loads(raw(original_closed['outer_owner']['path'], original_closed['outer_owner']['sha256']))
    for index, owner in enumerate(runs[1:], 1):
        admission = audit['closed_actual_runs'][index]
        cases = keys(owner)
        require(len(cases) == len(owner['cases']) and set(cases) <= set(original)
                and all(owner.get(field) == runs[0].get(field) for field in identity),
                'Continuation/recheck source/profile/graph policy differs')
        if admission['purpose'] == 'continuation':
            retry = owner['retry_input']
            parent_binding = {'path': retry['path'], 'sha256': retry['sha256']}
            parents = [i for i, row in enumerate(audit['closed_actual_runs'][:index])
                       if row['workspace_owner'] == parent_binding]
            require(len(parents) == 1, 'Continuation lacks one exact earlier retry parent')
            closed = json.loads(raw(admission['outer_owner']['path'], admission['outer_owner']['sha256']))
            command = closed['command']
            require(command[command.index('--retry-from') + 1] == retry['path']
                    and command[command.index('--continuation-phase') + 1] == owner['continuation_phase']
                    and command_policy(command) == command_policy(first_closed['command'])
                    and owner.get('partition_native_targets') == runs[0].get('partition_native_targets'),
                    'Actual continuation parent/phase argv differs')
            interrupted = '--retry-interrupted' in command
            eligible = {key: c for key, c in keys(runs[parents[0]]).items()
                        if c.get('state') is None or (interrupted and c.get('state') in ('interrupted', 'storage-deferred'))}
            selected = {key for key, c in eligible.items()
                        if schedule['is_barrier'](c) == (owner['continuation_phase'] == 'barriers')}
            require(set(cases) == selected and owner['continuation_phase'] in ('parallel', 'barriers'),
                    'Continuation is not the exact eligible phase inventory')
            require(all(current[key].get('state') not in ('passed', 'failed') for key in cases),
                    'Continuation overlaps a completed whole scope')
        else:
            require(admission['purpose'] == 'recheck' and all(original[key].get('state') == 'failed'
                    and key[2] != 'doc' and origins[key] == 0 for key in cases),
                    'Recheck is not a disjoint originally failed whole-native scope')
            for key, case in cases.items():
                old = original[key]
                names = inventory(raw(old['inventory_log'], old['inventory_sha256']))
                old_values, counts = outcomes(raw(old['transcript_log'], old['transcript_sha256']), names)
                require(counts['failed'] > 0 and old['exit_code'] != 0
                        and names == inventory(raw(case['inventory_log'], case['inventory_sha256']))
                        and old.get('shared_runtime_image') == case.get('shared_runtime_image'),
                        'Recheck lacks exact original failed raw inventory/runtime identity')
        for key, case in cases.items():
            require(case['selector'] == original[key]['selector'] and not case.get('native_filter'), 'Whole-scope selector changed')
            current[key], origins[key] = case, index
    require(audit['continuation_count'] == sum(row['purpose'] == 'continuation' for row in audit['closed_actual_runs'])
            and audit['recheck_count'] == sum(row['purpose'] == 'recheck' for row in audit['closed_actual_runs']),
            'Literal continuation/recheck count differs')
    for row in audit['scopes']:
        scope = tuple(row['scope'])
        require(row['evidence_run'] == origins[scope]
                and row['evidence_owner'] == audit['closed_actual_runs'][origins[scope]]['workspace_owner'],
                'Selected final scope is not the literal admitted whole owner')


def verify(document, expected_audit_sha=None):
    require(document['schema'] == PACK_SCHEMA, 'Unknown next portable format')
    all_features = document['feature_mode'] == 'all_features'
    require(document['feature_mode'] in ('default', 'all_features'), 'Unknown feature mode')
    audit_bytes = unpack(document['exact_audit'])
    require(sha(audit_bytes) == document['actual_audit']['sha256']
            and len(audit_bytes) == document['actual_audit']['bytes']
            and (expected_audit_sha is None or sha(audit_bytes) == expected_audit_sha), 'Actual audit external SHA differs')
    audit = json.loads(audit_bytes)
    accepted_audit(audit, all_features)
    store, stores, raw = raw_layers(document)
    parents = {row['role']: row for row in document['inherited_portables']}
    require(len(parents) == len(document['inherited_portables'])
            and set(parents) == ({'current_default', 'd2_all_features'} if all_features else {'d2_default'}),
            'Inherited portable roles differ')
    if all_features:
        current_default = json.loads(raw(parents['current_default']['path'], parents['current_default']['sha256']))
        default_result = verify(current_default, audit['same_source_default_audit']['sha256'])
        require(default_result['feature_mode'] == 'default' and default_result['compiler_revision'] == HEAD,
                'Actual same-final-source default portable differs')
        default_audit = json.loads(unpack(current_default['exact_audit']))
        require(audit['same_source_default_audit'] == current_default['actual_audit']
                or audit['same_source_default_audit'] == {key: current_default['actual_audit'][key] for key in ('path', 'sha256')},
                'Actual same-source default audit binding differs')
        require(parents['d2_all_features']['sha256'] == D2_AF_SHA, 'Prior all-features authority changed')
        old_af = json.loads(raw(parents['d2_all_features']['path'], D2_AF_SHA))
        ref = old_af['portable_verifier_source']
        load_reader(raw(ref['path'], D2_AF_READER_SHA), D2_AF_READER_SHA)['verify'](old_af)
        d2_ref = old_af['same_source_default_portable']
    else:
        d2_ref = parents['d2_default']
        require(d2_ref['sha256'] == D2_DEFAULT_SHA, 'D2 default authority changed')
    d2_default = json.loads(raw(d2_ref['path'], D2_DEFAULT_SHA))
    ref = d2_default['portable_verifier_source']
    load_reader(raw(ref['path'], D2_DEFAULT_READER_SHA), D2_DEFAULT_READER_SHA)['verify'](d2_default)
    d2_audit = json.loads(unpack(d2_default['exact_audit']))
    binaries = {row['observed_path']: row for row in document['observed_binary_bindings']}
    require(len(binaries) == len(document['observed_binary_bindings']), 'Duplicate binary closure binding')
    for path, binding in audit['bound_files'].items():
        key = (path, binding['sha256'])
        try:
            value = raw(*key)
        except ValueError as error:
            require(str(error).startswith('No exact portable path/SHA authority:'), str(error))
            require(path in binaries and all(binaries[path][field] == binding[field] for field in ('sha256', 'bytes'))
                    and binaries[path]['payload_embedded'] is False, 'Closure-time binary identity differs')
        else:
            require(len(value) == binding['bytes'], 'Audit raw binding differs')
    observations, metadata, runs = read_current_scopes(audit, raw, binaries, all_features)
    verify_closed_union(audit, runs, raw, all_features)
    fences = audit['read_only_verifier_source_index_fences']
    require(fences['before'] == fences['after'] and all(row['patch_sha256'] == sha(b'') and row['untracked'] == ''
                for row in fences['before'].values()) and fences['before'][ROOT[:-1]]['head'] == HEAD,
            'Actual clean source/index closure fences differ')
    require(document['current_source_snapshot']['sha256'] == SNAPSHOT_SHA, 'Exact final source snapshot SHA differs')
    snapshot = json.loads(raw(document['current_source_snapshot']['path'], SNAPSHOT_SHA))
    verify_source_snapshot(snapshot, raw)
    if all_features:
        registry = json.loads(raw(audit['source_authoritative_default_only_cfg_registry']['path'], audit['source_authoritative_default_only_cfg_registry']['sha256']))
        old_audit = json.loads(unpack(old_af['exact_audit']))
        require({(*row['scope'], row['id'], row['source_path']) for row in registry['rows']}
                == {(*row['scope'], row['id'], row['source_path']) for row in old_audit['source_proven_default_only_ids']},
                'Only the four exact prior default-only source IDs may qualify')
        disabled = qualified_default_ids(audit, default_audit, registry, observations, metadata, raw)
        require(len(disabled) == 4, 'Exact default-only qualification count differs')
    else:
        disabled = {}
    retained, excluded = require_passes(scope_maps(d2_audit, raw), observations, 9127, disabled)
    require((retained, excluded) == ((9123, 4) if all_features else (9127, 0)), 'D2 mode-qualified retention differs')
    retained_af = None
    if all_features:
        retained_af, excluded_af = require_passes(scope_maps(old_audit, raw), observations, 9220)
        require((retained_af, excluded_af) == (9220, 0), 'Prior9220 AF retention differs')
    base_controls = verify_controls(audit, observations, d2_audit, raw)
    extra_controls = verify_additional_controls(audit, observations, d2_audit, raw)
    required_controls = base_controls + len(extra_controls)
    require(document['compiler_revision'] == HEAD and document['metadata_scopes'] == len(observations)
            and document['native_counts'] == audit['actual_native_counts']
            and document['required_base_controls'] == base_controls == 94
            and document['required_additional_controls'] == len(extra_controls) == 10
            and document['additional_controls_registration'] == {'path': EXTRA_MANIFEST, 'sha256': EXTRA_MANIFEST_SHA}
            and document['required_controls'] == required_controls == 104
            and document['whole_workspace_passed'] is True
            and document['original_process_exit'] == audit['outer_exit_code']
            and document['original_process_passed'] == audit['single_original_process_passed'],
            'Portable summary changed actual acceptance')
    ref = document['portable_verifier_source']
    require(len(raw(ref['path'], ref['sha256'])) == ref['bytes'], 'Standalone verifier source bytes differ')
    for layer in stores:
        layer.verify_all()
    return {'verified': True, 'compiler_revision': HEAD, 'feature_mode': document['feature_mode'],
        'metadata_scopes': len(observations), 'native_counts': audit['actual_native_counts'],
        'required_controls': required_controls, 'retained_d2_default_passes': retained,
        'base_required_controls': base_controls, 'additional_required_controls': extra_controls,
        'additional_required_control_count': len(extra_controls),
        'source_proven_default_only_ids': excluded,
        'retained_prior_all_features_passes': retained_af,
        'current_source_files': len(snapshot['files']), 'binary_paths_reopened_or_presence_claimed': False,
        'ordinary_CLI_acceptance': False, 'historical5365_acceptance': False, 'full_pinned_suite_or_task_closure': False}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('checkpoint', type=Path)
    parser.add_argument('--audit-sha256')
    args = parser.parse_args()
    require(args.checkpoint.stat().st_size <= 4 * LIMIT, 'Portable file exceeds read bound')
    print(json.dumps(verify(json.loads(args.checkpoint.read_bytes()), args.audit_sha256), indent=2))
