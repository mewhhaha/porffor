"""Run complete Cargo-selected native scopes with source-bound exact receipts.

Run from the repository root under limited_verification.py --cloud. Cargo stays
serial; up to four native targets use one CPU and one test thread each. Original
worker/campaign deadlines, publication, performance and rustdoc scopes remain
idle-exclusive. --partition-cli inventories and executes every qualified CLI
test exactly once across disjoint processes.
"""
import argparse,hashlib,json,os,re,shutil,subprocess,sys,time,gzip
from pathlib import Path
root=Path.cwd();parser=argparse.ArgumentParser();parser.add_argument('--packages',nargs='*');parser.add_argument('--output',required=True);parser.add_argument('--all-features',action='store_true');parser.add_argument('--include-ignored',action='store_true');parser.add_argument('--test-processes',type=int,default=1);parser.add_argument('--shared-runtime-linker',type=Path);parser.add_argument('--strip-test-debug',action='store_true');parser.add_argument('--no-test-incremental',action='store_true');parser.add_argument('--retry-from',type=Path);parser.add_argument('--reuse-library-build-from',type=Path);parser.add_argument('--restored-library-provenance',type=Path);parser.add_argument('--retry-interrupted',action='store_true');parser.add_argument('--only-targets',nargs='*');parser.add_argument('--native-filter',action='append',default=[]);parser.add_argument('--stop-on-build-failure',action='store_true');parser.add_argument('--partition-cli',action='store_true');parser.add_argument('--continuation-phase',choices=['parallel','barriers','all'],default='all');args=parser.parse_args()
native_filters=dict(row.split('=',1) for row in args.native_filter)
if args.partition_cli:
    assert args.include_ignored, 'Complete CLI partitions require --include-ignored'
    assert 'lila-cli:cli' not in native_filters, 'Complete CLI partitions cannot use a native filter'
out=root/args.output;out.mkdir(parents=True,exist_ok=False)
def source_map():
    # HEAD names the entire committed tree, while a binary patch binds every
    # tracked working-tree change (including fixtures, locks and documentation).
    # Untracked source files are included except ordinary gitignored build output.
    untracked=subprocess.check_output(['git','ls-files','--others','--exclude-standard','-z']).split(b'\0')
    return {'head':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),
            'tracked_patch_sha256':hashlib.sha256(subprocess.check_output(['git','diff','--binary','HEAD','--'])).hexdigest(),
            'untracked':{os.fsdecode(p):hashlib.sha256((root/os.fsdecode(p)).read_bytes()).hexdigest() for p in untracked if p}}
source=source_map();(out/'source-start.json').write_text(json.dumps(source,sort_keys=True)+'\n')
metadata=json.loads(subprocess.check_output(['cargo','metadata','--format-version','1','--no-deps','--locked','--offline'],text=True));(out/'metadata.json').write_text(json.dumps(metadata)+'\n')
package_ids={p['name']:p['id'] for p in metadata['packages']}
members=set(metadata['workspace_members']);packages=[p for p in metadata['packages'] if p['id'] in members]
priority=['lila-front','lila-ir','lila-aot-wasm','lila-engine','lila-cli']
packages.sort(key=lambda p: (priority.index(p['name']) if p['name'] in priority else len(priority),p['name']))
if args.packages:
    assert set(args.packages)<={p['name'] for p in packages}
    packages=[p for p in packages if p['name'] in args.packages]
cases=[];disabled_targets=[]
for p in packages:
    enabled=set(p['features']) if args.all_features else set()
    pending=['default'] if not args.all_features else []
    while pending:
        feature=pending.pop()
        if feature not in enabled and feature in p['features']:
            enabled.add(feature);pending.extend(p['features'][feature])
    for t in p['targets']:
        kind=t['kind'][0]
        missing=set(t.get('required-features',[]))-enabled
        if missing:
            disabled_targets.append({'package':p['name'],'target':t['name'],'missing_required_features':sorted(missing)})
            continue
        if t.get('test') and kind in ['lib','bin','test','bench','example']:
            selector=['--lib'] if kind=='lib' else ['--'+kind,t['name']]
            cases.append({'package':p['name'],'target':t['name'],'kind':kind,'selector':selector})
    if any(t.get('doctest') and 'lib' in t['kind'] for t in p['targets']):cases.append({'package':p['name'],'target':'doc','kind':'doc','selector':['--doc']})
# Front-load the repaired cache and original-budget regressions before the
# remaining inventory. This changes scheduling, never native case selection.
first=[('lila-aot-wasm','lila_aot_wasm','lib'),
       ('lila-test262','differential_worker_execution','test'),
       ('lila-test262','differential_generated_campaign','test'),
       ('lila-test262','lila_test262','lib'),
       ('lila-test262','differential_corpus','test')]
def order(case):
    key=(case['package'],case['target'],case['kind'])
    return first.index(key) if key in first else len(first)
cases.sort(key=order)
def is_barrier(case):
    return case['kind']=='doc' or (case['package']=='lila-cli' and case['target'] in ('perf','fake_suite_publication','differential_corpus','differential_robustness','differential_scenario_pair','differential_worker')) or (case['package']=='lila-test262' and case['target'] in ('differential_worker_execution','differential_generated_campaign','differential_corpus','differential_robustness','differential_rooted_snapshot_execution'))
full_inventory=list(cases)
assert set(native_filters) <= {c['package']+':'+c['target'] for c in full_inventory}
if args.only_targets:
    requested=set(args.only_targets)
    known={c['package']+':'+c['target'] for c in full_inventory}
    assert requested <= known, requested-known
    cases=[c for c in cases if c['package']+':'+c['target'] in requested]

result={'verification_driver_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'source_map_sha256':hashlib.sha256(json.dumps(source,sort_keys=True).encode()).hexdigest(),'head':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'all_features':args.all_features,'include_ignored':args.include_ignored,'cases':cases,'targets_disabled_by_cargo_features':disabled_targets,'selected_workspace_packages':[p['name'] for p in packages]}
if args.retry_from:
    assert args.continuation_phase in ('parallel','barriers')
    previous=json.loads(args.retry_from.read_text())
    assert previous['head']==result['head'] and previous['source_unchanged']
    assert previous['all_features']==args.all_features and previous['include_ignored']==args.include_ignored
    assert previous['selected_workspace_packages']==result['selected_workspace_packages']
    previous_source=json.loads((args.retry_from.parent/'source-start.json').read_text())
    assert previous_source==source
    rejected={(c['package'],c['target'],c['kind']):c for c in previous['cases'] if c.get('state') is None or (args.retry_interrupted and c.get('state') in ('interrupted','storage-deferred'))}
    selected=[]
    for case in cases:
        key=(case['package'],case['target'],case['kind'])
        if key in rejected:
            assert case['selector']==rejected[key]['selector']
            selected.append(case)
    selected=[c for c in selected if is_barrier(c) == (args.continuation_phase=='barriers')]
    assert selected
    cases[:]=selected
    result['continuation_phase']=args.continuation_phase
    result['retry_input']={'path':str(args.retry_from),'sha256':hashlib.sha256(args.retry_from.read_bytes()).hexdigest(),'policy':'exact previously unstarted, explicitly interrupted or storage-deferred whole targets from immutable same-source input; parallel excludes docs/timing; barriers externally idle; original outcomes preserved'}
build_env=os.environ.copy()
if args.no_test_incremental:build_env['LILA_TEST_NO_INCREMENTAL']='1'
if args.shared_runtime_linker:
    linker=args.shared_runtime_linker.resolve()
    assert linker.name=='cc' and linker.is_file()
    build_env['PATH']=str(linker.parent)+os.pathsep+build_env['PATH']
    if args.strip_test_debug:build_env['LILA_TEST_STRIP_DEBUG']='1'
    result['shared_runtime_linker']={'files':{str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [linker,linker.with_name('shared_image.py'),linker.with_name('strip_debug.py')]},'policy':'only Rust libtest ELF outputs in target/debug/deps share immutable data; ordinary product and doc links retain static archive'}
if args.no_test_incremental:
    wrapper=(root/'scripts/test-shared-runtime-images/test-rustc').resolve()
    assert not build_env.get('RUSTC_WRAPPER'), 'Cannot replace an existing compiler wrapper'
    build_env['RUSTC_WRAPPER']=str(wrapper)
    result['test_incremental_policy']={'wrapper':str(wrapper.relative_to(root)),'sha256':hashlib.sha256(wrapper.read_bytes()).hexdigest(),'policy':'Verification compiler units omit incremental scratch; compiled normal libraries remain reusable, queries stay literal and compiler semantics, profiles and verdicts remain intact.'}

def release_incremental(case):
    prefix=case['target'].replace('-','_')+'-'
    removed=[]
    for cache in (root/'target/debug/incremental').glob(prefix+'*'):
        if cache.is_dir():
            # Library and library-test incremental namespaces overlap. Preserve
            # them rather than guessing current Cargo graph identities.
            if case['kind']=='lib':
                continue
            shutil.rmtree(cache);removed.append(cache.name)
    case['released_incremental']=removed

def save():
    result['source_unchanged']=source==source_map();(out/'receipt.json').write_text(json.dumps(result,indent=2)+'\n')
save()

# Independent complete native targets can run concurrently, each under one CPU
# and one libtest thread. Cargo only builds when a native CPU slot is free.
import fcntl
available=sorted(os.sched_getaffinity(0))
assert 1 <= args.test_processes <= min(4 if args.strip_test_debug else 2,len(available))
assert not args.strip_test_debug or args.shared_runtime_linker
slots=available[:args.test_processes]
result['artifact_selection_policy']='Exact Cargo package identity, target name, kind and test profile; every selector uses one stable workspace feature graph; same-name package targets and the library cohort build once.'
result['scheduling']={'native_cpus':slots,'test_processes':args.test_processes,
 'test_threads_per_process':1,'cargo_jobs':1,'strip_test_debug':args.strip_test_debug,
 'compilation_policy':'all inherited parent CPUs when no native target is active; a single free native CPU otherwise'}
active={}
sys.path.insert(0,str(root/'scripts'))
from workspace_test_artifacts import package_selection, selector_key, test_artifacts
artifact_cohorts={}
build_groups={}
result['build_groups']=build_groups

# A successful cold library cohort may have unexecuted siblings after a disk
# deferral. Carry only those exact outputs, from this unchanged source, without
# asking Cargo to regenerate already-retired unrelated libraries.
if args.reuse_library_build_from:
    carry_path=args.reuse_library_build_from
    carry=json.loads(carry_path.read_text())
    assert carry['source_unchanged'] and carry['head']==result['head']
    assert carry['all_features']==args.all_features and carry['include_ignored']==args.include_ignored
    assert carry['selected_workspace_packages']==result['selected_workspace_packages']
    assert json.loads((carry_path.parent/'source-start.json').read_text())==source
    library_group=carry['build_groups']['lib:*']
    archive=next(row for row in library_group['sealed_build_logs'] if row['path'].endswith('.build.jsonl.gz'))
    packed=root/archive['path']
    assert hashlib.sha256(packed.read_bytes()).hexdigest()==archive['gzip_sha256']
    raw=gzip.decompress(packed.read_bytes())
    assert hashlib.sha256(raw).hexdigest()==archive['uncompressed_sha256']
    messages=[json.loads(line) for line in raw.splitlines()]
    assert any(m.get('reason')=='build-finished' and m.get('success') is True for m in messages)
    expected=[(package_ids[c['package']],c['target'],c['kind']) for c in full_inventory if c['kind']=='lib']
    outputs=test_artifacts(messages,expected)
    wanted={(package_ids[c['package']],c['target'],c['kind']) for c in cases if c['kind']=='lib'}
    admitted={}
    evidence=[]
    restored_rows={}
    if args.restored_library_provenance:
        restoration=json.loads(args.restored_library_provenance.read_text())
        archive_path=root/restoration['archive']
        assert hashlib.sha256(archive_path.read_bytes()).hexdigest()==restoration['archive_sha256']
        restored_rows={row['path']:row for row in restoration['rows']}
        result['restored_library_provenance']={'path':str(args.restored_library_provenance),'sha256':hashlib.sha256(args.restored_library_provenance.read_bytes()).hexdigest(),'policy':'Only byte-identical restored ELF outputs subsequently selected as fresh by normal Cargo in the current-source completed library build. Cargo freshness metadata was never changed; all native cases execute anew.'}
    for key in sorted(wanted):
        path=outputs[key]
        matching=[m for m in messages if m.get('reason')=='compiler-artifact' and m.get('executable')==str(path) and m.get('profile',{}).get('test')]
        assert len(matching)==1
        if matching[0]['fresh']:
            original=restored_rows[str(path)]
            assert original['sha256']==hashlib.sha256(path.read_bytes()).hexdigest()
            assert original['bytes']==path.stat().st_size
        assert path.is_file() and path.parent.resolve()==(root/'target/debug/deps').resolve()
        previous_case=next(c for c in carry['cases'] if (package_ids[c['package']],c['target'],c['kind'])==key)
        assert previous_case.get('state') in ('storage-deferred',None), 'Never carry an executed or failed output'
        admitted[key]=path
        evidence.append({'package_id':key[0],'target':key[1],'kind':key[2],'path':str(path.relative_to(root)),'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'bytes':path.stat().st_size,'cargo_fresh':matching[0]['fresh']})
    assert admitted
    cases.sort(key=lambda case: 0 if case['kind']=='lib' else 1)
    result['scheduling']['cached_library_priority']='Run never-executed validated library outputs first; their retirement releases space without another link. Exact whole-target inventory is unchanged.'
    artifact_cohorts[('lib','*')]=admitted
    build_groups['lib:*']={**library_group,'carried_unexecuted_outputs':evidence,'carry_input':{'path':str(carry_path),'sha256':hashlib.sha256(carry_path.read_bytes()).hexdigest(),'policy':'Same exact source and feature graph; successful sealed Cargo stream; only never-executed exact Cargo-selected library siblings; restored fresh outputs additionally checked byte-for-byte against restoration provenance; hashes sealed at this admission.'}}

def build_with_completed_cache_retirement(command, stdout, stderr, case):
    # rustc receives the shared incremental root, not its opaque child cache.
    # Do not guess an active unit's child or delete caches during Cargo.
    case['completed_unit_cache_retirement']=[]
    return subprocess.run(command,stdout=stdout,stderr=stderr,env=build_env).returncode

def seal_build_logs(case, paths):
    archives=[]
    for path in paths:
        if not path.exists():continue
        original_sha=hashlib.sha256(path.read_bytes()).hexdigest()
        packed=path.with_suffix(path.suffix+'.gz')
        with path.open('rb') as src, gzip.open(packed,'wb',compresslevel=1) as dst:shutil.copyfileobj(src,dst)
        archives.append({'path':str(packed.relative_to(root)),'uncompressed_sha256':original_sha,'gzip_sha256':hashlib.sha256(packed.read_bytes()).hexdigest(),'uncompressed_bytes':path.stat().st_size,'compressed_bytes':packed.stat().st_size})
        path.unlink()
    case['sealed_build_logs']=archives
    if case.get('build_group') and archives:build_groups[case['build_group']]['sealed_build_logs']=archives
last_progress_at=time.monotonic()
last_progress_sizes={}

def forward_progress():
    global last_progress_at
    if time.monotonic()-last_progress_at < 20:return
    sizes={entry["label"]:entry["watch"].stat().st_size for entry in active.values() if entry["watch"].exists()}
    changed={label:size for label,size in sizes.items() if last_progress_sizes.get(label)!=size}
    if changed:
        print("PROGRESS",json.dumps(changed,sort_keys=True),flush=True)
        last_progress_sizes.update(changed);last_progress_at=time.monotonic()

def complete(cpu):
    entry=active[cpu];run=entry['run'];status=run.poll()
    if status is None:return False
    entry['stream'].close();case=entry['case'];watch=entry['watch'];log=entry['log']
    transcript=watch.read_text() if watch.exists() else log.read_text()
    matches=re.findall(r'^test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored;',transcript,re.M)
    outcomes=[dict(zip(['passed','failed','ignored'],map(int,row))) for row in matches]
    observed=sum(sum(row.values()) for row in outcomes)
    ignored=sum(row['ignored'] for row in outcomes)
    failed=sum(row['failed'] for row in outcomes)
    matches_inventory=bool(outcomes) and observed==case['inventoried_tests'] and ignored==0 and failed==0
    case.update(state='passed' if status==0 and matches_inventory else 'failed',exit_code=status,
      inventory_matches=matches_inventory,seconds=round(time.monotonic()-entry['start'],3),outcomes=outcomes,
      transcript_log=str(watch.relative_to(root)))
    if watch.exists():case['transcript_sha256']=hashlib.sha256(watch.read_bytes()).hexdigest()
    if entry.get('partition_receipt'):
        partition_path=entry['partition_receipt']
        if partition_path.exists():
            partition_evidence=json.loads(partition_path.read_text())
            case['native_partitions']={'path':str(partition_path.relative_to(root)),'sha256':hashlib.sha256(partition_path.read_bytes()).hexdigest(),'evidence':partition_evidence}
            if case['state']=='passed':
                assert partition_evidence['inventory_matches'] and partition_evidence['complete_test_count']==case['inventoried_tests']
                assert partition_evidence['counts']==outcomes[0] and len(outcomes)==1
                assert partition_evidence['executable_sha256']==case['executable_sha256']
    executable=entry.get('executable')
    if executable is not None:
        assert executable.parent.resolve()==(root/'target/debug/deps').resolve()
        assert hashlib.sha256(executable.read_bytes()).hexdigest()==case['executable_sha256'], 'Native artifact changed during execution'
        executable.unlink();case['executable_released']=True
        # No build is active in this orchestrator. Serialize cache cleanup with
        # any other Cargo producer; running test ELFs do not read incremental.
        with (root/'target/debug/.cargo-lock').open('a+b') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX)
            release_incremental(case)
    del active[cpu];save()
    print('END',entry['label'],case['state'],case['outcomes'],flush=True)
    if not result['source_unchanged']:raise SystemExit('source changed during sweep')
    return True

def reap(wait=False):
    while True:
        changed=False
        for cpu in list(active):changed=complete(cpu) or changed
        if changed or not wait or not active:return
        forward_progress()
        time.sleep(.1)

def split_doc_packages(text):
    starts=list(re.finditer(r'^\s*Doc-tests (\S+)\s*$',text,re.M))
    rows={}
    for index,start in enumerate(starts):
        name=start.group(1)
        assert name not in rows, name
        end=starts[index+1].start() if index+1<len(starts) else len(text)
        rows[name]=text[start.start():end]
    return rows

def run_doc_cohort():
    assert not active, 'Documentation remains idle-exclusive'
    selected=[c for c in cases if c['kind']=='doc']
    assert selected and all(c.get('state') is None for c in selected)
    expected={c['package'].replace('-','_') for c in full_inventory if c['kind']=='doc'}
    flags=['--all-features'] if args.all_features else []
    base=['cargo','test','--locked','--offline',*package_selection(args.packages),*flags,'--doc','--no-fail-fast']
    prefix=['python3','-B','scripts/limited_verification.py','--cloud','--cloud-cpus','auto','--']
    command_inventory=prefix+base+['--','--list']
    inventory_path=out/'workspace-doc-cohort.inventory.log'
    started=time.monotonic()
    with inventory_path.open('w') as stream:
        inventory=subprocess.run(command_inventory,stdout=stream,stderr=subprocess.STDOUT,env=build_env)
    inventory_rows=split_doc_packages(inventory_path.read_text())
    assert inventory.returncode==0 and set(inventory_rows)==expected, (inventory.returncode,set(inventory_rows),expected)
    command=prefix+base+['--','--test-threads=1']+(['--include-ignored'] if args.include_ignored else [])
    label='workspace-'+out.name+'-workspace-doc-cohort'
    log_path=out/'workspace-doc-cohort.execution.log'
    watched=['bash','scripts/run-watched.sh','--label',label,'--stall','2400','--poll','1','--',*command]
    with log_path.open('w') as stream:
        executed=subprocess.run(watched,stdout=stream,stderr=subprocess.STDOUT,env=build_env)
    watch=root/'target/watched'/(label+'.log')
    assert watch.is_file(), watch
    execution_rows=split_doc_packages(watch.read_text())
    assert set(execution_rows)==expected, (set(execution_rows),expected)
    cohort={'inventory_command':command_inventory,'command':command,'exit_code':executed.returncode,
        'inventory_log':str(inventory_path.relative_to(root)), 'inventory_sha256':hashlib.sha256(inventory_path.read_bytes()).hexdigest(),
        'transcript_log':str(watch.relative_to(root)), 'transcript_sha256':hashlib.sha256(watch.read_bytes()).hexdigest(),
        'policy':'One complete workspace documentation cohort on the same all-feature package graph; native rustdoc controls stay serial and idle-exclusive. Completed siblings are recorded separately and never replace prior verdicts.'}
    build_groups['doc:*']=cohort
    for case in selected:
        package=case['package'].replace('-','_')
        label='cohort-'+case['package']+'-doc'
        inventory_piece=out/(label+'.inventory.log')
        inventory_piece.write_text(inventory_rows[package])
        transcript=out/(label+'.native.log')
        transcript.write_text(execution_rows[package])
        outcomes=[dict(zip(['passed','failed','ignored'],map(int,row))) for row in re.findall(r'^test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored;',execution_rows[package],re.M)]
        count=len(re.findall(r'^.+: test$',inventory_rows[package],re.M))
        matches=len(outcomes)==1 and sum(sum(row.values()) for row in outcomes)==count and all(row['ignored']==0 and row['failed']==0 for row in outcomes)
        case.update(state='passed' if matches else 'failed',build_exit=inventory.returncode,
            exit_code=0 if matches else 1,cohort_exit_code=executed.returncode,inventory_matches=matches,inventoried_tests=count,
            inventoried_benchmarks=len(re.findall(r'^.+: benchmark$',inventory_rows[package],re.M)),
            outcomes=outcomes,transcript_log=str(transcript.relative_to(root)),transcript_sha256=hashlib.sha256(transcript.read_bytes()).hexdigest(),
            inventory_command=command_inventory,command=command,build_group='doc:*',seconds=round(time.monotonic()-started,3),
            test_policy='complete workspace docs, one libtest thread per rustdoc; idle-exclusive; cohort wall time is shared')
        print('END',label,case['state'],outcomes,flush=True)
    cohort['completed_siblings']=[{'package':name,'inventory_count':len(re.findall(r'^.+: test$',inventory_rows[name],re.M)),
        'transcript_sha256':hashlib.sha256(execution_rows[name].encode()).hexdigest()} for name in sorted(expected-{c['package'].replace('-','_') for c in selected})]
    save()
    assert result['source_unchanged'], 'Source changed during documentation cohort'

save()
for i,case in enumerate(cases):
    if case['kind']=='doc' and args.continuation_phase in ('barriers','all'):
        if case.get('state') is None:
            # The complete cohort sealed all documentation siblings already.
            # Only new documentation work needs an idle-exclusive barrier.
            while active:reap(wait=True)
            print('START workspace-doc-cohort',flush=True)
            run_doc_cohort()
        continue
    reap()
    partitioned=args.partition_cli and case['package']=='lila-cli' and case['target']=='cli' and case['kind']=='test'
    barrier=is_barrier(case) or partitioned
    if barrier:
        while active:reap(wait=True)
    while len(active)>=len(slots):reap(wait=True)
    cpu=next(cpu for cpu in slots if cpu not in active)
    label=f'{i:04d}-{case["package"]}-{case["target"]}'
    watch_label=f'workspace-{out.name}-{label}'
    print('START',label,flush=True)
    flags=['--all-features'] if args.all_features else []
    selection=['-p',case['package']] if case['kind']=='doc' else package_selection(args.packages)
    base=['cargo','test','--locked','--offline',*selection,*flags,*case['selector']]
    # Raw link output exists before optional DWARF stripping. Fail admission
    # before Cargo if there is insufficient temporary space for that link.
    free_bytes=shutil.disk_usage(root/'target').free
    case['build_start_free_disk_bytes']=free_bytes
    cached_native=case['kind']!='doc' and selector_key(case) in artifact_cohorts
    build_reserve=0 if cached_native else 2*1024**3
    case['build_space_policy']='No link admitted: use the exact source-bound validated existing native artifact' if cached_native else 'Reserve two GiB before an actual Cargo/rustdoc build'
    if not cached_native and free_bytes < 4*1024**3:
        with (root/'target/debug/.cargo-lock').open('a+b') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX)
            removed=[]
            for cache in (root/'target/debug/incremental').iterdir():
                if cache.is_dir():
                    removed.append({'path':str(cache.relative_to(root)),'allocated_bytes':sum(p.stat().st_blocks*512 for p in cache.rglob('*') if p.is_file())});shutil.rmtree(cache)
            case['inactive_incremental_prune']=removed
        free_bytes=shutil.disk_usage(root/'target').free
    while free_bytes < build_reserve and active:
        print('STORAGE DRAIN',label,free_bytes,flush=True)
        reap()
        forward_progress()
        time.sleep(.1)
        free_bytes=shutil.disk_usage(root/'target').free
    if free_bytes < build_reserve:
        case.update(state='storage-deferred',build_start_free_disk_bytes=free_bytes,reason='insufficient disk for raw link output; no Cargo process admitted; active native scopes drained normally')
        save();print('STORAGE DEFERRED',label,free_bytes,flush=True);continue
    cpu=next(cpu for cpu in slots if cpu not in active)
    start=time.monotonic();case.update(state='building',started_utc=time.strftime('%Y-%m-%dT%H:%M:%SZ',time.gmtime()));save()
    buildlog=out/(label+'.build.jsonl');errlog=out/(label+'.build.stderr')
    executable=None
    if case['kind']=='doc':
        inventory_command=['taskset','-c',str(cpu),'python3','-B','scripts/limited_verification.py','--cloud','--',*base,'--','--list']
        inventory=subprocess.run(inventory_command,capture_output=True,text=True,env=build_env)
        inventory_path=out/(label+'.inventory.log')
        inventory_path.write_text(inventory.stdout+inventory.stderr)
        case['inventory_log']=str(inventory_path.relative_to(root))
        case['inventory_sha256']=hashlib.sha256(inventory_path.read_bytes()).hexdigest()
        case['inventory_command']=inventory_command
        assert inventory.returncode==0,(case,inventory.stderr)
        case['inventoried_tests']=len(re.findall(r'^.+: test$',inventory.stdout,re.M))
        case['inventoried_benchmarks']=len(re.findall(r'^.+: benchmark$',inventory.stdout,re.M))
        command=base+['--','--test-threads=1']+(['--include-ignored'] if args.include_ignored else [])
    else:
        group=selector_key(case)
        group_name=':'.join(group)
        reused_group=group in artifact_cohorts
        cohort_base=base
        command_build=cohort_base+['--no-run','--message-format=json-render-diagnostics']
        if active:
            command_build=['taskset','-c',str(cpu),'python3','-B','scripts/limited_verification.py','--cloud','--',*command_build]
        else:
            command_build=['python3','-B','scripts/limited_verification.py','--cloud','--cloud-cpus','auto','--',*command_build]
        case['build_command']=command_build
        if reused_group:
            executable=artifact_cohorts[group][(package_ids[case['package']],case['target'],case['kind'])]
            assert executable.is_file()
            if 'carried_unexecuted_outputs' in build_groups[group_name]:
                admission=next(row for row in build_groups[group_name]['carried_unexecuted_outputs'] if row['path']==str(executable.relative_to(root)))
                assert hashlib.sha256(executable.read_bytes()).hexdigest()==admission['sha256']
                case['carried_library_admission']=admission
            case['build_exit']=0
            case['build_command']=build_groups[group_name]['command']
            case['reused_build_group']=group_name
        else:
            with buildlog.open('w') as stdout,errlog.open('w') as stderr:
                build_exit=build_with_completed_cache_retirement(command_build,stdout,stderr,case)
            case['build_exit']=build_exit
            if build_exit:
                case.update(state='build-failed',seconds=round(time.monotonic()-start,3))
                with (root/'target/debug/.cargo-lock').open('a+b') as lock:
                    fcntl.flock(lock,fcntl.LOCK_EX);release_incremental(case)
                seal_build_logs(case,[buildlog,errlog])
                save();print('BUILD FAILED',label,flush=True)
                if args.stop_on_build_failure:
                    result['stopped_after_build_failure']=label
                    break
                continue
            expected=[(package_ids[c['package']],c['target'],c['kind']) for c in full_inventory if selector_key(c)==group]
            messages=[json.loads(line) for line in buildlog.read_text().splitlines()]
            artifact_cohorts[group]=test_artifacts(messages,expected)
            executable=artifact_cohorts[group][(package_ids[case['package']],case['target'],case['kind'])]
            build_groups[group_name]={'command':command_build,'executables':[{'package_id':key[0],'target':key[1],'kind':key[2],'path':str(path.relative_to(root))} for key,path in artifact_cohorts[group].items()],
                'policy':'One stable workspace feature graph and one build per selector; every selected exact package executable is independently inventoried and executed.',
                'completed_unit_cache_retirement':case['completed_unit_cache_retirement']}
            wanted={(package_ids[c['package']],c['target'],c['kind']) for c in cases if selector_key(c)==group}
            retired_siblings=[]
            for key,path in artifact_cohorts[group].items():
                if key not in wanted:
                    assert path.parent.resolve()==(root/'target/debug/deps').resolve()
                    retired_siblings.append({'package_id':key[0],'target':key[1],'kind':key[2],'path':str(path.relative_to(root)),'bytes':path.stat().st_size,'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'reason':'Unselected already-completed sibling rebuilt by stable workspace selector; no native admission or verdict claimed.'})
                    path.unlink()
            build_groups[group_name]['unselected_artifacts_retired']=retired_siblings
        case['build_group']=group_name
        if args.shared_runtime_linker:
            # PATH is deliberately outside Cargo's compilation fingerprints.
            # A fresh static ELF must be explicitly relinked before claiming
            # data sharing. Targets with no embedded image need no DSO.
            dynamic=subprocess.check_output(['readelf','-d',str(executable)],text=True)
            if 'liblila_runtime_images_' not in dynamic:
                symbols=subprocess.check_output(['nm','-g','--defined-only',str(executable)],text=True)
                if re.search(r'\bLILA_RUNTIME_(PACKAGE|NATIVE|MANIFEST)$',symbols,re.M):
                    case['static_executable_relinked']=True
                    executable.unlink()
                    with buildlog.open('a') as stdout,errlog.open('a') as stderr:
                        rebuilt=subprocess.run(command_build,stdout=stdout,stderr=stderr,env=build_env)
                    assert rebuilt.returncode==0,(case,errlog.read_text()[-4000:])
                    dynamic=subprocess.check_output(['readelf','-d',str(executable)],text=True)
                    assert 'liblila_runtime_images_' in dynamic,case
        case['executable_sha256']=hashlib.sha256(executable.read_bytes()).hexdigest()
        case['executable_bytes']=executable.stat().st_size
        if args.shared_runtime_linker:
            dynamic=subprocess.check_output(['readelf','-d',str(executable)],text=True)
            needed=re.findall(r'\(NEEDED\).*\[([^]]+)\]',dynamic)
            case['needed_libraries']=needed
            shared=[name for name in needed if name.startswith('liblila_runtime_images_')]
            assert len(shared)<=1
            if shared:
                image=root/'target/shared-test-runtime-images'/shared[0]
                receipt=json.loads(image.with_suffix('.json').read_text())
                assert hashlib.sha256(image.read_bytes()).hexdigest()==receipt['shared_sha256']
                assert str(image.parent) in dynamic
                events=[json.loads(line) for line in (image.parent/'test-links.jsonl').read_text().splitlines()]
                matched=[event for event in events if event.get('executable_sha256')==case['executable_sha256'] and event['shared_image']==str(image)]
                assert matched,(case,'no sealed linker event')
                case['shared_link_event']=matched[-1]
                if args.strip_test_debug:
                    debug=matched[-1].get('debug_sections')
                    assert debug and debug['stripped_sha256']==case['executable_sha256'],(case,'stripping not sealed')
                case['shared_runtime_image']=receipt
        seal_build_logs(case,[buildlog,errlog])
        inventory=subprocess.run([str(executable),'--list']+([native_filters[case['package']+':'+case['target']]] if case['package']+':'+case['target'] in native_filters else []),capture_output=True,text=True)
        inventory_path=out/(label+'.inventory.log')
        inventory_path.write_text(inventory.stdout+inventory.stderr)
        case['inventory_log']=str(inventory_path.relative_to(root))
        case['inventory_sha256']=hashlib.sha256(inventory_path.read_bytes()).hexdigest()
        assert inventory.returncode==0,(case,inventory.stderr)
        case['inventoried_tests']=len(re.findall(r'^.+: test$',inventory.stdout,re.M))
        case['inventoried_benchmarks']=len(re.findall(r'^.+: benchmark$',inventory.stdout,re.M))
        if case['package']+':'+case['target'] in native_filters:assert case['inventoried_tests']>0, 'Native filter selected no tests'
        command=[str(executable),'--test-threads=1']+(['--include-ignored'] if args.include_ignored else [])
        if (native_filter:=native_filters.get(case['package']+':'+case['target'])):
            command.append(native_filter);case['native_filter']=native_filter
    partition_receipt=None
    if partitioned:
        assert not case.get('native_filter') and args.include_ignored and not active
        assert len(slots)==args.test_processes
        partition_output=out/(label+'-partitions')
        partition_receipt=partition_output/'receipt.json'
        partition_driver=root/'scripts/workspace_test_partitions.py'
        command=['python3','-B',str(partition_driver),'--executable',str(executable),'--inventory',str(out/(label+'.inventory.log')),'--output',str(partition_output),'--cpus',*map(str,slots),'--include-ignored']
        case['partition_driver_sha256']=hashlib.sha256(partition_driver.read_bytes()).hexdigest()
        case['partition_receipt_path']=str(partition_receipt.relative_to(root))
    log=out/(label+'.run.log');watch=root/'target/watched'/(watch_label+'.log')
    case.update(state='running',command=command,test_cpu=cpu,
      test_policy=('explicitly filtered native control with its matching exact inventory' if case.get('native_filter') else 'complete native executable')+' on one inherited CPU and one libtest thread; independent target concurrency recorded separately');save()
    stream=log.open('w')
    launch=['taskset','-c',str(cpu),'./scripts/run-watched.sh','--label',watch_label,
      '--stall','3600' if case['package']=='lila-cli' else '900','--poll','1','--',
      'python3','-B','scripts/limited_verification.py','--cloud','--',*command]
    if partitioned:
        launch=['./scripts/run-watched.sh','--label',watch_label,'--stall','3600','--poll','1','--','python3','-B','scripts/limited_verification.py','--cloud','--cloud-cpus','auto','--',*command]
        case['test_cpus']=slots
        case['test_policy']='complete logical native CLI target; all exact qualified IDs in disjoint source-bound partitions, each on one inherited CPU and one libtest thread; no other native target active'
        save()
    run=subprocess.Popen(launch,stdout=stream,stderr=subprocess.STDOUT,env=build_env)
    active[cpu]={'run':run,'stream':stream,'case':case,'start':start,'watch':watch,'log':log,
      'executable':executable,'label':label,'partition_receipt':partition_receipt}
    if barrier:
        while active:reap(wait=True)
while active:reap(wait=True)
save();raise SystemExit(not result['source_unchanged'] or any(c.get('state','unstarted')!='passed' for c in cases))
