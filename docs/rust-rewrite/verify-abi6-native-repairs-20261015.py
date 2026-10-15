#!/usr/bin/env python3
"""Verify the portable ABI6 source and focused native checkpoint using only Python's standard library."""
import argparse, base64, hashlib, json, pathlib, re, zlib

BASE = '1fc2599d6dd7b29f4a6b0dcafd1ec0106cfc7b9e'
CANDIDATE_SHA = '90576212a0c3a5f25e2e49e678b8214b6d3e0a665f78d375966f1ebfb3da8f0e'
HANDOFF_SHA = 'dbe53bb1fb1e4221d243f63031509bf8831d4236f02f30aa271473aff5615360'
FINGERPRINT = '30214a1c7dd65c2483eb6dbea22f8626cbae4413a0f8cc782979fedba7483e28'
NATIVE_ANCHORS = {
    ('optimized_core', 'original_RED'): '4eb25badea94258cd829be786fa5933136b4475f1710665f79b1a9b671329940',
    ('optimized_core', 'continuation_PASS'): '7569f1f69b2069f4b39f27ccfbccfff49d43952d705ca6b36bcb734d851f98e9',
    ('real_Test262_24', 'owner'): 'aecc28b567e2f806aab293b288153abc2d7190390f97adb43586002521a9443b',
    ('engine_controls', 'original_RED'): '32ec765620f2f3fec566c4a2b545fc773edc8ef35757c331bfcbb71dec02fd3c',
    ('engine_controls', 'continuation_PASS'): 'ee74efba10f39fa2790694dd75f42e17b449110e650ad8261ab5836dbee29763',
    ('cli_controls', 'earlier_RED'): 'f1edbd7dc70084973fc0810af519bdecf8ca000c770546f7f80422c185ba753b',
    ('cli_controls', 'later_storage_RED'): '3d6872c93a10e75d827150aecc7af959c4d12854e55dbc9cf8c7c2dfb71238a8',
}


def require(value, message):
    if not value: raise ValueError(message)


def digest(raw): return hashlib.sha256(raw).hexdigest()


class Payloads:
    def __init__(self, proof):
        self.rows = proof['payloads']; self.cache = {}
        require(sum(x['bytes'] for x in self.rows.values()) <= 240 * 1024**2, 'Raw checkpoint exceeds finite total bound')

    def raw(self, row):
        key = row['sha256']; item = self.rows[key]
        require(item['bytes'] == row['bytes'] and 0 <= item['bytes'] <= 24 * 1024**2,
                'Payload size binding or finite bound differs')
        if key not in self.cache:
            require(item['encoding'] == 'gzip+base64', 'Unknown payload encoding')
            decoder = zlib.decompressobj(16 + zlib.MAX_WBITS)
            raw = decoder.decompress(base64.b64decode(item['data'], validate=True), item['bytes'] + 1)
            require(len(raw) == item['bytes'] and decoder.eof and not decoder.unused_data and
                    digest(raw) == key, 'Payload byte count, framing or SHA256 differs')
            self.cache[key] = raw
        return self.cache[key]

    def json(self, row): return json.loads(self.raw(row))


def source_clean(owner, passed=None):
    require(owner['complete'] and owner['source_unchanged'] and
            owner['source_before'] == owner['source_after'] and
            owner['protected_before'] == owner['protected_after'], 'Owner is incomplete or changed source/protected inputs')
    if passed is not None:
        require(owner['passed'] is passed and owner['exit_code'] == (0 if passed else 1), 'Literal owner outcome differs')


def successful_tests(payloads, owner, rows):
    identities = [(x['target'], x['id']) for x in rows]
    require(rows and len(identities) == len(set(identities)), 'Native test IDs empty or repeated')
    groups = {}
    for row in rows: groups.setdefault(row['stdout']['sha256'], []).append(row)
    for group in groups.values():
        text = payloads.raw(group[0]['stdout']).decode()
        expected = [x['id'] for x in group]
        actual = re.findall(r'^test (.+) \.\.\. ok$', text, re.M)
        require(actual == expected and 'running ' + str(len(expected)) + ' tests' in text and
                re.search(r'test result: ok\. ' + str(len(expected)) + r' passed; 0 failed; 0 ignored;', text),
                'Exact native test IDs or zero-failure/ignore summary differs')
        commands = [x for x in owner['commands'] if x.get('stdout') == group[0]['stdout']]
        require(len(commands) == 1 and commands[0]['complete'] and commands[0]['exit_code'] == 0,
                'Successful transcript must belong to its actual zero-exit command')
        for row in group:
            child = payloads.json(row['child_preflight'])
            require(child['exec_argv'] == commands[0]['exec_argv'] and child['affinity'] == [3] and
                    child['cwd'] == child['tmpdir'] == commands[0]['cwd'] and
                    child['source_fences'] == owner['source_before'], 'Native child command, cwd or source fence differs')
    return len(identities)


def verify(proof, root=None):
    require(proof['schema'] == 'abi6-source-and-focused-native-publication-v1' and proof['parent'] == BASE,
            'Foreign source checkpoint or publication parent')
    p = Payloads(proof); provenance = proof['source']
    require(all(proof['native'][scope][key]['sha256'] == sha for (scope,key),sha in NATIVE_ANCHORS.items()),
            'Actual observed native-owner anchors differ')
    require(provenance['candidate_manifest']['sha256'] == CANDIDATE_SHA and
            provenance['handoff']['sha256'] == HANDOFF_SHA, 'Original reviewed source anchors differ')
    manifest = p.json(provenance['candidate_manifest']); selection = p.json(manifest['selection'])
    handoff = p.json(provenance['handoff']); identity = p.json(manifest['identity_scope'])
    require(manifest['selected_file_count'] == 230 and selection['base'] == manifest['base'] == BASE and
            not selection['ABI7_ABI8_components_selected'] and handoff['candidate_manifest'] == provenance['candidate_manifest'],
            'Reviewed 230-file ABI6 source selection differs')
    require(identity['actual_diagnostic_fingerprint'] == FINGERPRINT and identity['captured_inputs'] == 2617 and
            identity['matching_captured_bytes'] == 2611 and len(identity['preserved_published_differences']) == 6 and
            not identity['ordinary_new_identity_claimed'] and not identity['whole_current_main_native_acceptance'],
            'Diagnostic versus later published parser/policy identity scope differs')
    by_path = {x['relative_path']: x for x in provenance['selected_files']}
    require(len(by_path) == 230 and set(by_path).isdisjoint(selection['published_paths_preserved']),
            'Published parser/policy/proof paths may not be overwritten')
    for item in selection['selected']:
        name = item['relative_path']; row = by_path[name]
        require(not pathlib.PurePosixPath(name).is_absolute() and '..' not in pathlib.PurePosixPath(name).parts,
                'Unsafe selected path')
        require(row['bytes'] == item['candidate']['bytes'] and row['sha256'] == item['candidate']['sha256'] and
                row['base_blob_oid'] == item['base_blob_oid'] and row['mode'] == item['mode'], 'Selected source binding differs')
        raw = p.raw(row)
        require(hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() == item['candidate_blob_oid'],
                'Selected Git blob identity differs')
        if root is not None: require((root / name).read_bytes() == raw, 'Published selected source bytes differ: ' + name)
    for difference in identity['preserved_published_differences']:
        row = provenance['preserved_six'][difference['relative_path']]
        require(row['sha256'] == difference['preserved_main_sha256'] and difference['already_published'],
                'Later published parser/policy bytes differ')
        if root is not None: require((root / difference['relative_path']).read_bytes() == p.raw(row),
                                    'Preserved main source differs: ' + difference['relative_path'])
    logical_ref = next(x for x in manifest['component_inputs'] if x['path'].endswith('/logical-inputs.json'))
    logical = p.json(logical_ref); differences = []
    require(len(logical) == 2617, 'Original diagnostic input inventory differs')
    for item in logical:
        name = item['relative_path']; before = item['source']['sha256']
        after = (by_path[name]['sha256'] if name in by_path else
                 provenance['preserved_six'][name]['sha256'] if name in provenance['preserved_six'] else before)
        if before != after: differences.append((name,before,after))
    require(sorted(differences) == sorted((x['relative_path'],x['diagnostic_sha256'],x['preserved_main_sha256'])
                for x in identity['preserved_published_differences']), 'Combined source versus diagnostic input differences differ')
    for row in handoff['canonical_leaf_bindings']:
        selected = by_path[row['relative_path']]
        require(selected['sha256'] == row['canonical_source']['sha256'] == row['publication_candidate']['sha256'],
                'Canonical CLI/Test262 source handoff differs')
    old_readme = p.raw(provenance['parent_README']); new_readme = p.raw(provenance['published_README'])
    note = provenance['README_insertion'].encode()
    require(new_readme.startswith(b'# Lila\n\n' + note) and new_readme.replace(note, b'', 1) == old_readme,
            'README insertion must preserve every original byte')
    block = re.search(br'<!-- lila-status:start -->.*?<!-- lila-status:end -->', old_readme, re.S)
    require(block is not None and digest(block[0]) == provenance['generated_status_block_sha256'] and
            block[0] in new_readme, 'Generated Test262 status block differs')
    if root is not None: require((root / 'README.md').read_bytes() == new_readme, 'Published README differs')
    # Producer chain retains its actual failed owner; the later two-stage owner is a separate PASS.
    core = proof['native']['optimized_core']; first = p.json(core['original_RED']); second = p.json(core['continuation_PASS'])
    source_clean(first, False); source_clean(second, True)
    require([x['exit_code'] for x in first['commands']] == [0]*7 + [1] and
            [x['exit_code'] for x in second['commands']] == [0, 0] and
            len(first['completed_native']) == 7 and len(second['completed_native']) == 2,
            'Seven-plus-two actual native producer closure differs')
    require('E0463' in p.raw(first['commands'][-1]['stderr']).decode(), 'Original missing-crate RED diagnostic absent')
    all_produced = first['completed_native'] + second['completed_native']
    adopted = core['artifact_bindings']
    require([x['artifact'] for x in all_produced] == adopted and len(adopted) == 9, 'Actual nominal producer artifact hashes differ')
    for row in first['commands'][:7] + second['commands']:
        require('opt-level=1' in row['exec_argv'] and 'debuginfo=0' in row['exec_argv'], 'Actual producer optimization profile differs')
    counts = {}
    focused = proof['native']['real_Test262_24']; owner = p.json(focused['owner']); source_clean(owner, True)
    require(len(owner['cohorts']) == len(owner['commands']) == 1 and owner['commands'][0]['exit_code'] == 0,
            'Exactly one actual focused Test262 cohort required')
    cohort = owner['cohorts'][0]; snapshot = p.json(cohort['native_json'])
    ids = p.raw(cohort['source_execution_list']).decode().splitlines()
    require(len(ids) == len(set(ids)) == 24 and set(ids) == set(snapshot['completed_test_ids']) == set(cohort['success_ids']) and
            snapshot['total'] == snapshot['passed'] == 24 and not snapshot['failures'] and not snapshot['timeout_test_ids'] and
            snapshot['counts_per_outcome'] == {'Bug': 0, 'Crash': 0, 'NotImplemented': 0, 'Success': 24} and
            snapshot['execution_backend'] == 'wasm-aot' and snapshot['compiler_identity']['source_fingerprint'] == FINGERPRINT,
            'Actual exact24 IDs, native backend, diagnostic identity or outcomes differ')
    command = owner['commands'][0]; argv = command['exec_argv']
    require(argv[argv.index('--jobs')+1] == argv[argv.index('--threads')+1] == '1' and
            argv[argv.index('--execution-backend')+1] == argv[argv.index('--case-harness')+1] == 'wasm-aot' and
            command['seconds'] == owner['actual_native_cohort_seconds'] == focused['native_command_seconds'],
            'Focused replay workers, backend or actual timing scope differs')
    require(owner['adopted_native_core'] == all_produced, 'Focused replay nominal core differs from actual producer chain')
    p.raw(command['stdout']); p.raw(command['stderr']); counts['real_Test262_Success'] = 24
    engine = proof['native']['engine_controls']; old = p.json(engine['original_RED']); current = p.json(engine['continuation_PASS'])
    source_clean(old, False); source_clean(current, True)
    require(not old['completed_tests'] and old['commands'][-1]['exit_code'] == -9 and
            not current['original_six_test_group_passed'] and not current['adopted_builtin_compiler_reexecuted'] and
            not current['adopted_builtin_inventory_reexecuted'] and len(current['commands']) == 4 and
            all(x['exit_code'] == 0 for x in current['commands']), 'Interrupted Engine owner or no-rebuild continuation differs')
    observed = current['observed_prefix_tests']; text = p.raw(old['commands'][-1]['stdout']).decode()
    require(len(observed) == 2 and [x['id'] for x in observed] == re.findall(r'^test (.+) \.\.\. ok$', text, re.M) and
            all(x['original_command_exit_code'] == -9 and not x['whole_group_passed'] and
                x['stdout'] == old['commands'][-1]['stdout'] for x in observed) and 'test result:' not in text and
            current['completed_native'][0]['artifact'] == old['completed_native'][0]['artifact'],
            'Exactly two literal prefix observations must remain separate from completed new tests')
    counts['engine_new_completed'] = successful_tests(p, current, current['completed_tests'])
    require(counts['engine_new_completed'] == 12, 'Exactly twelve remaining Engine controls required')
    counts['engine_prior_observed'] = 2
    cli = proof['native']['cli_controls']; previous = p.json(cli['earlier_RED']); current = p.json(cli['later_storage_RED'])
    source_clean(previous, False); source_clean(current, False)
    require(current['error'].startswith('Storage deferred:') and len(current['completed_tests']) == 4 and
            current['adopted_passed_tests'] == previous['completed_tests'] and len(previous['completed_tests']) == 5,
            'CLI control adoption must preserve both literal failed owners')
    counts['cli_selected_completed'] = successful_tests(p, previous, previous['completed_tests']) + successful_tests(p, current, current['completed_tests'])
    require(counts['cli_selected_completed'] == 9, 'Exact selected nine CLI controls required')
    for key, expected_count in [('aot_ir_controls', 6), ('test262_unit', 428)]:
        scope = proof['native'][key]
        if scope['status'] == 'pending':
            require(scope['native_acceptance'] is False, 'Pending scope cannot claim native acceptance')
            counts[key] = None
        else:
            require(scope['status'] == 'passed', 'Unknown optional native scope')
            owner = p.json(scope['owner']); source_clean(owner, True)
            counts[key] = successful_tests(p, owner, owner['completed_tests'])
            require(counts[key] == expected_count and all(x['exit_code'] == 0 for x in owner['commands']),
                    'Exact additional selected native scope differs')
            plan = p.json(owner['plan'])
            expected = [x['expected_selector'] for stage in plan['stages'] for x in stage.get('controls', [])]
            if key == 'test262_unit': expected = plan['test262_expected_unit_tests']
            require(set(expected) == {x['id'] for x in owner['completed_tests']}, 'Actual selected test IDs differ from sealed plan')
    require(proof['native_counts'] == counts, 'Reported native counts differ from raw evidence')
    for row in proof['preserved_RED_owners']:
        owner = p.json(row); require(not owner['passed'], 'A preserved RED owner was relabeled PASS')
    require(proof['acceptance'] == dict(focused_diagnostic_native_evidence=True, whole_published_main_native_acceptance=False,
        ordinary_combined_main_identity_revalidated=False, broad_retention1358=False, full_Test262=False,
        full_default_workspace=False, full_all_features=False, performance_speedup_claimed=False,
        ABI7_ABI8_acceptance=False), 'Acceptance scope was broadened')
    return dict(verified=True, selected_source_files=230, native_counts=counts, native_executions=0)


def main():
    parser = argparse.ArgumentParser(description=__doc__); parser.add_argument('checkpoint'); parser.add_argument('--root', type=pathlib.Path)
    args = parser.parse_args(); result = verify(json.loads(pathlib.Path(args.checkpoint).read_text()), args.root)
    print(json.dumps(result)); return 0


# This extension is copied into the standalone successor reader before main().
import posixpath

UNIT_PRODUCER_SHA = 'e249eca5f2bc0decde67aee39698f460a8887974873e5d155454cf8c85ac1798'
UNIT_PRODUCER_PLAN_SHA = '4792b88b1e40cc0e4a6491460148bb7093929c28d46e0be914e445ff9663a12a'
UNIT_EXECUTION_PLAN_SHA = '2f308ff1aba1394032030e283efe2f41f5997962f739487c4c955299971331e5'
UNIT_ELF_SHA = '1f4a564fb5c2758730c6f1af8007c4b0aefb2366b26c70ca0be9319b313214c0'


def verify_unit_producer(payloads, producer_ref):
    require(producer_ref['sha256'] == UNIT_PRODUCER_SHA, 'Actual unit producer RED anchor differs')
    producer = payloads.json(producer_ref); source_clean(producer, False)
    require(producer['plan']['sha256'] == UNIT_PRODUCER_PLAN_SHA and producer['error'].startswith('Storage deferred:') and
            len(producer['commands']) == len(producer['completed_native']) == 1 and
            not producer['completed_tests'] and not producer['native_inventories'],
            'Preserve actual compile PASS followed by storage RED, with no unit tests or inventory')
    plan = payloads.json(producer['plan']); stage = plan['stages'][0]
    command = producer['commands'][0]; native = producer['completed_native'][0]
    require(stage['id'] == command['id'] == native['id'] == 'test262_unit' and
            command['complete'] and command['exit_code'] == 0 and command['kind'] == 'test' and
            command['exec_argv'][-len(stage['rustc_argv']):] == stage['rustc_argv'] and
            command['cwd'] == plan['temporary'] and '--test' in stage['rustc_argv'] and
            'opt-level=1' in stage['rustc_argv'], 'Exact actual native unit producer argv/cwd differs')
    require(native['artifact']['sha256'] == UNIT_ELF_SHA and native['artifact']['bytes'] == 85654448 and
            native['artifact']['path'] == stage['output'] and native['dep_info']['path'] == stage['dep_info'] and
            native['source_coverage']['actual_files'] == 79 and native['source_coverage']['required_files'] == 11,
            'Actual unit ELF identity or 79/11 consumed-source coverage differs')
    child = payloads.json(native['child_preflight'])
    require(child['plan'] == producer['plan'] and child['exec_argv'] == command['exec_argv'] and
            child['cwd'] == child['tmpdir'] == plan['temporary'] and child['affinity'] == [3] and
            child['source_fences'] == producer['source_before'], 'Actual unit compiler preflight differs')
    text = payloads.raw(native['dep_info']).decode().replace('\\\n', '')
    require('\\ ' not in text, 'Unexpected dep-info escaping')
    names = set()
    for line in text.splitlines():
        if ': ' in line and not line.startswith('#'): names.update(line.split(': ', 1)[1].split())
    allowed = plan['source_map'] | {row['path']: row for row in plan['generated_inputs']}
    require(len(names) == 79 and sorted(names) == native['source_coverage']['observed_paths'] and
            len(stage['required_sources']) == 11 and stage['source'] in names and
            all(pathlib.PurePosixPath(name).is_absolute() and posixpath.normpath(name) in allowed for name in names),
            'Actual native unit dep-info input inventory differs')
    observed = {allowed[posixpath.normpath(name)]['path'] for name in names}
    require({allowed[name]['path'] for name in stage['required_sources']} <= observed,
            'A mandatory actual native unit source was omitted')
    payloads.raw(command['stdout']); payloads.raw(command['stderr'])
    return producer, plan, stage, native


ORIGINAL_VERIFY = verify


def verify(proof, root=None):
    result = ORIGINAL_VERIFY(proof, root)
    scope = proof['native']['test262_unit']
    require('producer_storage_RED' in scope, 'Separate actual unit compiler RED provenance required')
    p = Payloads(proof)
    producer, producer_plan, stage, native = verify_unit_producer(p, scope['producer_storage_RED'])
    require(scope['producer_storage_RED'] in proof['preserved_RED_owners'], 'Original unit storage RED must remain preserved')
    if scope['status'] == 'pending':
        require(not scope['native_acceptance'], 'A compiled but unexecuted unit ELF cannot claim unit acceptance')
        return result
    owner = p.json(scope['owner']); plan = p.json(owner['plan'])
    require(owner['plan']['sha256'] == UNIT_EXECUTION_PLAN_SHA and not plan['stages'] and
            plan['previous_unit_compile_plan'] == producer['plan'] and
            plan['previous_unit_compile_storage_RED'] == scope['producer_storage_RED'] and
            plan['adopted_unit_stage'] == stage and plan['adopted_unit_native'] == native == owner['adopted_unit_native'] and
            owner['previous_unit_compile_storage_RED'] == scope['producer_storage_RED'] and
            not owner['completed_native'] and owner['compiler_phases_attempted'] == owner['rustc_invocations'] == 0,
            'Execution-only unit owner must adopt the actual producer ELF without recompilation')
    require([row['id'] for row in owner['commands']] == ['test262_unit-inventory','test262_unit-all'] and
            all(row['complete'] and row['exit_code'] == 0 and native['artifact']['path'] in row['exec_argv'] for row in owner['commands']) and
            len(owner['native_inventories']) == 1,
            'Exactly unit inventory plus execution commands required')
    inventory = owner['native_inventories'][0]; ids = inventory['ids']
    lines = p.raw(inventory['stdout']).decode().splitlines()
    require(len(ids) == len(set(ids)) == 428 and all(line.endswith(': test') for line in lines if line) and
            [line[:-6] for line in lines if line.endswith(': test')] == ids and
            ids == [row['id'] for row in owner['completed_tests']] and
            set(ids) == set(plan['test262_expected_unit_tests']), 'Exact actual native 428 inventory/execution ID set differs')
    run = owner['commands'][1]
    require('--include-ignored' in run['exec_argv'] and '--test-threads=1' in run['exec_argv'] and
            'test result: ok. 428 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;' in p.raw(run['stdout']).decode(),
            'All 428 native unit tests must complete without filtering or ignore')
    old_cli = p.json(proof['native']['cli_controls']['earlier_RED'])
    new_cli = p.json(proof['native']['cli_controls']['later_storage_RED'])
    require(owner['adopted_new_CLI_passes'] == new_cli['completed_tests'] and
            owner['adopted_prior_CLI_passes'] == old_cli['completed_tests'] and
            owner['actual_CLI_passes_adopted'] == 9 and owner['CLI_tests_rerun'] == 0,
            'Execution-only unit owner must retain the separate actual CLI4+5 facts')
    return result


# Copied into the standalone main-join successor before main().
JOIN_PARENT = '30fb4b9a27390fa508ce86f31aa25dd06ac51bd4'
PARTIAL_UNIT_SHA = '2c823393c17daf88fa554fab847c71c3eca455e14d2dc601635c8f3785b6099b'
UNIT_REMAINING_CASE = 'conformance_closure::tests::fixture_suite_cannot_enter_the_product_release_factory'


def git_object_oid(kind, raw):
    return hashlib.sha1(kind.encode()+b' '+str(len(raw)).encode()+b'\0'+raw).hexdigest()


def verify_main_join(proof, payloads):
    join=proof['publication_join']
    require(proof['parent']==join['parent']==JOIN_PARENT and join['reviewed_source_base']==BASE,
            'Exact root-published CI main successor required')
    commit=payloads.raw(join['actual_parent_commit'])
    require(git_object_oid('commit',commit)==JOIN_PARENT,'New parent Git commit object differs')
    headers=commit.split(b'\n\n',1)[0].decode().splitlines()
    require([x[7:] for x in headers if x.startswith('parent ')]==[BASE] and
            next(x[5:] for x in headers if x.startswith('tree '))==join['root_tree'],'New parent ancestry/tree differs')
    trees={}
    for oid,row in join['tree_objects'].items():
        raw=payloads.raw(row);require(git_object_oid('tree',raw)==oid,'Embedded Git tree object differs')
        entries={};index=0
        while index<len(raw):
            space=raw.index(b' ',index);null=raw.index(b'\0',space+1)
            require(len(raw[null+1:null+21])==20,'Incomplete Git tree entry')
            entries[raw[space+1:null].decode()]={'mode':raw[index:space].decode(),'oid':raw[null+1:null+21].hex()};index=null+21
        trees[oid]=entries
    def lookup(name):
        oid=join['root_tree'];parts=pathlib.PurePosixPath(name).parts
        for index,part in enumerate(parts):
            require(oid in trees,'Missing Git tree proof on reviewed source path')
            entry=trees[oid].get(part)
            if entry is None:return None
            if index==len(parts)-1:return entry
            require(entry['mode']=='40000','Git source parent is not a directory');oid=entry['oid']
    for row in proof['source']['selected_files']:
        entry=lookup(row['relative_path'])
        require((entry['oid'] if entry else '0'*40)==row['base_blob_oid'] and
                (not entry or entry['mode']==row['mode']),'New parent changed a reviewed source base blob/mode')
    for name,row in proof['source']['preserved_six'].items():
        entry=lookup(name);require(entry and entry['oid']==git_object_oid('blob',payloads.raw(row)),
                                  'New parent parser/policy blob differs')
    require(lookup('README.md')['oid']==git_object_oid('blob',payloads.raw(proof['source']['parent_README'])),
            'ABI6 insertion must preserve the exact new-main README')
    expected={'.github/workflows/aot-cache-fingerprint.yaml','.github/workflows/aot-product-validation.yaml',
        '.github/workflows/aot-regressions.yaml','.github/workflows/ci.yaml','README.md',
        'docs/rust-rewrite/workflow-yaml-check-20261015.json'}
    require(set(join['independent_CI_changed_paths'])==expected and not join['current_main_native_acceptance'] and
            not join['diagnostic_compiler_inputs_changed_by_CI'],'Independent CI join scope differs')


def verify_unit_partial(payloads, scope):
    require(scope['partial_execution_RED']['sha256']==PARTIAL_UNIT_SHA,'Actual failed428 owner anchor differs')
    owner=payloads.json(scope['partial_execution_RED']);source_clean(owner,False)
    plan=payloads.json(owner['plan']);_,_,_,native=verify_unit_producer(payloads,scope['producer_storage_RED'])
    require(owner['plan']['sha256']==UNIT_EXECUTION_PLAN_SHA and not plan['stages'] and
            plan['adopted_unit_native']==native==owner['adopted_unit_native'] and
            not owner['completed_native'] and not owner['completed_tests'] and
            owner['compiler_phases_attempted']==owner['rustc_invocations']==0 and
            [x['id'] for x in owner['commands']]==['test262_unit-inventory','test262_unit-all'] and
            [x['exit_code'] for x in owner['commands']]==[0,101],
            'Preserve actual no-compiler428 group RED with no fabricated completed_tests')
    inventory=owner['native_inventories'][0];ids=inventory['ids']
    lines=payloads.raw(inventory['stdout']).decode().splitlines()
    require(len(ids)==len(set(ids))==428 and [x[:-6] for x in lines if x.endswith(': test')]==ids and
            set(ids)==set(plan['test262_expected_unit_tests']),'Exact failed-group428 inventory differs')
    text=payloads.raw(owner['commands'][1]['stdout']).decode()
    observed=re.findall(r'^test (.+) \.\.\. ok$',text,re.M)
    failed=re.findall(r'^test (.+) \.\.\. FAILED$',text,re.M)
    require(len(observed)==len(set(observed))==427 and failed==[UNIT_REMAINING_CASE] and
            set(observed)==set(ids)-{UNIT_REMAINING_CASE} and
            'test result: FAILED. 427 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out;' in text and
            'closure requires the current product checkout and its pinned suite' in payloads.raw(owner['commands'][1]['stderr']).decode(),
            'Exactly427 literal observed passes and the unchanged fixture-root failure required')
    return owner,plan,native,observed


PRE_JOIN_VERIFY = verify


def verify(proof, root=None):
    payloads=Payloads(proof);verify_main_join(proof,payloads)
    scope=proof['native']['test262_unit']
    # Existing readers validate all other actual owners and the unit compiler RED.
    # The old full-unit-owner rule is replaced only by the explicit427+1 proof below.
    proxy=dict(proof,parent=BASE,native=dict(proof['native']),native_counts=dict(proof['native_counts']))
    proxy['native']['test262_unit']=dict(scope,status='pending',native_acceptance=False)
    proxy['native_counts']['test262_unit']=None
    result=PRE_JOIN_VERIFY(proxy,root)
    partial,partial_plan,native,observed=verify_unit_partial(payloads,scope)
    require(scope['partial_execution_RED'] in proof['preserved_RED_owners'],'Failed428 owner must remain literal RED')
    if scope['status']=='pending':
        require(proof['native_counts']['test262_unit'] is None and not scope['native_acceptance'],
                'Observed427 alone cannot claim full428 acceptance')
        result['unit_observed_in_failed_group']=427;return result
    require(scope['status']=='passed','Unknown final unit aggregate state')
    one=payloads.json(scope['owner']);source_clean(one,True);one_plan=payloads.json(one['plan'])
    require(one['plan']==scope['remaining_case_plan'] and len(one['commands'])==1 and
            one['commands'][0]['complete'] and one['commands'][0]['exit_code']==0 and
            not one['completed_native'],'Only a separate actual one-case owner with no compiler may complete428')
    command=one['commands'][0];argv=command['exec_argv']
    require(native['artifact']['path'] in argv,'Single remaining case must execute the original unit ELF')
    suffix=argv[argv.index(native['artifact']['path'])+1:]
    require(suffix.count(UNIT_REMAINING_CASE)==1 and all(x in [UNIT_REMAINING_CASE,'--exact','--nocapture',
            '--test-threads=1','--include-ignored'] for x in suffix) and '--exact' in suffix and '--test-threads=1' in suffix,
            'Only the one unchanged exact failed case may rerun')
    text=payloads.raw(command['stdout']).decode()
    require(re.findall(r'^test (.+) \.\.\. ok$',text,re.M)==[UNIT_REMAINING_CASE] and
            'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 427 filtered out;' in text,
            'Remaining case must actually pass with the other427 intentionally filtered')
    environments=[value for value in one_plan.values() if isinstance(value,dict) and 'GIT_DIR' in value and 'GIT_WORK_TREE' in value]
    require(environments and all(env['GIT_WORK_TREE']=='/workspace/porffor-next' and
            env['GIT_DIR']==proof['publication_join']['source_checkout_git_dir'] for env in environments),
            'Explicit unchanged product Git context required while private cwd is retained')
    child_ref=next((row['child_preflight'] for row in one.get('completed_tests',[]) if row['id']==UNIT_REMAINING_CASE),None)
    if child_ref is None:child_ref=scope['remaining_case_child_preflight']
    child=payloads.json(child_ref)
    require(child['plan']==one['plan'] and child['exec_argv']==argv and child['affinity']==[3] and
            child['cwd']==child['tmpdir']==command['cwd'] and child['cwd'].startswith('/workspace/.lila-tools/') and
            child['source_fences']==one['source_before'],'Exact one-case private cwd/preflight differs')
    require(len(set(observed)|{UNIT_REMAINING_CASE})==428 and proof['native_counts']['test262_unit']==428,
            'Actual427+one aggregate must cover exactly428 IDs')
    result['native_counts']=dict(proof['native_counts']);result['unit_observed_in_failed_group']=427
    result['unit_newly_completed']=1;return result


# Exact final native-only Git context contract, copied into the standalone reader.
STRICT_JOIN_VERIFY = verify


def verify(proof, root=None):
    result=STRICT_JOIN_VERIFY(proof,root);scope=proof['native']['test262_unit']
    if scope['status']=='pending':return result
    p=Payloads(proof);one=p.json(scope['owner']);plan=p.json(one['plan'])
    require(one['plan']['sha256']=='7e5f2b9c6f84576875de1c801e675f96b673d94be47dbc853d45200d9952cfcf',
            'Exact sealed one-case Git-context plan required')
    partial,_,native,observed=verify_unit_partial(p,scope)
    require(plan['adopted_unit_native']==native==one['adopted_unit_native'] and
            plan['adopted_427_passed_ids']==observed==one['adopted_427_passed_ids'] and
            plan['previous_427_PASS_one_FAIL_RED']==scope['partial_execution_RED']==one['previous_427_PASS_one_FAIL_RED'] and
            one['rustc_invocations']==one['compiler_phases_attempted']==one['previous_427_controls_rerun']==one['CLI_tests_rerun']==0 and
            one['unit_controls_verified_across_owners']==428 and not one['aggregate_is_a_producer_owner'],
            'Exact original ELF/427 observation adoption and zero rerun/recompile contract differs')
    env=plan['native_exec_git_environment'];context=p.json(plan['git_context_proof'])
    source=one['source_before']['/workspace/porffor-next']
    require(env=={'GIT_DIR':proof['publication_join']['source_checkout_git_dir'],
                'GIT_WORK_TREE':'/workspace/porffor-next','GIT_OPTIONAL_LOCKS':'0'} and
            context['native_exec_git_environment']==env and context['observed_toplevel']=='/workspace/porffor-next' and
            context['observed_head']==source['head'] and context['index_sha256']==source['index_sha256'],
            'Actual frozen product Git context proof differs')
    child=p.json(one['completed_tests'][0]['child_preflight'])
    require(child['native_exec_git_environment']==env and child['git_context_added_after_normal_source_guards'] and
            child['source_guard_environment_git_selection']=={'GIT_DIR':None,'GIT_WORK_TREE':None} and
            child['parent_environment_unchanged'], 'Git context must be added only after normal actual child source guards')
    return result


# Fresh source successor and47-control scope, copied into the standalone reader.
CFG_BASE_VERIFY = verify
CFG_SOURCE_SHA = 'db1bc521fe1a375f59d1cab67066c411257901573617b8a98070edfad4c6ba4e'
CFG_COMPONENT_SHA = '6d2741c61757c4ea870e4ff0fabb8da1a329d0da92ee57f2a45df0d8f024219d'
AOT_COMPILE_RED_SHA = 'e3483eb0e47b1800b643229c9052e0d99bf67c819857218e1395479355b78727'


def verify(proof, root=None):
    # Original230 source/identity and old six-control logic remain as provenance.
    # Current231 source and new47 controls have their own explicit checks below.
    proxy=dict(proof,native=dict(proof['native']),native_counts=dict(proof['native_counts']))
    proxy['native']['aot_ir_controls']=dict(status='pending',native_acceptance=False)
    proxy['native_counts']['aot_ir_controls']=None
    result=CFG_BASE_VERIFY(proxy,None)
    p=Payloads(proof);successor=proof['source_successor']
    require(successor['manifest']['sha256']==CFG_SOURCE_SHA,'Exact cfg(test) source successor required')
    manifest=p.json(successor['manifest']);component=p.json(manifest['component'])
    require(manifest['component']['sha256']==CFG_COMPONENT_SHA and manifest['selected_file_count']==231 and
        manifest['publisher_file_count']==234 and manifest['original230_paths_retained'] and
        not manifest['ordinary_combined_identity_claimed'] and not manifest['whole_current_main_native_acceptance'],
        'Selected source count, origin or identity scope differs')
    old={x['relative_path']:x for x in proof['source']['selected_files']}
    current={x['relative_path']:x for x in manifest['selected_files']}
    planning='crates/lila-aot-wasm/src/planning.rs'
    profile='crates/lila-aot-wasm/src/runtime_function/runtime_profile/tests.rs'
    require(len(current)==231 and set(old)<=set(current) and set(current)-set(old)=={planning} and
        [name for name in old if old[name]['sha256']!=current[name]['candidate']['sha256']]==[profile],
        'All original230 paths must remain; only one replacement and planning addition are allowed')
    for name,row in current.items():
        raw=p.raw(row['candidate'])
        require(git_object_oid('blob',raw)==row['candidate_blob_oid'],'Published successor Git blob differs')
        if root is not None:require((root/name).read_bytes()==raw,'Published current source differs: '+name)
    for repair in manifest['repairs']:
        before=p.raw(repair['base']);after=p.raw(repair['candidate']);name=repair['relative_path']
        require(repair['candidate']['sha256']==current[name]['candidate']['sha256'],'Repaired source binding differs')
        if name==planning:
            line=b'            crate::runtime_artifact::ModuleKind::Standalone,\n'
            require(after.count(line)==1 and after.replace(line,b'',1)==before,'Only exact test constructor witness may change')
            start=after.index(b'#[cfg(test)]\nmod tests {');prefix=after[start:after.index(line)]
            require(b'fn plan_metas(' in prefix and b'fn lower_script(' not in prefix and
                prefix.count(b'{')-prefix.count(b'}')==2,'Changed tokens must be inside the inline cfg(test) helper')
        else:
            require(name==profile,'Foreign cfg(test) repair')
            old_line=b'    let identity = selected.identity().artifact_identity().as_bytes();\n'
            new_line=b'    let artifact_identity = selected.identity().artifact_identity();\n    let identity = artifact_identity.as_bytes();\n'
            require(before.count(old_line)==1 and after==before.replace(old_line,new_line,1) and
                p.raw(repair['scope']['parent_source']).count(b'#[cfg(test)]\nmod tests;')==1,
                'Only lifetime witness in a cfg(test)-gated file may change')
    core=proof['native']['optimized_core']
    first=p.json(core['original_RED']);second=p.json(core['continuation_PASS'])
    require(all('--test' not in row['exec_argv'] for row in first['commands'][:7]+second['commands']),
        'Adopted production nominal producers must have normal cfg, not cfg(test)')
    original=p.json(proof['source']['candidate_manifest'])
    logical=p.json(next(x for x in original['component_inputs'] if x['path'].endswith('/logical-inputs.json')))
    differences=[]
    for item in logical:
        name=item['relative_path'];before=item['source']['sha256']
        after=(current[name]['candidate']['sha256'] if name in current else
            proof['source']['preserved_six'][name]['sha256'] if name in proof['source']['preserved_six'] else before)
        if before!=after:differences.append((name,before,after))
    require(len(differences)==8 and len(logical)-len(differences)==2609 and
        sorted(differences)==sorted((x['relative_path'],x['diagnostic_sha256'],x['published_sha256']) for x in manifest['explicit_differences']),
        'Two cfg(test) byte changes plus six published differences must remain explicit')
    if root is not None:
        for name,row in proof['source']['preserved_six'].items():require((root/name).read_bytes()==p.raw(row),'Preserved companion differs')
        require((root/'README.md').read_bytes()==p.raw(proof['source']['published_README']),'Published joined README differs')
    red_ref=successor['original_AOT_compile_RED'];require(red_ref['sha256']==AOT_COMPILE_RED_SHA,'Original AOT compile RED anchor differs')
    red=p.json(red_ref);source_clean(red,False)
    require(not red['completed_tests'] and not red['completed_native'] and len(red['commands'])==1 and
        red['commands'][0]['id']=='aot_unit' and red['commands'][0]['exit_code']==1 and red_ref in proof['preserved_RED_owners'],
        'Original failed AOT compile must remain literal RED with no invented tests')
    scope=proof['native']['aot_ir_controls']
    if scope['status']=='passed':
        owner,plan,actual,expected,required=verify_affected_aggregate(p,scope,proof)
        checks=p.json(manifest['component_checks']);witnesses=checks['all_historical_test_functions_retained']
        require(len(witnesses)==40 and all(sum(name==x['name'] or name.endswith('::'+x['name']) for _,name in actual)==1 for x in witnesses),
            'All39 planning and one runtime-profile witnesses must actually pass')
        require(len(actual-required)==41,'Exactly41 affected existing witnesses must supplement original6')
        aot_stage=next(stage for stage in plan['stages'] if stage['id']=='aot_unit')
        coverage=next(row['source_coverage'] for row in owner['completed_native'] if row['id']=='aot_unit')
        for repair in manifest['repairs']:
            matches=[name for name,row in plan['source_map'].items() if name.endswith('/'+repair['relative_path']) and row['sha256']==repair['candidate']['sha256']]
            require(matches and any(name in aot_stage['required_sources'] for name in matches) and
                any(posixpath.normpath(path) in matches for path in coverage['observed_paths']),
                'Actual cfg(test) compile must consume each exact repaired mandatory source')
        result['native_counts']=dict(proof['native_counts']);require(result['native_counts']['aot_ir_controls']==47,'Reported affected control count differs')
    else:require(scope['status']=='pending' and proof['native_counts']['aot_ir_controls'] is None,'Unrun corrected AOT controls cannot pass implicitly')
    result['selected_source_files']=231;result['explicit_diagnostic_input_differences']=8
    return result


# Fresh independent main publication over the historical30fb/native proof.
PUBLICATION_BASIS_VERIFY = verify


def verify(proof, root=None):
    if 'publication_successor' not in proof:
        require(root is None,'Latest-main publication proof required for a materialized publication')
        return PUBLICATION_BASIS_VERIFY(proof,None)
    joined=proof['publication_successor'];historical=proof['historical_publication_readme']
    require(joined['historical_parent']==JOIN_PARENT and joined['parent']==proof['parent'] and
        joined['parent']!=JOIN_PARENT and not joined['whole_current_main_native_acceptance'],
        'Explicit independent main successor required')
    proxy=dict(proof,parent=JOIN_PARENT,source=dict(proof['source'],**historical))
    result=PUBLICATION_BASIS_VERIFY(proxy,None);p=Payloads(proof)
    trees={}
    for oid,row in joined['tree_objects'].items():
        raw=p.raw(row);require(git_object_oid('tree',raw)==oid,'Latest-main Git tree object differs')
        entries={};index=0
        while index<len(raw):
            space=raw.index(b' ',index);null=raw.index(b'\0',space+1)
            require(len(raw[null+1:null+21])==20,'Incomplete current-main Git tree entry')
            name=raw[space+1:null].decode();require(name not in entries,'Duplicate Git tree entry')
            entries[name]=dict(mode=raw[index:space].decode(),oid=raw[null+1:null+21].hex());index=null+21
        trees[oid]=entries
    commits=joined['commits'];require(0<len(commits)<=16,'Bounded actual independent main ancestry required')
    previous=JOIN_PARENT
    for row in commits:
        raw=p.raw(row['commit']);require(git_object_oid('commit',raw)==row['oid'],'Current-main commit object differs')
        headers=raw.split(b'\n\n',1)[0].decode().splitlines()
        require([x[7:] for x in headers if x.startswith('parent ')]==[previous] and row['parent']==previous and
            next(x[5:] for x in headers if x.startswith('tree '))==row['root_tree'],'Current-main ancestry or root differs')
        previous=row['oid']
    require(previous==proof['parent'] and joined['root_tree']==commits[-1]['root_tree'] and
        joined['historical_root_tree']==proof['publication_join']['root_tree'],'Current-main endpoint differs')
    def load(oid):
        require(oid in trees,'Missing changed/current-source tree object');return trees[oid]
    changed=set()
    def compare(before,after,prefix=''):
        if before==after:return
        a=load(before) if before else {};b=load(after) if after else {}
        for name in sorted(set(a)|set(b)):
            left=a.get(name);right=b.get(name)
            if left==right:continue
            path=prefix+name
            if (not left or left['mode']=='40000') and (not right or right['mode']=='40000'):
                compare(left['oid'] if left else None,right['oid'] if right else None,path+'/')
            else:changed.add(path)
    compare(joined['historical_root_tree'],joined['root_tree'])
    require(sorted(changed)==joined['changed_paths'],'Actual independent main changed paths differ')
    source=p.json(proof['source_successor']['manifest']);original=p.json(proof['source']['candidate_manifest'])
    logical=p.json(next(x for x in original['component_inputs'] if x['path'].endswith('/logical-inputs.json')))
    blocked={x['relative_path'] for x in source['selected_files']}|set(proof['source']['preserved_six'])|{x['relative_path'] for x in logical}
    blocked|={'docs/rust-rewrite/abi6-native-repairs-20261015.json','docs/rust-rewrite/verify-abi6-native-repairs-20261015.py'}
    require(not changed.intersection(blocked) and joined['selected_source_bytes_and_modes_unchanged'] and
        joined['diagnostic_inputs_unchanged'] and joined['all_six_companions_unchanged'],
        'Independent main must preserve all selected, diagnostic and historical companion source')
    def lookup(name):
        oid=joined['root_tree'];parts=pathlib.PurePosixPath(name).parts
        for index,part in enumerate(parts):
            entry=load(oid).get(part)
            if entry is None:return None
            if index==len(parts)-1:return entry
            require(entry['mode']=='40000','Current-main source parent is not a directory');oid=entry['oid']
    for row in source['selected_files']:
        entry=lookup(row['relative_path'])
        require((entry['oid'] if entry else '0'*40)==row['base_blob_oid'] and
            (not entry or entry['mode']==row['mode']),'Current-main selected source base or mode differs')
        if root is not None:require((root/row['relative_path']).read_bytes()==p.raw(row['candidate']),'Current published source differs')
    for name,row in proof['source']['preserved_six'].items():
        require(lookup(name)['oid']==git_object_oid('blob',p.raw(row)),'Current-main parser/policy companion differs')
        if root is not None:require((root/name).read_bytes()==p.raw(row),'Current published companion differs')
    parent=p.raw(proof['source']['parent_README']);published=p.raw(proof['source']['published_README'])
    require(lookup('README.md')['oid']==git_object_oid('blob',parent) and parent.startswith(b'# Lila\n\n') and
        proof['source']['README_insertion']==historical['README_insertion'] and
        published==b'# Lila\n\n'+historical['README_insertion'].encode()+parent[len(b'# Lila\n\n'):],
        'README insertion must preserve every actual current-main byte')
    pattern=br'<!-- lila-status:start -->.*?<!-- lila-status:end -->'
    require(re.search(pattern,parent,re.S)[0]==re.search(pattern,p.raw(historical['parent_README']),re.S)[0],
        'Current main changed generated status')
    if root is not None:require((root/'README.md').read_bytes()==published,'Current published README differs')
    result['publication_parent']=proof['parent'];result['independent_main_changes']=len(changed)
    return result


# The exact corrected test-only namespace and captured wasmparser contract.
AFFECTED_BASE_VERIFY = verify
NAMESPACE_RED_SHA = '4930dbcb1e88112e0711945fcd7fcf22f80575aefe22eac66a8cfdb204657d93'


def verify(proof, root=None):
    result=AFFECTED_BASE_VERIFY(proof,root)
    scope=proof['native']['aot_ir_controls']
    if scope['status']!='passed':return result
    p=Payloads(proof);owner=p.json(scope['aot_prefix_RED']);plan=p.json(owner['plan'])
    require(scope['corrected_plan']==owner['plan'],'Actual corrected namespace plan must be explicit')
    held=scope['held_namespace_RED'];require(held['sha256']==NAMESPACE_RED_SHA and held in proof['preserved_RED_owners'],
        'Original missing-namespace compile RED must remain literal')
    failed=p.json(held);source_clean(failed,False)
    require(not failed['completed_native'] and not failed['completed_tests'] and len(failed['commands'])==1 and
        failed['commands'][0]['exit_code']==1 and 'tests/unit/code_sink.rs' in p.raw(failed['commands'][0]['stderr']).decode(),
        'Missing namespace attempt cannot supply produced artifacts or passing controls')
    stage=next(x for x in plan['stages'] if x['id']=='aot_unit')
    projection=p.json(plan['unit_source_projection']);expansion=p.json(plan['unit_actual_expansion'])
    binding=p.json(plan['unit_dependency_binding'])
    require(projection['component']['sha256']==CFG_COMPONENT_SHA and len(projection['aliases'])==875 and
        set(projection['aliases'])==set(projection['origins'])==set(projection['transport']) and
        len(stage['required_sources'])==len(set(stage['required_sources']))==648 and
        set(stage['required_sources'])==set(projection['actual_required_sources']),
        'Complete selected875-member namespace and exact648 mandatory inputs required')
    require(expansion['producer_receipt']==proof['source_successor']['original_AOT_compile_RED'] and
        len(expansion['normalized_inputs'])==len(set(expansion['normalized_inputs']))==648 and
        expansion['old_623_required_are_actual'] and not expansion['compiler_executed_during_preparation'] and
        len(expansion['generated_binary_inputs'])==5 and len(expansion['outlined_unit_inputs'])==5,
        'Expanded namespace must derive from the unchanged actual earlier compiler dep-info')
    # Consume the original dep-info bytes as evidence, rather than treating a guessed list as native coverage.
    require(expansion['dep_info']['sha256']=='a9e6aef3302371b3de856d7ff99df8b7d8255a35a5d454d68635d1f9a3ad5a0b',
        'Exact actual earlier compiler dep-info required')
    dep=p.raw(expansion['dep_info']).decode();require(all(name in dep for name in expansion['generated_binary_inputs']),
        'Captured actual dep-info must include all five generated binary images')
    coverage=next(x['source_coverage'] for x in owner['completed_native'] if x['id']=='aot_unit')
    observed={posixpath.normpath(x) for x in coverage['observed_paths']}
    require(set(stage['required_sources'])<=observed,'Actual successful cfg(test) producer must consume all648 mandatory inputs')
    generated={item['path']:item for item in plan['generated_inputs']}
    captured=dict(plan['source_map'])
    for name,item in generated.items():
        require(name not in captured or all(captured[name][key]==item[key] for key in ['sha256','bytes']),
            'Source/generated input bindings collide')
        captured[name]=item
    require({name for name in stage['required_sources'] if name not in plan['source_map']}==
        set(expansion['generated_binary_inputs']) and
        all(generated[name] in plan['consumed_inputs'] for name in expansion['generated_binary_inputs']) and
        {str(pathlib.PurePosixPath(name).parent) for name in expansion['generated_binary_inputs']}=={stage['environment']['OUT_DIR']},
        'Exactly five captured producer OUT_DIR images supplement the643 source bindings')
    for name in stage['required_sources']:
        require(name in captured,'Every mandatory test input needs an exact captured source or generated binding')
        if name in projection['aliases']:
            alias=projection['aliases'][name];origin=projection['origins'][name]
            require(alias['sha256']==origin['sha256']==captured[name]['sha256'] and
                alias['bytes']==origin['bytes']==captured[name]['bytes'],
                'Selected namespace alias, actual origin and compiler source binding differ')
    parser=binding['parser'];encoder=binding['encoder']
    require(parser['producer']['package_id'].endswith('#wasmparser@0.245.1') and
        encoder['producer']['package_id'].endswith('#wasm-encoder@0.245.1') and
        parser['producer']['target']['name']=='wasmparser' and encoder['producer']['target']['name']=='wasm_encoder',
        'Captured direct parser and encoder must both be Cargo version0.245.1')
    for key,item in [('wasmparser',parser),('wasm_encoder',encoder)]:
        expected=pathlib.PurePosixPath(item['native']['path']).name
        require(pathlib.PurePosixPath(stage['externs'][key]).name==expected and item in plan['native_externs'] and
            item['native'] in plan['closed_lookup_inputs'] and '--extern' in stage['rustc_argv'] and
            key+'='+stage['externs'][key] in stage['rustc_argv'],
            'Actual direct rustc extern must bind the exact captured native Cargo producer')
        p.raw(item['fingerprint_stamp']);p.json(item['fingerprint'])
    stamp=int.from_bytes(bytes.fromhex(p.raw(parser['fingerprint_stamp']).decode()),'little')
    require(stamp==binding['exact_parser_stamp'] and
        [row[3] for row in p.json(encoder['fingerprint'])['deps'] if row[1]=='wasmparser']==[stamp],
        'Encoder fingerprint must identify the same exact parser nominal dependency')
    manifest=p.raw(binding['package_manifest']).decode()
    require('wasmparser = "0.245.1"' in manifest and
        'wasm-encoder = { version = "0.245.1", features = ["wasmparser"] }' in manifest and
        binding['lookup_members_unchanged'] and binding['conversions_and_assertions_unchanged'],
        'Captured Cargo dependency and unchanged test assertions must remain explicit')
    result['AOT_actual_required_inputs']=648;result['AOT_selected_namespace_members']=875
    result['AOT_direct_wasmparser_version']='0.245.1'
    return result


# Preserve the explicit index-stat transition between historical and new owners.
FENCE_BASIS_VERIFY = verify


def verify(proof, root=None):
    result=FENCE_BASIS_VERIFY(proof,root);scope=proof['native']['aot_ir_controls']
    if scope['status']!='passed':return result
    p=Payloads(proof);owner=p.json(scope['aot_prefix_RED']);plan=p.json(owner['plan'])
    adoption_ref=plan['source_fence_adoption']
    require(adoption_ref['sha256']=='6373f8daae48bba2f91e93c3065bb6921e4ac2581996f565e0d1c6568776e362',
        'Exact root-approved physical index transition adoption required')
    adoption=p.json(adoption_ref);transition_ref=adoption['root_transition_receipt']
    require(transition_ref['sha256']=='055af45bd76bc6c4ead9335daad54ba53dd549c2451076026e0bb601542c929f',
        'Exact actual root index-stat audit receipt required')
    transition=p.json(transition_ref)
    require(adoption['passed'] and adoption['only_expected_index_changed'] and
        adoption['semantic_index_equals_HEAD_and_worktree'] and adoption['historical_receipts_untouched'] and
        not adoption['new_source_hardlinks_created_after_admission'] and owner['source_before']==adoption['after'],
        'New owner must bind the explicit adopted fence without rewriting historical owners')
    held=p.json(scope['held_namespace_RED'])
    require(held['source_before']==adoption['before'] and held['source_after']==adoption['before'],
        'Historical namespace failure must retain its original physical source fence')
    require(transition['schema']=='explicit-frozen-index-stat-transition-v1' and transition['complete'] and
        transition['accepted_for_successor_admission'] and transition['all_index_entries_match_unchanged_frozen_HEADs'] and
        transition['all_present_tracked_working_bytes_and_modes_match_HEADs'] and
        transition['all_absent_index_entries_declared_sparse'] and not transition['content_or_mode_errors'] and
        not transition['old_owner_literal_source_unchanged'] and not transition['old_results_or_guard_edited'] and
        not transition['source_HEAD_or_index_reset_performed'] and not transition['protected_802_inode_changes'],
        'Metadata adoption must preserve the literal old guard failure and unchanged source content/modes')
    roots={row['root']:row for row in transition['roots']}
    require(set(roots)==set(adoption['before'])==set(adoption['after']),'Exactly three frozen-root fences required')
    changed=[]
    for name,row in roots.items():
        before=adoption['before'][name];after=adoption['after'][name]
        require(before['head']==after['head']==row['head'] and before['clean'] and after['clean'] and
            row['semantic_index_matches_HEAD'] and row['working_tree_clean'] and
            before['index_sha256']==row['historical_raw_index_sha256'] and
            after['index_sha256']==row['current_raw_index_sha256'],
            'Physical index transition must preserve frozen HEAD and checked source semantics')
        if before['index_sha256']!=after['index_sha256']:changed.append(name)
    require(changed==['/workspace/porffor-next'],'Only the explicitly audited index-stat bytes may differ')
    result['explicit_index_stat_transition_preserved']=True
    return result


# Resolve captured suffix contracts against actual successful native inventory.
def resolve_affected_controls(payloads,owner,plan):
    result=set()
    for stage in plan['stages']:
        inventories=[x for x in owner['commands'] if x['id']==stage['id']+'-inventory']
        require(len(inventories)==1 and inventories[0]['complete'] and inventories[0]['exit_code']==0,
            'Each selected native target needs its actual successful inventory command')
        command=inventories[0]
        produced=[x for x in owner['completed_native'] if x['id']==stage['id']]
        require(len(produced)==1 and produced[0]['artifact']['path'] in command['exec_argv'] and
            '--list' in command['exec_argv'] and '--format=terse' in command['exec_argv'],
            'Inventory must list the exact actual produced native target')
        names=[line[:-6] for line in payloads.raw(command['stdout']).decode().splitlines() if line.endswith(': test')]
        require(names and len(names)==len(set(names)),'Actual target inventory must contain distinct test IDs')
        selected=[]
        for control in stage['controls']:
            suffix=control['name_suffix'];matches=[name for name in names if name==suffix or name.endswith('::'+suffix)]
            require(len(matches)==1 and (control.get('known_selector') is None or matches==[control['known_selector']]),
                'Each declared suffix must resolve to exactly its actual native selector')
            selected.append(matches[0])
        require(len(selected)==len(set(selected)),'Selected controls must resolve to distinct native IDs')
        result.update((stage['id'],name) for name in selected)
    return result


def successful_affected_tests(payloads,owner,rows):
    # Receipts preserve declared selector order; libtest prints its native order.
    # Verify the same exact IDs and their immutable row bindings in either order.
    groups={}
    for row in rows:groups.setdefault(row['stdout']['sha256'],[]).append(row)
    ordered=[]
    for group in groups.values():
        actual=re.findall(r'^test (.+) \.\.\. ok$',payloads.raw(group[0]['stdout']).decode(),re.M)
        by_id={row['id']:row for row in group}
        require(len(by_id)==len(group)==len(actual) and len(actual)==len(set(actual)) and set(actual)==set(by_id),
            'Exact selected native IDs and stdout IDs differ')
        ordered.extend(by_id[name] for name in actual)
    return successful_tests(payloads,owner,ordered)


# Actual43 controls in a literal StorageRED plus a separate actualIR4 owner.
AOT43_PREFIX_SHA='7737ac97410c69ac324b8809e9f5825c9cbc2efe5b205eb99990a96345fafc43'
IR4_PLAN_SHA='4b12006dd0257aaebae7d7f52263a9ebc262af397f3da2a7d713175c79e339dc'


def verify_affected_aggregate(p,scope,proof):
    require(scope['aggregate_policy']=='43-AOT-in-literal-storage-RED-plus-four-actual-IR-controls',
        'Explicit43-plus4 aggregate policy required')
    prefix_ref=scope['aot_prefix_RED'];require(prefix_ref['sha256']==AOT43_PREFIX_SHA and prefix_ref in proof['preserved_RED_owners'],
        'Actual AOT43 StorageRED must remain preserved')
    aot=p.json(prefix_ref);source_clean(aot,False);plan=p.json(aot['plan'])
    require(aot['plan']==scope['corrected_plan'] and aot['plan']['sha256']=='53dc338488d2863d12f2817b79d7cbc07d28fa7e31bd3288dd222a3a884b6afa' and
        [x['id'] for x in aot['commands']]==['aot_unit','aot_unit-inventory','aot_unit-focused'] and
        all(x['complete'] and x['exit_code']==0 for x in aot['commands']) and
        [x['id'] for x in aot['completed_native']]==['aot_unit'] and len(aot['completed_tests'])==43 and
        all(x['target']=='aot_unit' for x in aot['completed_tests']) and aot['error'].startswith('Storage deferred:'),
        'Exactly43 actual AOT passes and no IR attempt in the failed original owner required')
    ir=p.json(scope['owner']);source_clean(ir,True);ir_plan=p.json(ir['plan'])
    require(ir['plan']==scope['continuation_plan'] and ir['plan']['sha256']==IR4_PLAN_SHA and
        ir_plan['ir4_original47_plan']==aot['plan'] and ir_plan['aot_prefix_RED']==prefix_ref and
        ir_plan['ir4_only'] and ir_plan['AOT_compile_or_rerun_forbidden'] and
        ir['aot_prefix_RED']==prefix_ref and ir['original47_selector_contract']==aot['plan'] and
        ir['adopted_AOT43_count']==43 and not ir['AOT_recompiled'] and not ir['AOT_tests_rerun'] and
        ir['aggregate_actual_controls_passed']==47 and not ir['aggregate_is_a_producer_owner'] and
        ir['original_AOT43_owner_stays_literal_RED'],
        'Actual IR4 suffix must adopt43 explicitly without fabricating a whole47 producer')
    target='ir_prepared_dynamic_function'
    require([x['id'] for x in ir['commands']]==[target,target+'-inventory',target+'-focused'] and
        all(x['complete'] and x['exit_code']==0 for x in ir['commands']) and
        [x['id'] for x in ir['completed_native']]==[target] and len(ir['completed_tests'])==4 and
        all(x['target']==target for x in ir['completed_tests']) and ir['source_before']==aot['source_before'],
        'Only the new IR compile, inventory and four native controls may execute')
    adoption=p.json(ir_plan['adopted_AOT43'])
    require(adoption['original_owner']==prefix_ref and adoption['original_plan']==aot['plan'] and
        adoption['completed_native']==aot['completed_native'] and adoption['completed_tests']==aot['completed_tests'] and
        adoption['original_owner_stays_literal_RED'] and not adoption['AOT_recompiled'] and not adoption['AOT_tests_rerun'],
        'Adopted43 rows must be the exact immutable actual producer rows')
    relocation=p.json(adoption['artifact_relocation'])
    require(adoption['artifact_relocation']['sha256']=='a85f44d7208f9f6157c36d311451b3190c814f810960a7a1c8ad9036e75aba6a',
        'Exact lossless AOT artifact relocation receipt required')
    aot_view=dict(plan,stages=[stage for stage in plan['stages'] if stage['id']=='aot_unit'])
    ir_view=dict(plan,stages=[stage for stage in plan['stages'] if stage['id']==target])
    expected=resolve_affected_controls(p,aot,aot_view)|resolve_affected_controls(p,ir,ir_view)
    actual={(x['target'],x['id']) for x in aot['completed_tests']+ir['completed_tests']}
    old_plan=p.json(p.json(proof['source_successor']['original_AOT_compile_RED'])['plan'])
    required=resolve_affected_controls(p,aot,dict(old_plan,stages=[stage for stage in old_plan['stages'] if stage['id']=='aot_unit']))
    required|=resolve_affected_controls(p,ir,dict(old_plan,stages=[stage for stage in old_plan['stages'] if stage['id']==target]))
    require(len(actual)==len(expected)==47 and actual==expected and len(required)==6 and required<=actual and
        successful_affected_tests(p,aot,aot['completed_tests'])==43 and successful_affected_tests(p,ir,ir['completed_tests'])==4,
        'Exact actual43+4 native controls must retain the original6 selector contracts')
    return aot,plan,actual,expected,required

AOT_IR_AGGREGATE_VERIFY=verify

def verify(proof,root=None):
    result=AOT_IR_AGGREGATE_VERIFY(proof,root)
    if proof['native']['aot_ir_controls']['status']=='passed':
        result['AOT_observed_in_literal_storage_RED']=43
        result['IR_newly_completed']=4
    return result


if __name__ == '__main__': raise SystemExit(main())
