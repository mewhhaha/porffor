#!/usr/bin/env python3
"""Verify a source-bound four-scope native checkpoint offline; no native execution."""
import argparse
import base64
import gzip
import hashlib
import io
import json
from pathlib import Path
import re

PRIOR_HEAD='ace7a60cfde80cbcd4d1fad6403f8a4386f75036'
PRIOR_NATIVE_HEAD='cc0210eba1bf646c35a4190c792951023033b5ad'
PRIOR_FAILED_IDS={
 'lila-ir:lila_ir_integration:ir_cases::finite_regexp_source::literal_and_resolved_constructor_sources_prepare_escaped_script_candidates',
 'lila-engine:aot_language:aot_callable_entry_roles::labelled_chains_do_not_invent_closure_aliases_or_finite_eval_bindings',
}
ADDED_IDS={
 'lila-ir:lila_ir:tests::labelled_case_chains_do_not_reserve_activation_cells_visible_to_direct_eval',
 'lila-ir:lila_ir:tests::labelled_chains_do_not_reserve_activation_cells_visible_to_direct_eval',
 'lila-engine:aot_language:aot_callable_entry_roles::labelled_nonlexical_chains_do_not_shadow_initialized_bindings_in_direct_eval',
}
FOCUS={('lila-ir','lila_ir'):None,('lila-ir','lila_ir_integration'):'ir_cases::finite_regexp_source::',
       ('lila-engine','aot_language'):'aot_callable_entry_roles::',('lila-engine','aot_regexp'):'aot_regexp_finite_source::'}
EMPTY=hashlib.sha256(b'').hexdigest()


def require(value,message):
    if not value:raise ValueError(message)


def sha(raw):return hashlib.sha256(raw).hexdigest()


def unique(pairs):
    result={}
    for k,v in pairs:
        require(k not in result,'Duplicate JSON field');result[k]=v
    return result


class Evidence:
    def __init__(self,files):
        self.files={};total=0
        for path,row in files.items():
            require(path.startswith(('/workspace/','/tmp/')) and '/..' not in path,'Invalid evidence path')
            require(row['codec']=='gzip-base64','Unknown evidence codec')
            with gzip.GzipFile(fileobj=io.BytesIO(base64.b64decode(row['data'],validate=True))) as stream:
                raw=stream.read(16*1024**2+1)
            require(len(raw)<=16*1024**2 and len(raw)==row['bytes'] and sha(raw)==row['sha256'],'Corrupt evidence bytes')
            require(not raw.startswith(b'\x7fELF'),'Native ELF embedding forbidden')
            total+=len(raw);require(total<=64*1024**2,'Evidence exceeds bounded size');self.files[path]=raw
    def raw(self,reference):
        raw=self.files[reference['path']]
        require(sha(raw)==reference['sha256'],'Bound evidence hash changed')
        if 'bytes' in reference:require(len(raw)==reference['bytes'],'Bound evidence size changed')
        return raw
    def doc(self,reference):return json.loads(self.raw(reference),object_pairs_hook=unique)
    def path(self,path,digest=None):
        if not path.startswith('/'):path='/workspace/porffor-next/'+path
        raw=self.files[path]
        require(digest is None or sha(raw)==digest,'Original path digest changed');return raw


def owner(e,reference,exit_code,head):
    value=e.doc(reference)
    require(value['complete'] is True and value['exit_code']==exit_code and value['head']==head
            and value['source_unchanged'] is True and value['source_before']==value['source_after'],
            'Actual outer/source equality or exit differs')
    source=value['source_before']
    require(source['head']==head and source['tracked_patch_sha256']==EMPTY and source['untracked']=={},'Focused source was not clean')
    require(value['native_acceptance'] is False,'Focused outer cannot claim full native acceptance')
    for name in ('stdout','stderr'):
        row=value[name];packed=e.path(row['path'],row['archive_sha256']);raw=gzip.decompress(packed)
        require(sha(raw)==row['raw_sha256'] and len(raw)==row['raw_bytes'],'Actual outer raw output differs')
    return value


def build_outputs(e,row):
    logs=[];stderr=b''
    for archive in row['sealed_build_logs']:
        raw=gzip.decompress(e.path(archive['path'],archive['gzip_sha256']))
        require(sha(raw)==archive['uncompressed_sha256'] and len(raw)==archive['uncompressed_bytes'],'Raw Cargo log changed')
        if archive['path'].endswith('.build.jsonl.gz'):logs.extend(json.loads(line) for line in raw.decode().splitlines() if line)
        if archive['path'].endswith('.build.stderr.gz'):stderr+=raw
    return logs,stderr


def native_case(e,row,head,allow_failed=False):
    key=(row['package'],row['target']);require(key in FOCUS,'Unexpected focused scope')
    failed=row['exit_code']==101
    require(row['exit_code'] in (0,101) and row['state']==('failed' if failed else 'passed')
            and row['inventory_matches'] is (not failed) and (allow_failed or not failed)
            and row.get('native_filter')==FOCUS[key],'Scope/filter or literal outcome differs')
    inventory=e.path(row['inventory_log'],row['inventory_sha256']).decode()
    names=re.findall(r'^(.+): test$',inventory,re.M)
    require(names and len(names)==len(set(names))==row['inventoried_tests'],'Native inventory mismatch')
    require(row.get('inventoried_benchmarks',0)==0,'Unexpected benchmark inventory')
    transcript=e.path(row['transcript_log'],row['transcript_sha256']).decode()
    actual=re.findall(r'^test (.+?)(?: - should panic)? \.\.\. (ok|FAILED|ignored)[ \t]*$',transcript,re.M)
    require(len(actual)==len(names) and {n for n,_ in actual}==set(names)
            and all(result in ('ok','FAILED') for _,result in actual),'Exact inventoried native IDs were not all run once')
    counts={'passed':sum(result=='ok' for _,result in actual),'failed':sum(result=='FAILED' for _,result in actual),'ignored':0}
    summaries=[dict(zip(('passed','failed','ignored'),map(int,values))) for values in
               re.findall(r'^test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored;',transcript,re.M)]
    require(summaries==row['outcomes']==[counts] and bool(counts['failed'])==failed,'Native raw result summary differs')
    cmd=row['command'];expected=[cmd[0],'--test-threads=1','--include-ignored']+([FOCUS[key]] if FOCUS[key] else [])
    require(cmd==expected,'Native focused argv/thread/filter changed')
    require(row['executable_sha256'] and row['executable_bytes']>0 and row.get('executable_released') is True,
            'Actual test executable identity/lifecycle missing')
    logs,_=build_outputs(e,row)
    require(logs and [m.get('success') for m in logs if m.get('reason')=='build-finished'][-1:] == [True],
            'No successful actual Cargo producer transcript')
    selected=[m for m in logs if m.get('reason')=='compiler-artifact' and m.get('profile',{}).get('test') is True
              and m.get('target',{}).get('name')==row['target'] and m.get('executable')==cmd[0]]
    require(selected and all(m['target']['kind']==(['lib'] if row['kind']=='lib' else ['test']) for m in selected),
            'Actual Cargo test producer does not match native executable path')
    revisions=[dict(m['env'])['LILA_COMPILER_SOURCE_REVISION'] for m in logs
               if m.get('reason')=='build-script-executed' and 'LILA_COMPILER_SOURCE_REVISION' in dict(m.get('env',[]))]
    require(all(revision=='git:'+head for revision in revisions),'A compiler producer has stale source revision')
    if row.get('shared_link_event'):
        require(row['shared_link_event']['executable_sha256']==row['executable_sha256'],'Shared link event differs from actual test image')
    return {label:[':'.join(key)+':'+name for name,result in actual if result==status]
            for label,status in [('passed','ok'),('failed','FAILED')]}


def failed_attempt(e,attempt):
    earlier=owner(e,attempt['outer'],1,PRIOR_HEAD);native=e.doc(attempt['native'])
    require(native['head']==PRIOR_HEAD and native['source_unchanged'] is True,'Earlier failed source differs')
    require(native['source_map_sha256']==sha(json.dumps(earlier['source_before'],sort_keys=True).encode()),
            'Earlier native and outer source-owner maps differ')
    require(len(native['cases'])==4 and set(FOCUS)=={(r['package'],r['target']) for r in native['cases']},'Earlier selected scopes changed')
    require(attempt['native_test_commands']==0 and all(not any(k in row for k in
            ('command','exit_code','inventory_log','transcript_log','executable_sha256')) for row in native['cases']),
            'Earlier failed attempt executed a native test')
    if attempt['kind']=='preflight':
        require(all('build_command' not in row and 'state' not in row for row in native['cases']),
                'Earlier preflight executed a Cargo build')
        require(attempt['cargo_build_exit'] is None,'Earlier preflight was relabelled as a build')
    else:
        require(attempt['kind']=='compile' and attempt['cargo_build_exit']==101,'Unexpected failed-attempt classification')
        failed=[row for row in native['cases'] if row.get('state')=='build-failed']
        require(len(failed)==1 and (failed[0]['package'],failed[0]['target'])==('lila-ir','lila_ir')
                and failed[0]['build_exit']==101 and all(row is failed[0] or 'build_command' not in row for row in native['cases']),
                'Earlier Cargo failure was reinterpreted')
        command=failed[0]['build_command']
        require('--no-run' in command and any(command[i:i+2]==['cargo','test'] for i in range(len(command)-1)),
                'Earlier failed Cargo command could execute native tests')
        logs,stderr=build_outputs(e,failed[0])
        require([m.get('success') for m in logs if m.get('reason')=='build-finished']==[False]
                and all(s in stderr for s in (b'error[E0609]',b'capture_aliases',b'FunctionPlan')),
                'Original failed Cargo transcript changed')
    return {'kind':attempt['kind'],'head':PRIOR_HEAD,'outer_exit_code':earlier['exit_code'],
            'cargo_build_exit':attempt['cargo_build_exit'],'native_test_commands':0}


def prior_native_failure(e,prior):
    wrapper=owner(e,prior['outer'],1,PRIOR_NATIVE_HEAD);native=e.doc(prior['native'])
    source=e.doc(prior['source_map']);e.doc(prior['cargo_metadata'])
    require(native['head']==PRIOR_NATIVE_HEAD and native['source_unchanged'] is True
            and native['profile']=='release' and native['all_features'] is False and native['include_ignored'] is True
            and native['shared_runtime_linker_unchanged'] is True,'Prior native default/source policy differs')
    require(source==wrapper['source_before'] and sha(json.dumps(source,sort_keys=True).encode())==native['source_map_sha256'],
            'Prior native and outer source-owner maps differ')
    require(len(native['cases'])==4 and {(row['package'],row['target']) for row in native['cases']}==set(FOCUS),
            'Prior native scopes differ')
    observed={'passed':[],'failed':[]}
    for row in native['cases']:
        result=native_case(e,row,PRIOR_NATIVE_HEAD,allow_failed=True)
        for key in observed:observed[key]+=result[key]
    require(len(observed['passed'])==len(set(observed['passed']))==1551
            and len(observed['failed'])==2 and set(observed['failed'])==PRIOR_FAILED_IDS,
            'Prior native literal positive/negative IDs differ')
    require(sorted(observed['passed'])==prior['passing_native_ids'] and sorted(observed['failed'])==prior['failed_native_ids'],
            'Prior native outcome projection changed')
    return observed


def added_control_ids(manifest):
    require(manifest['source_head']==PRIOR_NATIVE_HEAD and manifest['prepared_only'] is True
            and manifest['native_fix_verification_executed'] is False and manifest['native_acceptance'] is False,
            'New-control preparation was relabelled as verification')
    ids=[]
    for row in manifest['new_controls']:
        target='lila_ir' if (row['package'],row['target'])==('lila-ir','lib') else row['target']
        ids.append(':'.join((row['package'],target,row['id'])))
    require(len(ids)==3 and set(ids)==ADDED_IDS,'Added native control IDs differ')
    return set(ids)


def verify(value):
    head=value['head'];require(value['schema']=='actual-focused-native-v4' and re.fullmatch('[0-9a-f]{40}',head),'Wrong focused checkpoint')
    for key in ('full_workspace_passed','full_test262_passed','ordinary_cli55_passed','task_acceptance','native_elf_embedded'):
        require(value['claims'][key] is False,'Focused evidence cannot claim '+key)
    e=Evidence(value['files']);wrapper=owner(e,value['actual_outer'],0,head);native=e.doc(value['actual_native'])
    require(native['head']==head and native['profile']=='release' and native['all_features'] is False
            and native['include_ignored'] is True and native['source_unchanged'] is True
            and native['shared_runtime_linker_unchanged'] is True,'Native source/default/profile differs')
    source=e.doc(value['native_source_map']);e.doc(value['cargo_metadata'])
    require(source==wrapper['source_before'] and sha(json.dumps(source,sort_keys=True).encode())==native['source_map_sha256'],
            'Native and outer source-owner maps differ')
    require('scripts/run_workspace_tests.py' in wrapper['command'] and '--all-features' not in wrapper['command'],'Not an actual default native focus')
    output=Path(wrapper['command'][wrapper['command'].index('--output')+1])
    if not output.is_absolute():output=Path('/workspace/porffor-next')/output
    require(value['actual_native']['path']==str(output/'receipt.json'),'Actual native receipt does not belong to outer command')
    admitted=value['local_admission'];e.raw(value['local_gate'])
    require(admitted['outer']==value['actual_outer'] and admitted['native']['path']==value['actual_native']['path']
            and admitted['native']['sha256']==value['actual_native']['sha256'] and admitted['head']==head
            and admitted['default_mode'] is True and admitted['scopes']==sorted(':'.join(k) for k in FOCUS),
            'Recorded stock focused admission differs from embedded evidence')
    require({(r['package'],r['target']) for r in native['cases']}==set(FOCUS) and len(native['cases'])==4,'Exact four focused scopes required')
    ids=[]
    for row in native['cases']:ids+=native_case(e,row,head)['passed']
    require(len(ids)==len(set(ids)),'Duplicate fully qualified native execution ID')
    require(sorted(ids)==value['passing_native_ids'],'Passing native ID projection changed')
    require([row['kind'] for row in value['earlier_failed_attempts']]==['preflight','compile'],'Preserve both earlier failed attempts')
    failures=[failed_attempt(e,row) for row in value['earlier_failed_attempts']]
    prior=prior_native_failure(e,value['earlier_native_failure'])
    added=added_control_ids(e.doc(value['new_controls_manifest']))
    require(set(prior['passed']+prior['failed'])|added<=set(ids),
            'Every prior native pass, both failed IDs and all three added controls must pass in the repaired focus')
    return {'focused_native_evidence_verified':True,'head':head,'scopes':4,'native_tests_passed':len(ids),
            'failed':0,'ignored':0,'earlier_failed_attempts':failures,
            'prior_native_positive_ids_retained':len(prior['passed']),'prior_native_failed_ids_now_pass':sorted(prior['failed']),
            'new_native_controls_passed':sorted(added),
            'test_executable_presence_verified_offline':False,'full_workspace_passed':False,
            'full_test262_passed':False,'ordinary_cli55_passed':False,'task_acceptance':False}


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('checkpoint',type=Path);args=parser.parse_args()
    value=json.loads(args.checkpoint.read_bytes(),object_pairs_hook=unique)
    require(value['reader_sha256']==sha(Path(__file__).read_bytes()),'Exact offline reader bytes differ')
    print(json.dumps(verify(value),indent=2))


if __name__=='__main__':main()
