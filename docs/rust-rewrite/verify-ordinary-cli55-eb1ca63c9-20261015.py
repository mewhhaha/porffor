#!/usr/bin/env python3
"""Verify the actual CLI55 checkpoint offline, using two existing public defaults."""
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

HEAD='eb1ca63c9dbf09148703ccfbd0ca35d9629753e7'
SCHEMA='ordinary-cli55-retention-publication-v1'
INPUT_SHA='35c95407489e875ef28ddbd61592c01e7c2907061ba398b60d219063ab1e04a6'
PRIOR_SHA='3b5117ca2cac5e68b90850849e8dd22b4701f77f6c29cb11444b28fc75973659'
PRIOR_READER_SHA='f2e792512d4df711de32f83ef510b59f30984a0be60e61e82e2968162603a187'
POLICY_SHA='9f0cda6f53e648e7124b672bb57a36a3bc9f635c6c973c93ea62ba51ab9a96cd'
SOURCE_PINS_SHA='ab47a6da190c07bc54ac2cc752580345f89fce9c31b739e7987be78b5ffea480'
PACKET_SHA='36f9a11c5357c19e70b66cc91e84816f8b484e77651d7e7b626ab54179005013'
BUILDER_SHA='ae64177ba061223d61018bd11af284a66f7eb12dad06930c633a4384b6411cf6'
OLD_GATE_SHA='a63830292b9c2a93d47fa5d0cf4bd8cafa0ee20dc568ec80719676ae54cb5ce9'
DEFAULT_ADMISSION_SHA='beec74fd95c41af1f52f02591822e9e356408f6a31adafb526ebbd8fed813fef'
WORKSPACE_READER_SHA='7e5447ab7a7af128733d4c7f9c1099e428090cb40d700b7885ac55aca7609ebe'
ADOPTION_SHA='a3c6dd858a6057c1ebe0aca2037ebc5644dc82f47b71a9a76989ffacd86782b5'
ADOPTION_PACKET_SHA='bc5db4cdb77ffe79029f04cbd0e938bf29792aed421c7557eed10e39f643525a'
ADOPTION_VERIFIER_SHA='323309fac93046358a70ffaf0876386cd0f171fc9ca99665c894e8f148b6e0ab'
CANONICAL_GATE_SHA='c0b565abf6103c8354c8eedb3a93457ff57c3d9e67beaa35e9b1c1e21f0d015d'
ORIGINAL_QUEUE_SHA='e2dc813c936542a7f911414374e9a3b020eed6f6259eb1c021ca98d8fd456604'
ORIGINAL_SUPERVISOR_SHA='f2b925d1722f7dbe97929922a1e364511879dac14b304a0a3441b32a7612ea0d'
ORIGINAL_BUILD_SHA='3afb7a95fdbe9d8aca2008bce6dbd5437d46f998cc27fd7376f5df0f4c003018'
ORIGINAL_OUTER_SHA='9be91f80a5f05cdd4645b35c34246f4ef0341678ef531d6f4ad1c2a247bd00ed'
ORIGINAL_RUN_SHA='d9bad480a7c7f69cd3927ec32ab8e522351fdc4597c05e35d4086289b0e62d1e'
ORIGINAL_SUMMARY_SHA='e362c089c8334706f90e921a7b9b1d73ff9924a0398a03a51166cafaff7d1612'
DEFAULT={'filename':'default-workspace-eb1ca63c9-native-checkpoint-20261015.json',
         'sha256':'76837df1b468cd1da432bc1ff11eece2088cfa1b6a46d28b813bdf4f13776c8c'}
DEFAULT_PRIOR={'filename':'default-workspace-8e9a8c82-native-checkpoint-20261015.json',
               'sha256':'e99d0dd1c41facd021fa5e21740124c7ef20b216319df59e5724843661123e49'}
PINS={'ecma262':'5345883164f463e87f8b40aca4956157ecba8783','test262':'6a4268a9354a545d41c3b62efebc478ed8c521f4'}
COMMIT='2e0a56762801e275a9fdf96dc49d90ba0cddcf63'
BUGFIX=[mode+':staging/sm/RegExp/escape.js' for mode in ('sloppy-script','strict-script')]
OUTCOMES=('Success','NotImplemented','Crash','Bug','InfrastructureError')
LIMIT=64*1024*1024
EMPTY=hashlib.sha256(b'').hexdigest()


def require(value,message):
    if not value:raise ValueError(message)


def digest(raw):return hashlib.sha256(raw).hexdigest()


def decode_assets(report):
    assets={}
    for key,value in report['raw_evidence'].items():
        require(value['encoding']=='gzip-base64' and 0<=value['raw_bytes']<=LIMIT,'Evidence size/encoding differs')
        packed=base64.b64decode(value['data'],validate=True)
        require(digest(packed)==value['archive_sha256'] and len(packed)==value['archive_bytes'],'Evidence archive differs')
        with gzip.GzipFile(fileobj=io.BytesIO(packed)) as stream:raw=stream.read(LIMIT+1)
        require(len(raw)<=LIMIT and digest(raw)==key==value['raw_sha256'] and len(raw)==value['raw_bytes'],
                'Evidence raw bytes differ')
        assets[key]=raw
    require(sum(map(len,assets.values()))<=2*LIMIT,'Total embedded evidence exceeds bound')
    return assets


def raw_ref(reference,assets):
    raw=assets[reference['sha256']]
    require(digest(raw)==reference['sha256'] and ('bytes' not in reference or len(raw)==reference['bytes']),
            'Exact raw reference differs')
    return raw


def document(reference,assets):return json.loads(raw_ref(reference,assets))


def exact_namespace(raw,expected,label):
    require(digest(raw)==expected,'Reviewed offline reader changed: '+label)
    namespace={'__name__':'embedded_hash_pinned_'+label}
    exec(compile(raw,'<hash-pinned '+label+'>','exec'),namespace)
    return namespace


def pure_function(raw,expected,name,*,total55=False):
    """Reuse one hash-pinned pure rule, without executing its filesystem imports.

    The CLI53 normal-provenance rule has one literal requested count. Its only
    adaptation is 53 -> 55; every artifact/source/static-runtime check is kept.
    """
    require(digest(raw)==expected,'Original pure rule source differs')
    tree=ast.parse(raw);nodes=[node for node in tree.body if isinstance(node,ast.FunctionDef) and node.name==name]
    require(len(nodes)==1,'Exact reusable pure function missing')
    if total55:
        constants=[node for node in ast.walk(nodes[0]) if isinstance(node,ast.Constant) and type(node.value) is int and node.value==53]
        require(len(constants)==1,'Unexpected CLI53 provenance rule shape')
        constants[0].value=55
    namespace={'require':require,'digest':digest,'HEAD':HEAD,'Path':Path,'json':json,'re':re,
               'hashlib':hashlib,'Counter':Counter}
    exec(compile(ast.fix_missing_locations(ast.Module(body=nodes,type_ignores=[])),'<hash-pinned pure '+name+'>','exec'),namespace)
    return namespace[name]


def external_default(report,assets,current_raw,prior_raw,policy):
    require(report['default_checkpoint']==DEFAULT and report['default_predecessor']==DEFAULT_PRIOR,
            'The existing public default filenames/hashes changed')
    require(digest(current_raw)==DEFAULT['sha256'] and digest(prior_raw)==DEFAULT_PRIOR['sha256'],
            'Provide both exact existing public default proofs')
    namespace=exact_namespace(assets[WORKSPACE_READER_SHA],WORKSPACE_READER_SHA,'workspace_reader')
    checkpoint=json.loads(current_raw)
    result=namespace['verify_current'](checkpoint,{'default':prior_raw})
    core=namespace['load_core'](base64.b64decode(checkpoint['raw_reader_source']['base64'],validate=True))
    audit=json.loads(core.unpack(checkpoint['base_audit']))
    rule=pure_function(assets[DEFAULT_ADMISSION_SHA],DEFAULT_ADMISSION_SHA,'metadata')
    admission=rule(checkpoint,result,audit,HEAD,policy)
    require(result['required_controls']==17 and result['native_counts']=={'ok':9176},
            'Actual current default native/control counts differ')
    return admission


def normal_build(report,assets):
    build=report['ordinary_build_preparation'];stage=document({'sha256':build['owner_raw_sha256']},assets)
    require(stage['driver_sha256']==build['driver_sha256']==BUILDER_SHA and not stage.get('controller_error')
            and not stage.get('final_guard_error'),'Ordinary build owner/guards differ')
    pure_function(assets[PRIOR_READER_SHA],PRIOR_READER_SHA,'normal_provenance',total55=True)(report,assets)
    messages=[json.loads(line) for line in assets[build['stdout_sha256']].splitlines() if line.startswith(b'{')]
    require([event['success'] for event in messages if event.get('reason')=='build-finished']==[True]
            and all(build[key] in messages for key in ('cli_event','engine_event','runtime_event')),
            'Actual Cargo stream/selected producers differ')
    event=build['cli_event'];identity=build['worker']['identity'];environment=dict(build['engine_event']['env'])
    require(event['profile']['test'] is False and event['profile']['opt_level']=='3' and event['features']==[]
            and event['target']['kind']==['bin'] and event['target']['name']=='lila'
            and event['target']['src_path']=='/workspace/porffor-next/crates/lila-cli/src/bin/lila.rs'
            and event['executable']=='/workspace/porffor/target/release/lila'
            and build['exit_code']==0 and build['ordinary_cli_acceptance'] is False,
            'Actual default optimized Cargo CLI graph differs')
    require(identity['source_revision']=={'kind':'git-commit','commit':HEAD}
            and environment['LILA_COMPILER_SOURCE_REVISION']=='git:'+HEAD
            and identity['source_fingerprint']==environment['LILA_COMPILER_FINGERPRINT']
            and identity['source_fingerprint_scheme']==environment['LILA_COMPILER_FINGERPRINT_SCHEME']
            and identity['executable_sha256']==build['worker']['sha256']==build['replay_compiler_sha256'],
            'Actual same-source compiler/engine identity differs')
    require(stage['build']['complete'] is True and all(stage['build'][key]==build[key]
            for key in ('command','exit_code','seconds','stdout_sha256','stderr_sha256')),
            'Published build fields differ from actual ordinary owner')
    return stage


def replay(report,assets,policy,stage,ids,queue):
    refs=report['actual_replay'];run=document(refs['run'],assets);summary=document(refs['summary'],assets)
    outer=document(refs['outer'],assets);worker=stage['native_cli_worker']
    require({key:value for key,value in summary.items() if key not in ('total','outcomes','results')}==run,
            'Summary/run manifests differ')
    require(run['execution_list_sha256']==run['batch_execution_list_sha256']==INPUT_SHA
            and raw_ref(refs['executions'],assets)==raw_ref(refs['batch_executions'],assets)==assets[INPUT_SHA]
            and run['batch_executions']==ids and run['absent_executions']==[]
            and run['compiler_identity']==worker['identity'] and run['binary_sha256']==worker['sha256']
            and run['binary_source']==worker['path'] and run['execution_list_source']==policy['input']['path']
            and run['replay_mode']=='native-execution-list-v1' and run['batch_threads']==1
            and run['suite_root']==policy['suite_root'] and run['fixture_sha256']==policy['fixture_sha256'],
            'Actual ordered input/compiler/fixtures differ')
    root=Path(run['binary']).parent
    require(root==Path(queue['plan']['replay_output']) and run['binary']==str(root/'compiler')
            and run['execution_list']==str(root/'executions') and run['batch_execution_list']==str(root/'batch-executions'),
            'Actual replay owned paths differ')
    source={'head':HEAD,'tracked_patch_sha256':EMPTY,'untracked':{}}
    require(outer['complete'] is outer['source_unchanged'] is True and outer['head']==HEAD
            and outer['native_acceptance'] is False and outer['source_before']==outer['source_after']==source
            and outer['driver_sha256']==policy['helper_refs']['outer_driver']['sha256']
            and outer['cwd']=='/workspace/porffor-next' and outer['cargo_target_directory']=='/workspace/porffor/target'
            and outer['exit_code']==0 and outer['command']==queue['plan']['commands']['probe55'][6:],
            'Actual closed replay outer differs')
    expected_outer=['taskset','-c','3','python3','-B','scripts/limited_verification.py','--cloud','--',
                    'python3','-B','scripts/replay-test262-executions.py',policy['input']['path'],
                    '--binary',worker['path'],'--suite-root',policy['suite_root'],'--output-dir',str(root),
                    '--workers','1','--reuse-case-workers']
    require(outer['command']==expected_outer,'Actual replay cloud/worker policy differs')
    for stream in ('stdout','stderr'):
        row=outer[stream];raw=assets[row['raw_sha256']]
        value=report['raw_evidence'][row['raw_sha256']]
        require(len(raw)==row['raw_bytes'] and value['archive_sha256']==row['archive_sha256'],
                'Actual outer stream/archive differs')
    require(summary['total']==len(summary['results'])==55 and report['results']==summary['results']
            and [row['execution_id'] for row in summary['results']]==sorted(ids),
            'Actual native results must be the unique canonical exact55 list')
    require(len(refs['invocations'])==len(refs['snapshots'])==1,'Require one genuine native invocation/snapshot')
    call=document(next(iter(refs['invocations'].values())),assets)
    expected=[run['binary'],'--jobs','1','test262','run-list',run['batch_execution_list'],
              '--execution-backend','wasm-aot','--suite-root',run['suite_root'],
              '--snapshot-dir',str(root/'snapshots'),'--snapshot-name','exact-batch',
              '--threads','1','--timeout-ms','60000','--reuse-case-workers']
    require(call['command']==expected and call['exit_code']==0 and call['seconds']>=0,
            'Actual native jobs/thread/deadline/exit differs')
    for stream in ('stdout','stderr'):require(call[stream+'_sha256'] in assets,'Actual native stream missing')
    snapshot_ref=refs['snapshots'][0];snapshot=document(snapshot_ref,assets)
    require(snapshot['snapshot_version']==8 and snapshot['execution_backend']=='wasm-aot' and snapshot['run_kind']=='full'
            and snapshot['compiler_identity']==worker['identity'] and snapshot['pinned_revisions']==PINS
            and len(snapshot['completed_test_ids'])==len(set(snapshot['completed_test_ids']))==snapshot['total']==55
            and set(snapshot['completed_test_ids'])==set(ids) and snapshot['failures']==[]
            and snapshot['counts_per_outcome']=={name:(55 if name=='Success' else 0) for name in OUTCOMES[:-1]},
            'Actual native exact55 denominator, literal outcomes or pins differ')
    for row in summary['results']:
        execution=row['execution_id'];saved=document(refs['saved_results'][execution],assets)
        require(saved==row and row['protocol']=='native-execution-list-v1' and row['outcome']=='Success'
                and type(row['exit_code']) is int and row['exit_code']==0 and not row.get('infrastructure_error')
                and row['transcript']==call['stdout'] and row['transcript_sha256']==call['stdout_sha256']
                and row['native_snapshot']['sha256']==snapshot_ref['sha256']
                and document(refs['invocations'][row['invocation']],assets)==call,
                'Actual saved per-ID native projection differs')
    require(summary['outcomes']=={name:(55 if name=='Success' else 0) for name in OUTCOMES[:-1]},
            'Literal summary outcome counts differ')
    inventory=assets[report['native_inventory']['embedded_raw_sha256']].decode().splitlines()
    native=run['native_inventory']
    require(native['stdout_sha256']==report['native_inventory']['sha256']==report['native_inventory']['embedded_raw_sha256']
            and native['command']==[run['binary'],'--jobs','1','test262','list-all','--suite-root',run['suite_root'],
                                   '--execution-backend','wasm-aot']
            and len({name.split(':',1)[1] for name in inventory[1:]})==53616,
            'Pinned list-all file denominator/native argv differs')
    return summary,outer


def adoption_closure(report,assets,policy,ids,prior_ids,admission):
    refs=report['closure'];queue=document(refs['original_queue'],assets)
    supervisor=document(refs['original_supervisor'],assets);adopt=document(refs['adoption'],assets)
    packet=document(refs['adoption_packet'],assets)
    require({name:refs[name]['sha256'] for name in ('original_queue','original_supervisor','adoption',
                'adoption_packet','adoption_verifier','corrected_gate')}=={
                'original_queue':ORIGINAL_QUEUE_SHA,'original_supervisor':ORIGINAL_SUPERVISOR_SHA,
                'adoption':ADOPTION_SHA,'adoption_packet':ADOPTION_PACKET_SHA,
                'adoption_verifier':ADOPTION_VERIFIER_SHA,'corrected_gate':CANONICAL_GATE_SHA},
            'Exact actual original/adoption proof identities differ')
    for row in packet['files']:
        raw_ref(row,assets)
    require(adopt['schema']=='ordinary-cli55-canonical-results-adoption-v1'
            and adopt['complete'] is adopt['passed'] is True and adopt['head']==HEAD and adopt['commands']==[]
            and adopt['adoption_packet']==refs['adoption_packet'] and adopt['adoption_verifier']==refs['adoption_verifier']
            and adopt['native_or_cargo_rerun'] is adopt['producer_commands_created'] is adopt['original_queue_reclassified'] is False
            and adopt['full_suite_or_task_acceptance'] is False,'Actual narrow read-only adoption differs')
    require(queue['schema']=='next-source-ordinary-cli55-owner-v1' and queue['complete'] is False
            and queue['stage']=='closed-error' and queue['head']==HEAD and queue['source_unchanged'] is True
            and queue['packet_sha256']==PACKET_SHA and 'gate55' not in queue and not queue.get('final_guard_error')
            and queue['error'].endswith('ValueError: Preserve the exact ordered53+2 unique execution IDs\n'),
            'Keep the original queue literal incomplete/RED and diagnosed error')
    require(supervisor['complete'] is True and supervisor['stage']=='closed-error' and supervisor['exit_code']==1
            and supervisor['head']==HEAD and supervisor['source_unchanged'] is True
            and supervisor['source_before']==supervisor['source_after'] and len(supervisor['commands'])==1
            and supervisor['commands'][0]['label']=='cli55-run' and supervisor['commands'][0]['exit_code']==1,
            'Keep the original supervisor literal RED/exit1')
    for stream in ('stdout','stderr'):raw_ref(supervisor['commands'][0][stream],assets)
    commands=queue['commands'];require([row['label'] for row in commands]==
            ['producer-snapshot','ordinary-build','ordinary-build-verify','probe55'],'Actual four command inventory differs')
    for row in commands:
        require(row['complete'] is True and type(row['exit_code']) is int and row['exit_code']==0
                and row['command']==queue['plan']['commands'][row['label']], 'Original actual producer command changed or failed')
        for stream in ('stdout','stderr'):raw_ref(row[stream],assets)
    require(queue['admission_kind']=='complete-default-workspace' and queue['default_workspace']==
            {'checkpoint':queue['default_workspace']['checkpoint'],**admission}
            and queue['default_workspace']['checkpoint']['sha256']==DEFAULT['sha256'],
            'Actual17-control full-default admission differs')
    require(queue['frozen_sources_before']==queue['frozen_sources_after']==adopt['frozen_sources_before']==adopt['frozen_sources_after']
            and len(queue['frozen_sources_before'])==3 and all(row['patch_sha256']==EMPTY and not row['untracked']
            for row in queue['frozen_sources_before'].values())
            and adopt['index_before']==adopt['index_after'] and adopt['source_unchanged'] is True,
            'Actual source/index closure differs')
    oldgate=assets[OLD_GATE_SHA];corrected=raw_ref(refs['corrected_gate'],assets)
    old_text="""    require(len(rows)==len(values)==len(expected)==55 and [row['execution_id'] for row in rows]==expected,
            'Preserve the exact ordered53+2 unique execution IDs')"""
    new_text="""    require(len(prior)==len(set(prior))==53 and len(added)==len(set(added))==2
            and len(rows)==len(values)==len(expected)==len(set(expected))==55
            and [row['execution_id'] for row in rows]==sorted(expected),
            'Preserve the exact53+2 unique IDs in canonical native-result order')"""
    require(oldgate.decode().count(old_text)==1 and corrected==oldgate.decode().replace(old_text,new_text).encode(),
            'Gate repair changed anything beyond exact canonical-order validation')
    rule=pure_function(oldgate,OLD_GATE_SHA,'verdicts')
    try:rule(report['results'],prior_ids,BUGFIX)
    except ValueError as error:require(str(error)=='Preserve the exact ordered53+2 unique execution IDs','Old gate failed for another reason')
    else:raise ValueError('Original ordering gate failure no longer reproduced')
    rule=pure_function(corrected,refs['corrected_gate']['sha256'],'verdicts')
    gate=rule(report['results'],prior_ids,BUGFIX)
    actual=adopt['gate55']
    require(all(actual[key]==value for key,value in gate.items()) and actual['head']==HEAD
            and actual['native_pinned_revisions']==PINS and actual['upstream_test262_commit']==COMMIT
            and actual['actual_outer']['sha256']==report['actual_replay']['outer']['sha256']
            and actual['actual_summary']['sha256']==report['actual_replay']['summary']['sha256'],
            'Adopted gate differs from independently checked native exact55')
    require(adopt['original_failed_queue']['reference']==refs['original_queue']
            and adopt['original_failed_queue']['complete'] is False
            and adopt['original_failed_queue']['error']==queue['error']
            and adopt['original_failed_supervisor']['reference']==refs['original_supervisor']
            and adopt['original_failed_supervisor']['exit_code']==1
            and adopt['original_commands']==commands and adopt['plan']==queue['plan']
            and adopt['default_workspace']==queue['default_workspace']
            and adopt['input_order_preserved'] is True and adopt['native_result_order']=='canonical-sorted-execution-id'
            and adopt['ordered_input_ids']==ids and adopt['canonical_native_execution_ids']==sorted(ids)
            and adopt['input']['sha256']==INPUT_SHA,
            'Read-only adoption lost original literal provenance/order distinction')
    return queue,gate


def verify(report,default_raw,default_prior_raw):
    require(report['schema']==SCHEMA and report['head']==HEAD and report['complete'] is True
            and report['execution_evidence_verified'] is report['native_cli_replay_executed'] is True
            and report['binary_payloads_embedded'] is False and report['full_test262_acceptance'] is False
            and report['current_source_full_historical_replay_executed'] is False
            and report['full_pin_acceptance'] is report['task_closure'] is False,'Scope is incomplete or overstated')
    assets=decode_assets(report);policy=json.loads(assets[POLICY_SHA]);prior=json.loads(assets[PRIOR_SHA])
    reader=exact_namespace(assets[PRIOR_READER_SHA],PRIOR_READER_SHA,'CLI53_reader')
    old=reader['verify'](prior)
    require(old['all53_passed'] is True and prior['ordinary_cli_probe53_passed'] is True,'Prior53 proof did not PASS')
    prior_ids=[row['execution_id'] for row in prior['results']];ids=assets[INPUT_SHA].decode().splitlines()
    require(len(ids)==len(set(ids))==55 and ids==prior_ids+BUGFIX and policy['bugfix_ids']==BUGFIX
            and policy['input']['sha256']==INPUT_SHA and report['input']['sha256']==INPUT_SHA
            and policy['prior53']['sha256']==PRIOR_SHA and all(row['outcome']=='Success' for row in prior['results']),
            'Exact ordered prior53 plus two original RegExp modes changed')
    require(report['outcomes']=={name:(55 if name=='Success' else 0) for name in OUTCOMES}
            and report['total']==55 and report['ordinary_cli55_passed'] is True,'Literal55 outcome counts differ')
    require(policy['source_pins']=={'ecma262':PINS['ecma262'],'test262_upstream':COMMIT,'test262_tree':PINS['test262']}
            and json.loads(assets[SOURCE_PINS_SHA])==policy['source_pins']
            and report['pinned_revisions']==PINS and report['fixture_sha256']==policy['fixture_sha256'],
            'Native tree/upstream commit or pinned fixture mapping differs')
    admission=external_default(report,assets,default_raw,default_prior_raw,policy)
    queue,gate=adoption_closure(report,assets,policy,ids,prior_ids,admission)
    stage=normal_build(report,assets)
    require(queue['ordinary_build_owner']['sha256']==report['ordinary_build_preparation']['owner_raw_sha256']==ORIGINAL_BUILD_SHA
            and stage['frozen_sources_before']==stage['frozen_sources_after']==queue['frozen_sources_before']
            and {name:report['actual_replay'][name]['sha256'] for name in ('outer','run','summary')}=={
                'outer':ORIGINAL_OUTER_SHA,'run':ORIGINAL_RUN_SHA,'summary':ORIGINAL_SUMMARY_SHA},
            'Actual queue/build owner binding differs')
    replay(report,assets,policy,stage,ids,queue)
    require(report['retention']=={'prior53_required':53,'prior53_retained':53,'added_regexp_modes':BUGFIX,
                                'both_regexp_modes_passed':True,'regressions':[]},'Retention projection differs')
    return {'verified':True,'head':HEAD,'total':55,'outcomes':report['outcomes'],'ordinary_cli55_passed':True,
            'retained_prior53_successes':53,'added_regexp_successes':2,'default_required_controls':17,
            'actual_native_and_outer_exit':0,'original_queue_complete':False,'original_supervisor_exit':1,
            'adoption_native_or_cargo_rerun':False,'binary_payloads_embedded':False,
            'full_test262_acceptance':False,'full_pin_acceptance':False,'task_closure':False}


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('checkpoint',type=Path)
    parser.add_argument('--default-checkpoint',type=Path);parser.add_argument('--default-predecessor',type=Path)
    args=parser.parse_args();require(args.checkpoint.stat().st_size<=16*1024*1024,'Portable CLI55 file exceeds compact bound')
    current=args.default_checkpoint or args.checkpoint.with_name(DEFAULT['filename'])
    prior=args.default_predecessor or args.checkpoint.with_name(DEFAULT_PRIOR['filename'])
    print(json.dumps(verify(json.loads(args.checkpoint.read_bytes()),current.read_bytes(),prior.read_bytes()),indent=2))


if __name__=='__main__':main()
