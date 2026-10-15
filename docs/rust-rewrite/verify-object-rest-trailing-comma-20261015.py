#!/usr/bin/env python3
"""Verify the portable parser-native transcript and source checkpoint, without executing native code."""
import argparse
import base64
import hashlib
import json
from pathlib import Path
import zlib


def require(value,message):
    if not value:raise ValueError(message)


def digest(data):return hashlib.sha256(data).hexdigest()


def verify(proof,root=None):
    require(proof['schema']=='object-rest-trailing-comma-native-retention-v1' and
        proof['status']=='passed','Closed actual parser-native PASS checkpoint required')
    payloads={}
    for key,row in proof['payloads'].items():
        require(row['encoding']=='base64(zlib(original bytes))' and 0<=row['bytes']<=32*1024**2,'Bounded lossless payload required')
        stream=zlib.decompressobj();data=stream.decompress(base64.b64decode(row['data'],validate=True),row['bytes']+1)
        require(stream.eof and not stream.unused_data and len(data)==row['bytes'] and digest(data)==key==row['sha256'],
            'Raw native/source payload digest or length differs')
        payloads[key]=data
    counts={'new_front_regressions':0,'prior_front_retained':0,'boa_constructor_witnesses':0,'total_selected_passed':0}
    targets={}
    for target in proof['native_targets']:
        require(target['target'] not in targets,'Duplicate native target')
        ids=[line[:-6] for line in payloads[target['inventory_payload']].decode().splitlines() if line.endswith(': test')]
        require(ids==target['inventory_ids'] and ids and len(ids)==len(set(ids)),'Actual native inventory differs')
        chosen=target['selected_ids'];require(chosen and len(chosen)==len(set(chosen)) and set(chosen)<=set(ids),'Invalid selected native IDs')
        require(target['unrun_ids']==sorted(set(ids)-set(chosen)),'Unrun vendor IDs not explicitly retained')
        text=payloads[target['run_payload']].decode();filtered=len(ids)-len(chosen);n=len(chosen)
        require(all(text.count('test '+name+' ... ok')==1 for name in chosen) and
            ('test result: ok. '+str(n)+' passed; 0 failed; 0 ignored; 0 measured; '+str(filtered)+' filtered out;') in text,
            'Every selected native ID must have an actual PASS with exact zero-failure summary')
        role=target['role'];require(role in ['new_front_regressions','prior_front_retained','boa_constructor_witnesses'],'Foreign native acceptance role')
        if role!='boa_constructor_witnesses':require(filtered==0,'Front native retention may not filter IDs')
        counts[role]+=n;counts['total_selected_passed']+=n;targets[target['target']]=target
    require(counts==proof['native_counts'] and counts['new_front_regressions']==7 and counts['boa_constructor_witnesses']==19,
        'Native counts differ from actual complete transcripts')
    require(len(targets)==16,'New regression, parser witness and fourteen prior Front targets required')
    for row in proof['prior_front_retention']['targets']:
        if not row['ids']:continue
        name='front_unit' if row['target']=='lila_front' else 'front_'+row['target']
        require(name in targets and set(row['ids'])==set(targets[name]['inventory_ids'])==set(targets[name]['selected_ids']),
            'A prior published Front native ID was lost or not executed')
        prior=[line[:-6] for line in payloads[row['inventory_payload']].decode().splitlines() if line.endswith(': test')]
        require(prior==row['ids'],'Earlier published inventory payload differs')
    failures=proof['preserved_failures'];require(len(failures)==5,'Both harness setup REDs, stale API/source guard REDs and storage deferral required')
    for row in failures:
        text=payloads[row['diagnostic_payload']].decode()
        require(row['exit_code']==1 and all(needle in text for needle in row['required_diagnostic_text']),
            'A literal prior failure was omitted or reclassified as passing')
    require([r['classification'] for r in failures[:2]]==['harness-setup','harness-setup'],'Indoc setup REDs are not conformance outcomes')
    provenance_bytes=payloads[proof['source_provenance_payload']];provenance=json.loads(provenance_bytes)
    require(digest(provenance_bytes)==proof['source_provenance_sha256'] and len(provenance['selected_files'])==7,
        'Seven-path source provenance differs')
    fixture=payloads[provenance['fixture_payload']]
    require(digest(fixture)==provenance['fixture']['sha256'] and len(fixture)==provenance['fixture']['bytes'],
        'Original pinned fixture bytes differ')
    component_bytes=payloads[proof['evidence_index']['source_component']]
    component=json.loads(component_bytes)
    require(digest(component_bytes)==provenance['component_sha256'],'Exact seven-path component seal differs')
    component_files={row['relative_path']:row for row in component['selected_files']}
    plan_bytes=payloads[proof['evidence_index']['final_native_plan']];plan=json.loads(plan_bytes)
    require(digest(plan_bytes)==provenance['native_plan_sha256'] and plan['source_manifest']['sha256']==provenance['component_sha256'],
        'Actual source-bound native plan differs')
    prefix_plan_bytes=payloads[proof['evidence_index']['source_correct_Front_prefix_plan']]
    prefix_plan=json.loads(prefix_plan_bytes)
    require(digest(prefix_plan_bytes)==plan['storage_previous_plan']['sha256'] and
        all(row['environment']['CARGO_MANIFEST_DIR']==plan['view']+'/crates/lila-front' for row in prefix_plan['stages']),
        'Earlier source-correct Front prefix plan differs')
    prefix_bytes=payloads[proof['evidence_index']['source_correct_Front_prefix_result']]
    prefix=json.loads(prefix_bytes)
    require(digest(prefix_bytes)==plan['storage_deferred_result']['sha256'] and prefix['complete'] and not prefix['passed'] and
        prefix['error'].startswith('Storage deferred: ') and prefix['source_before']==prefix['source_after'] and
        prefix['protected_before']==prefix['protected_after'] and len(prefix['completed_native'])==11 and
        prefix['completed_native']==plan['adopted_front_native'] and prefix['completed_tests']==plan['adopted_front_tests'] and
        len(prefix['commands'])==33 and all(row['exit_code']==0 for row in prefix['commands']),
        'Actual complete eleven-target PASS prefix followed only by reserve deferral differs')
    require(all(row['environment']['CARGO_MANIFEST_DIR']==plan['view']+'/crates/lila-front' for row in plan['stages']),
        'Front runtime inspection did not use the corrected selected view')
    for row in provenance['selected_files']:
        require(not Path(row['relative_path']).is_absolute() and '..' not in Path(row['relative_path']).parts,'Unsafe portable source path')
        candidate=payloads[row['candidate_payload']];published=payloads[row['published_payload']]
        require(digest(candidate)==row['candidate_sha256'] and digest(published)==row['published_sha256'],'Exact selected source bytes differ')
        require(row['candidate_sha256']==component_files[row['relative_path']]['candidate']['sha256'] and
            row['candidate_sha256']==plan['source_map'][plan['view']+'/'+row['relative_path']]['sha256'],
            'Selected native-view source differs from the published code provenance')
        if row['relative_path'].endswith('.rs'):require(candidate==published,'Compiled Rust source changed during publication staging')
        if root is not None:require(digest((Path(root)/row['relative_path']).read_bytes())==row['published_sha256'],'Published checkout source differs')
    require(proof['ordinary_build_acceptance'] is False and proof['full_test262_acceptance'] is False and
        proof['performance_acceptance'] is False and proof['task_closure'] is False,'Native checkpoint scope was broadened')
    current=json.loads(payloads[proof['closed_result_payload']]);require(current['complete'] and current['passed'] and current['exit_code']==0 and
        current['source_unchanged'] and current['source_before']==current['source_after'] and current['protected_before']==current['protected_after'],
        'Actual closed source/protected native result differs')
    return counts


def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('checkpoint',type=Path);ap.add_argument('--root',type=Path)
    args=ap.parse_args();counts=verify(json.loads(args.checkpoint.read_text()),args.root)
    print(json.dumps({'verified':True,'native_counts':counts,'native_executions':0}))


if __name__=='__main__':main()
