#!/usr/bin/env python3
"""Verify a portable operational checkpoint offline; no native/network execution."""
import argparse,base64,gzip,hashlib,json,re,struct,sys
from collections import Counter
from pathlib import Path,PurePosixPath


HEAD = '8e9a8c8210f99d8d14b4267461ff42079b603c00'

PINS = {'ecma262': '5345883164f463e87f8b40aca4956157ecba8783',
        'test262': '6a4268a9354a545d41c3b62efebc478ed8c521f4'}

SOURCE_PINS = {'ecma262': PINS['ecma262'],
               'test262_upstream': '2e0a56762801e275a9fdf96dc49d90ba0cddcf63',
               'test262_tree': PINS['test262']}

KINDS = ('Parser', 'EarlyError', 'Lowering', 'Runtime', 'WasmBackend', 'HostHarness', 'Unsupported')

OUTCOMES = ('Success', 'NotImplemented', 'Crash', 'Bug')

ORIGINS = ('unknown', 'local-harness', 'boa-runtime', 'boa-parser', 'icu-intl', 'spec-exec-host', 'wasm-backend')

MODES = ('sloppy-script', 'strict-script', 'raw-script', 'module', 'raw-module')

MASK = (1 << 64) - 1

def require(value, message):
    if not value:
        raise ValueError(message)

def unique(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'Duplicate JSON field: ' + key)
        result[key] = value
    return result

def integer(value, label, maximum=None):
    require(type(value) is int and value >= 0 and (maximum is None or value <= maximum),
            'Invalid integer ' + label)
    return value

def exact(actual, expected, label):
    require(actual == expected, label + ' differs')

def execution(value):
    require(isinstance(value, str) and ':' in value, 'Missing execution identity')
    mode, path = value.split(':', 1)
    part = PurePosixPath(path)
    require(mode in MODES and path.endswith('.js') and not re.search(r'\s', value)
            and not part.is_absolute() and '..' not in part.parts and path == part.as_posix(),
            'Invalid execution identity: ' + value)
    return mode, path

def ids(values, label):
    require(isinstance(values, list), 'Expected ID array: ' + label)
    for value in values:
        execution(value)
    require(len(values) == len(set(values)), 'Duplicate execution IDs: ' + label)
    return set(values)

def counts(value, labels, label):
    require(isinstance(value, dict) and set(value) == set(labels), 'Incomplete taxonomy: ' + label)
    for key, count in value.items():
        integer(count, label + '/' + key)
    return value

def complete(counter, labels):
    return {label: counter[label] for label in labels}

def compiler(value):
    exact(set(value), {'source_fingerprint_scheme', 'source_fingerprint', 'source_revision', 'executable_sha256'},
          'Compiler identity fields')
    exact(value['source_fingerprint_scheme'], 'lila-program-cache-compiler-v3', 'Compiler fingerprint scheme')
    exact(value['source_revision'], {'kind': 'git-commit', 'commit': HEAD}, 'Compiler revision')
    for field in ('source_fingerprint', 'executable_sha256'):
        require(isinstance(value[field], str) and re.fullmatch('[0-9a-f]{64}', value[field]), 'Invalid compiler digest')
    return value

def siphash13(data):
    state = [0x736f6d6570736575, 0x646f72616e646f6d, 0x6c7967656e657261, 0x7465646279746573]

    def rotate(x, n):
        return ((x << n) | (x >> (64 - n))) & MASK

    def round_once():
        v0, v1, v2, v3 = state
        v0 = (v0 + v1) & MASK; v1 = rotate(v1, 13); v1 ^= v0; v0 = rotate(v0, 32)
        v2 = (v2 + v3) & MASK; v3 = rotate(v3, 16); v3 ^= v2
        v0 = (v0 + v3) & MASK; v3 = rotate(v3, 21); v3 ^= v0
        v2 = (v2 + v1) & MASK; v1 = rotate(v1, 17); v1 ^= v2; v2 = rotate(v2, 32)
        state[:] = [v0, v1, v2, v3]

    for at in range(0, len(data) // 8 * 8, 8):
        word = int.from_bytes(data[at:at + 8], 'little')
        state[3] ^= word; round_once(); state[0] ^= word
    last = int.from_bytes(data[len(data) // 8 * 8:], 'little') | ((len(data) & 255) << 56)
    state[3] ^= last; round_once(); state[0] ^= last; state[2] ^= 255
    for _ in range(3):
        round_once()
    return state[0] ^ state[1] ^ state[2] ^ state[3]

def hstr(value):
    return value.encode() + b'\xff'

def u64(value):
    return struct.pack('<Q', value)

def hid(value):
    mode, path = execution(value)
    return hstr(path) + u64(MODES.index(mode))

def node_hash(node):
    return siphash13(hstr(PINS['ecma262']) + hstr(PINS['test262']) + u64(1) + hstr(node['node_id'])
                     + b''.join(hid(value) for value in node['case_ids']))

def matrix_hash(nodes):
    data = bytearray(struct.pack('<I', 3) + hstr('wasm-aot'))
    for node in nodes:
        data.extend(hstr(node['node_id']))
        data.extend(u64(('FilterLeaf', 'ChunkLeaf').index(node['node_kind'])))
        data.extend(hstr(node['filter']))
        data.extend(u64(len(node['matrix_path'])))
        for part in node['matrix_path']:
            data.extend(hstr(part))
        data.extend(u64(node['total_cases']))
        data.extend(u64(len(node['case_ids'])))
        for value in node['case_ids']:
            data.extend(hid(value))
    return siphash13(data)

def envelope(value, identity):
    exact(value.get('snapshot_version'), 8, 'Current snapshot version')
    exact(value.get('producer'), 'lila', 'Snapshot producer')
    exact(value.get('matrix_strategy_version'), 3, 'Current matrix strategy')
    exact(value.get('execution_backend'), 'wasm-aot', 'Actual execution backend')
    exact(value.get('pinned_revisions'), PINS, 'Exact current source pins')
    exact(value.get('compiler_identity'), identity, 'Actual compiler binding')
    require('completed_paths' not in value and 'timeout_list' not in value, 'Legacy path-only field in current evidence')
    integer(value['manifest_hash'], 'Manifest hash', MASK)

def audit_case(value, expected_ids, identity):
    envelope(value, identity)
    completed = ids(value['completed_test_ids'], 'completed_test_ids')
    exact(completed, set(expected_ids), 'Exact node executions')
    exact(value['total'], len(completed), 'Case total')
    integer(value['total'], 'Case total'); integer(value['passed'], 'Case passes')
    require(value['passed'] <= value['total'], 'Passed exceeds total')
    require(not value['completed_nodes'] and not value['aggregate_entries'], 'Case contains aggregate-only fields')
    exact(counts(value['aggregate_counts_so_far'], KINDS, 'Case aggregate counts'), dict.fromkeys(KINDS, 0),
          'Case aggregate-only counts')
    failures, outcomes, kinds, origins = {}, Counter({'Success': value['passed']}), Counter(), Counter()
    for row in value['failures']:
        mode, path = execution(row['test_id'])
        require(row['test_id'] in completed and row['test_id'] not in failures, 'Unknown or duplicate failure ID')
        exact(row['test_path'], path, 'Failure physical path')
        require(row['outcome'] in OUTCOMES[1:] and row['kind'] in KINDS and row['origin'] in ORIGINS,
                'Unknown or successful failure classification')
        require(isinstance(row['detail'], str), 'Missing failure detail')
        integer(row['detail_hash'], 'Failure detail hash', MASK)
        if row.get('duration_ms') is not None:
            integer(row['duration_ms'], 'Failure duration')
        failures[row['test_id']] = row
        outcomes[row['outcome']] += 1; kinds[row['kind']] += 1; origins[row['origin']] += 1
    exact(len(failures), value['total'] - value['passed'], 'Literal failure count')
    exact(counts(value['counts_per_kind'], KINDS, 'Case kinds'), complete(kinds, KINDS), 'Case kind projection')
    exact(counts(value['counts_per_outcome'], OUTCOMES, 'Case outcomes'), complete(outcomes, OUTCOMES),
          'Case outcome projection')
    exact(ids(value['timeout_test_ids'], 'Timeout IDs'),
          {key for key, row in failures.items() if 'timeout exceeded' in row['detail']}, 'Literal timeout IDs')
    slow = set()
    for row in value['slowest_tests']:
        require(row['test_id'] in completed and row['test_id'] not in slow, 'Unknown or duplicate slow-test ID')
        exact(row['path'], execution(row['test_id'])[1], 'Slow-test physical path')
        integer(row['duration_ms'], 'Slow-test duration'); slow.add(row['test_id'])
    return {'outcomes': complete(outcomes, OUTCOMES), 'kinds': complete(kinds, KINDS),
            'origins': complete(origins, ORIGINS), 'failures': failures}

def canonical_json(value):
    return json.dumps(value, separators=(',', ':'), ensure_ascii=False)

def render_summary(value):
    out = [f"test262 {value['run_kind']} summary (snapshot v8)", 'producer=lila',
           'compiler_identity=' + canonical_json(value['compiler_identity']), 'execution_backend=wasm-aot',
           'matrix_strategy_version=3', f"pinned: ecma262={PINS['ecma262']} test262={PINS['test262']}",
           f"total={value['total']} pass={value['passed']} fail={value['total'] - value['passed']}"]
    out.extend(f'{key}={value["counts_per_kind"][key]}' for key in KINDS)
    out.extend(f'outcome_{key}={value["counts_per_outcome"][key]}' for key in OUTCOMES)
    if value['timeout_test_ids']:
        out.append('timeouts=' + ', '.join(value['timeout_test_ids']))
    if value['failures']:
        out.append('top_failures:')
        for row in value['failures'][:10]:
            mode, path = execution(row['test_id'])
            out.append(f"- {path} [{mode}] [{row['outcome']}/{row['kind']}/{row['origin']}] {row['detail']}")
    return '\n'.join(out) + '\n'

BASE=Path('/workspace/porffor/target')
ROOT=Path('/workspace/porffor-next')

def selected(messages, head):
    require([m['success'] for m in messages if m.get('reason') == 'build-finished'] == [True],
            'No unique successful actual Cargo build')
    artifacts = [m for m in messages if m.get('reason') == 'compiler-artifact']
    require(not any(m['profile']['test'] or m['target']['name'] == 'boa_engine'
                    or 'spec-exec-oracle' in m.get('features', []) for m in artifacts),
            'A test or oracle graph cannot own the ordinary CLI')
    bins = [m for m in artifacts if m['target']['name'] == 'lila' and m['target']['kind'] == ['bin']]
    require(len(bins) == 1, 'Actual ordinary CLI artifact missing or ambiguous')
    artifact = bins[0]
    require(artifact['executable'] == str(BASE / 'release/lila')
            and artifact['target']['src_path'] == str(ROOT / 'crates/lila-cli/src/bin/lila.rs')
            and artifact['profile']['opt_level'] == '3' and artifact['features'] == [],
            'Require the actual optimized default official CLI entry')
    engines = [m for m in messages if m.get('reason') == 'build-script-executed'
               and '/crates/lila-engine#' in m.get('package_id', '')]
    runtime = [m for m in messages if m.get('reason') == 'build-script-executed'
               and '/crates/lila-runtime-image#' in m.get('package_id', '')]
    require(len(engines) == len(runtime) == 1, 'Actual Engine/runtime producer missing or ambiguous')
    identity = dict(engines[0]['env'])
    require(identity['LILA_COMPILER_SOURCE_REVISION'] == 'git:' + head,
            'Actual Engine producer has a stale source revision')
    require(Path(runtime[0]['out_dir']).parent.parent == BASE / 'release/build'
            and 'native=' + runtime[0]['out_dir'] in runtime[0]['linked_paths']
            and 'static:-bundle=lila_runtime_images' in runtime[0]['linked_libs'],
            'Runtime producer is outside the ordinary release static-link graph')
    return artifact, engines[0], runtime[0], artifacts


TRANSITION_SHA='801169119ebe673c45542f8971333df0a559104354adc2dc28073ac076ebd0ef'
CONTROLLER_SHA='b76856723adda22e79cb0da62fcff345f943d1e3073b7b3a8e718e90d9c8e9ae'
PREPARATIONS={
 'controller':'7d07ef6fcecdbad514db105338ba254b7ceccb0e50fd26a6740b0fa08563bec5',
 'boundary':'48bb7e3f92c1ec2f056082f6cf6efcae0eea11980c0cea553d4dca429bb37214',
 'auditor_v2':'c01816100c18a5013f5458d349f3e1d818b7e5e8882182bf3290022d4ab61736',
 'watcher':'ec429de85d7ff8985fd7ac73a94fdf85b572f174020259acef47474ad546c02d'}
PUBLICATION_ENVIRONMENT=frozenset(('REPO_ROOT','LILA_BIN','SUITE_ROOT','SNAPSHOT_DIR','SNAPSHOT_NAME','README_PATH',
 'THREADS','JOBS','ISOLATE_CASES','REUSE_CASE_WORKERS','MAX_MATRIX_NODES','CARGO_TARGET_DIR','CARGO_BUILD_JOBS',
 'RUST_TEST_THREADS','LILA_JOBS','RAYON_NUM_THREADS','LILA_CPU_PERCENT','LILA_CACHE_DIR','LILA_MODULE_MEMORY_CACHE_LIMIT_BYTES'))
RUNTIME_ENVIRONMENT=frozenset(('TZ','LANG','LC_ALL','LC_TIME','RUST_MIN_STACK','LILA_MODULE_MEMORY_CACHE_ENTRIES',
 'LILA_CACHE_LIMIT_BYTES','LILA_FUNCTION_CACHE_LIMIT_BYTES','LILA_MODULE_CACHE_LIMIT_BYTES','LILA_PROGRAM_CACHE_LIMIT_BYTES'))


def canonical(value):
    return json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=True).encode()


class Evidence:
    def __init__(self,value):
        self.value=value;self.files={};total=0
        for path,row in value['files'].items():
            require(path.startswith(('/workspace/','/tmp/')) and '..' not in PurePosixPath(path).parts,'Invalid original evidence path')
            exact(row['codec'],'gzip-base64','Portable file codec')
            packed=base64.b64decode(row['data'],validate=True)
            raw=gzip.decompress(packed)
            integer(row['bytes'],'Embedded file bytes',32*1024**2)
            exact(len(raw),row['bytes'],'Embedded file size')
            exact(hashlib.sha256(raw).hexdigest(),row['sha256'],'Embedded file digest')
            require(not raw.startswith(b'\x7fELF'),'Native executable must not be embedded')
            total+=len(raw);require(total<=96*1024**2,'Portable operational evidence exceeds its bounded size')
            self.files[path]=raw

    def raw(self,row):
        require(isinstance(row,dict) and 'path' in row and 'sha256' in row,'Missing bound evidence reference')
        require(row['path'] in self.files,'A required evidence file is absent from the portable checkpoint')
        raw=self.files[row['path']]
        exact(hashlib.sha256(raw).hexdigest(),row['sha256'],'Bound evidence SHA')
        if 'bytes' in row:exact(len(raw),row['bytes'],'Bound evidence size')
        return raw

    def doc(self,row):
        value=json.loads(self.raw(row),object_pairs_hook=unique)
        require(isinstance(value,dict),'Expected bound JSON object')
        return value

    def path(self,path,digest=None):
        require(path in self.files,'Original evidence path was not exported')
        row={'path':path,'sha256':digest or hashlib.sha256(self.files[path]).hexdigest()}
        return self.raw(row)


def native_command(env,operation):
    result=[env['LILA_BIN'],'test262',operation]
    if operation in ('report-all','publish-status'):result+=['--jobs',env['JOBS']]
    result+=['--execution-backend','wasm-aot','--suite-root',env['SUITE_ROOT'],'--snapshot-dir',env['SNAPSHOT_DIR'],
             '--snapshot-name',env['SNAPSHOT_NAME']]
    if operation=='report-all':result+=['--resume','--threads',env['THREADS'],'--max-matrix-nodes','1','--reuse-case-workers']
    elif operation=='publish-status':result+=['--readme-path',env['README_PATH']]
    else:exact(operation,'progress-status','Native operation')
    return result


def progress(text,identity):
    found={}
    for line in text.splitlines():
        key,separator,value=line.partition(': ')
        if key in ('matrix_nodes_completed','matrix_nodes_total','compiler_identity'):
            require(separator and key not in found,'Duplicate or malformed native progress field');found[key]=value
    exact(set(found),{'matrix_nodes_completed','matrix_nodes_total','compiler_identity'},'Native progress fields')
    exact(json.loads(found['compiler_identity'],object_pairs_hook=unique),identity,'Raw progress compiler identity')
    for key in ('matrix_nodes_completed','matrix_nodes_total'):
        require(re.fullmatch(r'0|[1-9][0-9]{0,17}',found[key]),'Invalid raw progress count')
    result={'completed':int(found['matrix_nodes_completed']),'total':int(found['matrix_nodes_total'])}
    require(0<=result['completed']<=result['total'] and result['total']==748,'Changed original matrix denominator')
    return result


def check_preparations(e):
    for role,expected in PREPARATIONS.items():
        ref=e.value['preparations'][role];exact(ref['sha256'],expected,'Reviewed preparation manifest '+role)
        doc=e.doc(ref)
        for row in doc.get('sealed_files',[]):e.raw(row)
        for leaf,row in doc.get('files',{}).items():
            e.raw(dict(row,path=str(PurePosixPath(ref['path']).parent/leaf)))
    return True


def check_normal_compiler(e,owner,identity):
    stage=e.doc(owner['ordinary_build_owner'])
    require(stage['schema']=='ordinary-default-cli-owner-v2' and stage['head']==HEAD
            and all(stage.get(k) is True for k in ('complete','passed','source_unchanged','protected_images_unchanged'))
            and stage['oracle'] is False and not stage.get('controller_error') and not stage.get('final_guard_error'),
            'Actual ordinary Cargo compiler owner did not close')
    exact(stage['frozen_sources_before'],stage['frozen_sources_after'],'Ordinary compiler frozen source')
    exact(stage['frozen_sources_before'],owner['frozen_sources_before'],'Long/normal compiler source identity')
    build=stage['build']
    expected=['python3','-B','scripts/limited_verification.py','--cloud','--cloud-cpus','auto','--','cargo','build',
              '--locked','--offline','--release','-p','lila-cli','--bin','lila','--message-format=json-render-diagnostics','--timings']
    exact(build['command'],expected,'Genuine stock default Cargo command')
    for row in stage['commands']:
        require(row.get('complete') is True and row.get('exit_code')==0,'An original normal build command failed')
        for stream in ('stdout','stderr'):
            e.path(row[stream],row[stream+'_sha256'])
    messages=[json.loads(line,object_pairs_hook=unique) for line in e.path(build['stdout'],build['stdout_sha256']).decode().splitlines() if line.startswith('{')]
    artifact,engine,runtime,artifacts=selected(messages,HEAD)
    exact((artifact,engine,runtime,artifacts),(stage['actual_cli_artifact'],stage['actual_engine_producer'],
          stage['actual_runtime_producer'],stage['compiler_artifacts']),'Actual Cargo-selected producer graph')
    worker=stage['native_cli_worker'];exact(worker['identity'],identity,'Ordinary native identity')
    exact(worker['sha256'],identity['executable_sha256'],'Normal worker executable digest')
    exact(stage['ordinary_cli']['sha256'],worker['sha256'],'Normal bin/worker identical bytes digest')
    exact(json.loads(e.path(stage['worker_identity_run']['stdout']).decode(),object_pairs_hook=unique),identity,'Actual worker identity raw output')
    embedded=dict(engine['env'])
    exact(embedded['LILA_COMPILER_FINGERPRINT'],identity['source_fingerprint'],'Actual source fingerprint producer')
    require(not re.search(r'\(NEEDED\).*\[liblila_runtime_images_',e.path(stage['readelf_run']['stdout']).decode()),
            'A shared test image cannot own the ordinary compiler')
    return worker


def check_original_interruption(e,transition,owner):
    rows=[row for row in owner['commands'] if row['label']=='fullpin102994'];exact(len(rows),1,'Original full publisher row count')
    row=rows[0];require(row['complete'] is True and type(row['exit_code']) is int and row['exit_code']==143,
                       'Original full publisher must remain literal interrupted143')
    exact(row['command'],transition['original_publisher_command'],'Original publisher argv')
    exact(row['command'],owner['plan']['commands']['fullpin102994'],'Original sealed publisher plan')
    for stream in ('stdout','stderr'):e.raw(row[stream])
    outer=e.doc(transition['original_publisher_outer'])
    require(outer['complete'] is True and outer['source_unchanged'] is True and outer['exit_code']==143 and outer['head']==HEAD,
            'Actual original outer interruption/source closure missing')
    exact(outer['command'],row['command'][6:],'Original outer payload argv')
    exact(outer['source_before'],outer['source_after'],'Actual original outer source fence')
    for stream in ('stdout','stderr'):
        archive=outer[stream];packed=e.path(archive['path'],archive['archive_sha256']);raw=gzip.decompress(packed)
        exact(hashlib.sha256(raw).hexdigest(),archive['raw_sha256'],'Original interrupted raw stream digest')
        exact(len(raw),archive['raw_bytes'],'Original interrupted raw stream length')
    census=e.doc(transition['process_census'])
    for key in ('live_original_descendants','history_signals_sent','family_live_processes'):exact(census[key],[],'Root terminal census '+key)
    known={(x['pid'],x['start_ticks']):x for x in [*census['original_roles'].values(),*census['drain_observed_processes']]}
    long=census['original_roles']['long_owner'];expected=set(known)-{(long['pid'],long['start_ticks'])};seen=set()
    for terminal in census['terminal_observations']:
        old=terminal['identity'];key=(old['pid'],old['start_ticks']);require(key in expected and key not in seen,'Unknown/repeated terminal original process')
        exact(old,known[key],'Original terminal process identity');require(terminal['same_identity_live'] is False,'Original publisher was still live')
        current=terminal['current'];require(current is None or current['start_ticks']!=old['start_ticks'] or current['state'] in ('Z','X'),
                                           'A live original process was not drained');seen.add(key)
    exact(seen,expected,'All original descendants terminal')
    return outer


def check_closed_prefix(e,transition,identity):
    audit=e.doc(transition['artifact_audit']);aggregate=e.doc(transition['initial_aggregate_copy'])
    initial=e.doc(transition['initial_manifest_copy']);exact(initial['identity'],identity,'Initial full-byte-bound family identity')
    exact(initial['progress'],{'completed':28,'total':748},'Actual closed original prefix progress')
    exact(transition['progress'],initial['progress'],'Root transition exact original prefix')
    cache=e.doc(audit['matrix_cache']);nodes=cache['nodes'];exact(len(nodes),748,'Full pinned matrix node denominator')
    union=set();by_id={}
    for node in nodes:
        selected_ids=ids(node['case_ids'],'Full native matrix partition');require(not union.intersection(selected_ids),'Duplicated full matrix execution ID')
        exact(node['total_cases'],len(selected_ids),'Full native partition denominator');union.update(selected_ids);by_id[node['node_id']]=node
    exact(len(by_id),748,'Full unique matrix node identities');exact(len(union),102994,'Full pinned execution denominator')
    exact(len({execution(x)[1] for x in union}),53616,'Full pinned physical file denominator')
    inventory=e.raw(e.value['native_inventory']).decode().splitlines();exact(inventory[0],'count: 102994','Actual normal native inventory count')
    exact(ids(inventory[1:],'Actual normal native inventory'),union,'Matrix and actual normal native inventory')
    envelope(aggregate,identity['compiler_identity']);exact(aggregate['manifest_hash'],matrix_hash(nodes),'Exact actual full matrix hash')
    exact(aggregate['run_kind'],'aggregate-matrix','Initial aggregate run kind');exact(aggregate['completed_test_ids'],[],'Initial aggregate case-only IDs')
    exact(aggregate['failures'],[],'Initial aggregate case-only failures');exact(len(aggregate['aggregate_entries']),28,'Closed initial node denominator')
    exact(aggregate['completed_nodes'],[row['node_id'] for row in aggregate['aggregate_entries']],'Initial aggregate completed node identities')
    prior_refs={r['path']:r for r in transition['previously_closed_snapshots']};exact(len(prior_refs),84,'Every original JSON/TXT/timing ref')
    total=0;positive=set();negative={};sums={'outcomes':Counter(),'kinds':Counter(),'origins':Counter()}
    for entry in aggregate['aggregate_entries']:
        node=by_id[entry['node_id']];nh=node_hash(node);exact(nh,entry['manifest_hash'],'Recomputed original node identity')
        path=identity['snapshot_directory']+'/'+identity['snapshot_name']+'-'+node['node_id'].replace('/','_')+'-'+str(nh)+'.json'
        value=e.doc(prior_refs[path]);result=audit_case(value,node['case_ids'],identity['compiler_identity'])
        require('checkpoint_identity' not in value,'An original node was not closed')
        exact(value['run_kind'],'matrix-'+{'FilterLeaf':'filter-leaf','ChunkLeaf':'chunk-leaf'}[node['node_kind']],'Closed original node run kind')
        exact(value['manifest_hash'],nh,'Original raw node manifest hash')
        for field in ('filter','matrix_path'):exact(entry[field],node[field],'Original entry/node '+field)
        for field,expected in [('total',value['total']),('passed',value['passed']),('failed',len(result['failures']))]:exact(entry[field],expected,'Original literal node '+field)
        for key,wire in [('outcomes','counts_per_outcome'),('kinds','counts_per_kind'),('origins','counts_per_origin')]:
            exact(entry[wire],result[key],'Original raw taxonomy projection');sums[key].update(result[key])
        exact(e.raw(prior_refs[path[:-5]+'.txt']).decode(),render_summary(value),'Original node JSON/TXT projection')
        timing=e.doc(prior_refs[path[:-5]+'.timing']);exact(timing['snapshot_sha256'],prior_refs[path]['sha256'],'Original exact timing snapshot')
        exact(timing['compiler'],identity['compiler_identity'],'Original timing compiler identity')
        for key,expected in [('timeout_ms',60000),('configured_workers',3),('reuse_case_workers',True)]:exact(timing['settings'][key],expected,'Original native resource policy '+key)
        selected_ids=set(value['completed_test_ids']);positive.update(selected_ids-set(result['failures']));negative.update(result['failures']);total+=value['total']
    exact(aggregate['total'],total,'Actual original prefix execution count');exact(aggregate['passed'],len(positive),'Actual original prefix successes')
    exact(aggregate['counts_per_outcome'],complete(sums['outcomes'],OUTCOMES),'Actual original prefix literal outcomes')
    exact(aggregate['counts_per_kind'],complete(sums['kinds'],KINDS),'Actual original prefix failure kinds')
    exact(e.raw(transition['prior_success_ids']).decode(),''.join(x+'\n' for x in sorted(positive)),'Every exact original positive execution ID')
    return {'nodes':28,'executions':total,'counts_per_outcome':complete(sums['outcomes'],OUTCOMES),
            'literal_nonpasses':[negative[x] for x in sorted(negative)],'positive_execution_ids':len(positive)}


def check_active_resume(e,transition,owner,identity):
    continuation=e.doc(e.value['active_continuation']);exact(continuation['schema'],'guarded-publication-continuation-v1','Actual active controller schema')
    require(continuation['complete'] is False and continuation['native_acceptance'] is False and not continuation.get('error'),
            'This operational snapshot must retain actual active partial status')
    exact(continuation['initial_identity'],identity,'Actual successful full startup identity capture')
    exact(continuation['initial_progress'],transition['progress'],'Same original resumed prefix')
    exact(continuation['transition']['sha256'],TRANSITION_SHA,'Actual same-family transition')
    exact(continuation['driver']['sha256'],CONTROLLER_SHA,'Actual reviewed running continuation')
    exact(continuation['previously_closed_snapshots'],transition['previously_closed_snapshots'],'Every original frozen node ref remains')
    projection=e.doc(continuation['effective_environment']);exact(set(projection),PUBLICATION_ENVIRONMENT|RUNTIME_ENVIRONMENT,'Only nonsecret policy fields may be persisted')
    exact(continuation['environment_projection'],'explicit-nonsecret-publication-resource-and-runtime-fields-v1','Explicit nonsecret projection schema')
    for key,value in owner['plan']['publisher_environment'].items():exact(projection[key],value,'Exact original native payload '+key)
    for key,value in identity['environment'].items():exact(projection[key],value,'Exact original observed runtime '+key)
    exact(projection['REPO_ROOT'],identity['repository'],'Native original repository');exact(projection['SNAPSHOT_NAME'],identity['snapshot_name'],'Native same matrix family')
    observer=e.doc(transition['progress_observer']);exact(observer['environment_projection'],projection,'Boundary/resume exact nonsecret policy')
    exact(progress(e.raw(observer['stdout']).decode(),identity['compiler_identity']),transition['progress'],'Actual root stock boundary observation')
    require(observer['exit_code']==0 and observer['after_report'] is True,'Actual root boundary native observation did not pass')
    manifest_before=e.doc(observer['manifest_before']['immutable_copy']);manifest_after=e.doc(observer['manifest_after']['immutable_copy'])
    exact(manifest_after['identity'],identity,'Observed same-family identity');exact(manifest_after['progress'],transition['progress'],'Observed original high-water mark')
    exact(manifest_before['progress']['completed']+1,manifest_after['progress']['completed'],'Actual completed original last node before interruption')
    rows=continuation['commands'];require(rows and all(row.get('complete') is True and row.get('exit_code')==0 for row in rows),
                                     'Snapshot must end at a genuinely completed native command boundary')
    expected_operations=['progress-status']+['report-all','progress-status']*((len(rows)-1)//2)
    exact([row['operation'] for row in rows],expected_operations,'Actual active stock native command prefix')
    digest=hashlib.sha256(canonical(projection)).hexdigest()
    for index,row in enumerate(rows):
        exact(row['sequence'],index,'Actual native sequence');exact(row['command'],native_command(projection,row['operation']),'Unchanged actual native argv')
        exact(row['environment_sha256'],digest,'Native exact nonsecret policy digest')
        for stream in ('stdout','stderr'):e.raw(row[stream])
    observations=continuation['observations'];exact(len(observations),(len(rows)+1)//2,'Actual native progress observation coverage')
    for index,observation in enumerate(observations):
        exact(observation['command_sequence'],index*2,'Progress raw command binding')
        require(observation['after_report'] is (index>0),'Actual after-report progress classification')
        expected={'completed':28+index,'total':748};exact(observation['progress'],expected,'Actual one-node resume advancement')
        exact(progress(e.raw(rows[index*2]['stdout']).decode(),identity['compiler_identity']),expected,'Actual native progress raw stdout')
    exact(continuation['final_progress'],observations[-1]['progress'],'Snapshot last observed progress')
    require(continuation['final_progress']['completed']>=29,'At least one actual resumed node must have finished')
    require(continuation['full_captures'] and continuation['full_captures'][0]['label']=='startup','Successful full startup byte capture missing')
    for capture in continuation['full_captures']:exact(capture['identity'],identity,'Actual full capture identity')
    watcher=e.doc(e.value['active_watcher']);exact(watcher['controller_pid'],continuation['pid'],'Actual watcher controller ownership')
    exact(watcher['controller_receipt'],e.value['active_continuation_original_path'],'Actual watcher controller receipt')
    exact(watcher['reviewed_controller_sha256'],CONTROLLER_SHA,'Actual watcher reviewed driver')
    exact(watcher['transition']['sha256'],TRANSITION_SHA,'Actual watcher same boundary')
    exact(watcher['policy_projection'],continuation['effective_environment'],'Actual watcher same nonsecret policy')
    for key,expected in [('worker_threads',1),('native_jobs',1),('case_workers',3),('timeout_ms',60000),('full_pin_cpus',[0,1,2]),
                         ('source_or_environment_changed',False),('cases_restarted',False)]:exact(watcher[key],expected,'Actual watcher policy '+key)
    require(not watcher.get('error'),'Actual watcher reported an error')
    for change in watcher['changes']:require(change['cpus'] and set(change['cpus'])<={0,1,2},'Watcher affinity exceeded original native CPUs')
    return {'controller_pid':continuation['pid'],'controller_active_at_capture':True,'observed_progress':continuation['final_progress'],
            'completed_native_commands':len(rows),'startup_full_capture_seconds':continuation['full_captures'][0]['seconds'],
            'raw_next_node_cases_verified':False,'watcher_pid':watcher['pid']}


def verify(value):
    exact(value['schema'],'actual-portable-operational-publication-handoff-8e-v1','Operational checkpoint schema')
    exact(value['head'],HEAD,'Frozen source HEAD')
    for key in ('full_pin_completed','full_pin_all_success','historical_replay_completed','task_acceptance',
                'controlled_speed_gain_claimed','canonical_status_updated','native_executables_embedded'):
        require(value['claims'][key] is False,'A partial operational checkpoint cannot claim completed-suite/task/performance acceptance')
    e=Evidence(value);check_preparations(e)
    exact(value['transition']['sha256'],TRANSITION_SHA,'Actual reviewed root transition SHA')
    transition=e.doc(value['transition']);identity=transition['identity'];compiler(identity['compiler_identity'])
    require(transition['boundary_closed'] is True and transition['no_in_flight_node'] is True,'Actual original boundary was not closed')
    owner=e.doc(transition['original_owner']);require(owner['complete'] is False,'Original historical owner must remain active in this snapshot')
    exact(owner['head'],HEAD,'Original active long source')
    worker=check_normal_compiler(e,owner,identity['compiler_identity']);exact(worker['path'],identity['executable'],'Original actual normal worker path')
    check_original_interruption(e,transition,owner)
    prefix=check_closed_prefix(e,transition,identity)
    resumed=check_active_resume(e,transition,owner,identity)
    return {'evidence_verified':True,'head':HEAD,'original_publisher_literal_exit_code':143,
            'verified_closed_original_prefix':prefix,'actual_active_resume':resumed,
            'normal_executable_bytes_embedded':False,'normal_executable_presence_verified_offline':False,
            'full_suite_acceptance':False,'task_acceptance':False,'controlled_speed_gain_claimed':False}


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('checkpoint',type=Path)
    args=parser.parse_args();value=json.loads(args.checkpoint.read_text(),object_pairs_hook=unique)
    exact(hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),value['reader_sha256'],'Exact standalone reader bytes')
    report=verify(value)
    print(json.dumps({key:data for key,data in report.items() if key!='verified_closed_original_prefix'}|{
        'verified_closed_original_prefix':{key:data for key,data in report['verified_closed_original_prefix'].items() if key!='literal_nonpasses'}},indent=2))


if __name__=='__main__':
    try:main()
    except (OSError,ValueError,KeyError,TypeError,AttributeError) as error:
        print('Operational checkpoint verification failed: '+str(error),file=sys.stderr);raise SystemExit(2)
