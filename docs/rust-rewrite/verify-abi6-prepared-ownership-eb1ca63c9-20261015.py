#!/usr/bin/env python3
"""Verify the recorded prepared ownership diagnostic offline; runs no native code."""
import hashlib,json,sys
from pathlib import Path
def require(c,m):
 if not c:raise ValueError(m)
def verify(x):
 require(x['schema']=='unapplied-abi6-prepared-ownership-proof-v1' and x['diagnostic_only'] is True,'scope')
 require(all(x[k] is False for k in ('applied','native_function_host_miss_acceptance','ordinary_cargo_acceptance','conformance_acceptance','task_complete')),'scope broadened')
 d=x['documents']
 for k,v in d.items():
  b=v['text'].encode();require(len(b)==v['bytes'] and hashlib.sha256(b).hexdigest()==v['sha256'],'changed captured bytes: '+k)
 r=json.loads(d['run3_receipt.json']['text']);p=json.loads(d['run3_plan']['text']);n=json.loads(d['native_libraries']['text'])
 require(r['complete'] is r['passed'] is r['source_unchanged'] is True and r['exit_code']==0,'owner failed')
 require(r['source_before']==r['source_after'] and r['protected_before']==r['protected_after'],'source/protected inputs')
 require(r['compiler_attempts']==r['probe_attempts']==1 and r['cargo_invocations']==0 and r['whole_ABI6_product_acceptance'] is False,'attempt/scope accounting')
 require(n['complete'] and n['passed'] and n['source_unchanged'] and r['native_owner']['sha256']==d['native_libraries']['sha256'],'actual fresh native libraries')
 require([c['phase'] for c in r['commands']]==['compile','run'] and all(c['complete'] and c['exit_code']==0 for c in r['commands']),'commands')
 stdout=d['run3_run.stdout']['text'];require(stdout.startswith('fresh-R-validated bytes=40444161\n'),'fresh R validation')
 expected={f'case-passed conflict={c} order={o} outer=17 transport=42 old-P=17 native-misses=0' for c in ('false','true') for o in ('P1First','P2First')}
 require({z for z in stdout.splitlines() if z.startswith('case-passed ')}==expected and stdout.count('case-passed ')==4 and stdout.count('reentrant-typed-GC-call=42')==4,'four exact GC/reentry cases')
 require('no native-miss/product/conformance claim' in stdout,'diagnostic scope disclosure')
 for i in (1,2):
  old=json.loads(d[f'run{i}_receipt.json']['text']);require(old['complete'] and old['passed'] is False and old['exit_code']==1 and old['source_unchanged'],'prior diagnostic RED hidden')
 require('unrooted' in d['run2_run.stderr']['text'] and len(json.loads(d['run1_receipt.json']['text'])['commands'])==1,'original failure evidence')
 o=json.loads(d['outer3']['text']);require(o['complete'] and o['source_unchanged'] and o['exit_code']==0,'outer owner')
 return {'verified':True,'prepared_cases':4,'native_misses':0,'compiler_seconds':r['commands'][0]['seconds'],'native_run_seconds':r['commands'][1]['seconds'],'total_seconds':r['seconds'],'whole_product_acceptance':False}
if __name__=='__main__':print(json.dumps(verify(json.loads(Path(sys.argv[1]).read_text()))))
