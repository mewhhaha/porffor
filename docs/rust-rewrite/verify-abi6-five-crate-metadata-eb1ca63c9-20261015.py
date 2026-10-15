#!/usr/bin/env python3
"""Check captured metadata evidence offline; performs no compilation or tests."""
import hashlib,json,re,sys
from pathlib import Path
def require(c,m):
 if not c:raise ValueError(m)
def verify(x):
 require(x['schema']=='unapplied-abi6-five-crate-metadata-proof-v1' and x['diagnostic_only'] is True,'schema/scope')
 require(all(x[k] is False for k in ('applied','native_acceptance','stock_cargo_acceptance','conformance_acceptance','task_complete')),'scope broadened')
 docs=x['documents']
 for k,v in docs.items():
  b=v['text'].encode();require(len(b)==v['bytes'] and hashlib.sha256(b).hexdigest()==v['sha256'],'captured bytes changed: '+k)
 r=json.loads(docs['actual']['text']);o=json.loads(docs['outer']['text']);old=json.loads(docs['original_red']['text']);p=json.loads(docs['run2_plan']['text'])
 require(r['complete'] is r['passed'] is r['source_unchanged'] is True and r['exit_code']==0 and o['complete'] and o['source_unchanged'] and o['exit_code']==0,'actual owner failure')
 require(r['source_before']==r['source_after']==p['source_fences'] and r['protected_before']==r['protected_after'],'input/source guard')
 require(all(r[k]==0 for k in ('native_codegen_invocations','cargo_invocations','native_tests_executed')) and r['r_rebuilt'] is False and r['whole_abi6_native_acceptance'] is False,'scope accounting')
 names=['lila_ir','lila_aot_wasm','lila_backend_c','lila_backend_native','lila_engine']
 require([c['crate'] for c in r['commands']]==names and [c['crate'] for c in r['completed_metadata']]==names and r['rustc_metadata_successes']==5,'phase inventory')
 for c,m,s in zip(r['commands'],r['completed_metadata'],p['stages']):
  require(c['complete'] and c['exit_code']==0 and c['compiler_exec_argv']==s['argv'],'literal compiler command')
  require(c['cwd']==p['temporary'] and c['environment_overrides']['TMPDIR']==p['temporary'] and all(not Path(c['cwd']).is_relative_to(z) for z in p['source_fences']),'compiler cwd')
  a=s['rustc_argv'];require([z for z in a if z.startswith('--emit=')]==['--emit=metadata='+s['output']+',dep-info='+s['dep_info']] and '--test' not in a,'native or test emission')
  require(m['metadata']['bytes']>0 and m['source_coverage']['all_required_present'] and m['source_coverage']['no_foreign_inputs'],'output/source coverage')
  child=json.loads(docs[c['crate']+'_child']['text']);require(child['pid']==c['pid'] and child['start_tick']==c['start_tick'] and child['cwd']==c['cwd'] and child['tmpdir']==c['cwd'] and child['affinity']==[3] and child['exec_argv']==s['argv'],'actual private child gate')
 require(old['complete'] and old['passed'] is False and old['exit_code']==1 and old['source_unchanged'] and old['commands'][-1]['crate']=='lila_aot_wasm' and old['commands'][-1]['exit_code']==1,'original RED hidden')
 require(len(re.findall(r'^error\[',docs['original_aot_stderr']['text'],re.M))==12,'original diagnostics')
 g=json.loads(docs['type_fixed_boundary_guard']['text']);require(g['complete'] and g['passed'] and g['exit_code']==0 and g['materialized_files_unchanged'] and docs['type_fixed_boundary_stdout']['text']=='check-module-boundaries: ok\n' and docs['type_fixed_boundary_stderr']['text']=='','boundary guard')
 return {'verified':True,'metadata_crates':5,'compiler_seconds':sum(c['seconds'] for c in r['commands']),'total_seconds':r['seconds'],'native_acceptance':False,'task_complete':False}
if __name__=='__main__':print(json.dumps(verify(json.loads(Path(sys.argv[1]).read_text()))))
