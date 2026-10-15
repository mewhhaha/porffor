#!/usr/bin/env python3
"""Verify captured IR diagnostics offline; this does not rerun native code."""
import hashlib,json,re,sys
from pathlib import Path
def require(condition,message):
 if not condition: raise ValueError(message)
def verify(x):
 require(x['schema']=='focused-ir-candidate-native-proof-v1','schema')
 require(x['diagnostic_only'] is True and all(x[k] is False for k in ['applied','stock_cargo_acceptance','whole_ir_suite_executed','whole_candidate_native_acceptance','conformance_acceptance','task_complete']),'scope broadened')
 for k in ['inventory_stdout','focused_stdout']:
  r=x[k];b=r['text'].encode();require(len(b)==r['bytes'] and hashlib.sha256(b).hexdigest()==r['sha256'],'captured stream changed')
 for k in ['actual_inner_receipt','actual_outer_receipt']:
  r=x[k];b=(json.dumps(r['document'],indent=2)+'\n').encode();require(len(b)==r['bytes'] and hashlib.sha256(b).hexdigest()==r['sha256'],'actual owner changed')
 r=x['actual_inner_receipt']['document'];o=x['actual_outer_receipt']['document']
 require(r['complete'] and r['passed'] and r['exit_code']==0 and r['source_unchanged'] and o['complete'] and o['exit_code']==0 and o['source_unchanged'],'native owners not successful')
 require([c['label'] for c in r['commands']]==['native-build','inventory','focused-unit'] and all(c['complete'] and c['exit_code']==0 for c in r['commands']),'actual commands')
 require(r['rustc_invocations']==1 and r['native_codegen_invocations']==1 and r['metadata_only_invocations']==0 and r['adopted_metadata_only_invocations']==1,'invocation accounting')
 require(r['stock_cargo_acceptance'] is False and r['conformance_acceptance'] is False and r['all_ir_unit_tests_executed'] is False,'owner scope')
 ids=[line[:-6] for line in x['inventory_stdout']['text'].splitlines() if line.endswith(': test')]
 old=x['expected_old_inventory'];new=x['expected_new_controls'];existing=x['expected_existing_controls'];focus=x['expected_focused_controls']
 require(len(old)==1536 and len(new)==7 and len(existing)==29 and len(focus)==36,'expected counts')
 require(len(ids)==1543 and len(set(ids))==1543 and set(ids)==set(old)|set(new) and not(set(old)&set(new)),'inventory retention')
 rows=re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)(?:[ \t].*)?$',x['focused_stdout']['text'],re.M)
 require(len(rows)==36 and len({n for n,_ in rows})==36 and {n for n,_ in rows}==set(focus)==set(new)|set(existing) and all(v=='ok' for _,v in rows),'focused result IDs/outcomes')
 require(re.search(r'^test result: ok\. 36 passed; 0 failed; 0 ignored; 0 measured; 1507 filtered out;',x['focused_stdout']['text'],re.M),'literal summary')
 for a in x['selected_ir_aliases']:
  require(all(a['typechecked_input'][k]==a['first_batch_v5_candidate'][k] for k in ['sha256','bytes']),'different candidate')
 require(len(x['selected_ir_aliases'])==8,'source alias scope')
 m=x['metadata_adopted_without_rerun'];require(m['literal_original_owner_passed'] is False and m['literal_original_inner_exit']==m['literal_original_outer_exit']==1 and m['literal_metadata_exit']==0 and m['metadata_reexecuted'] is False and m['native_codegen_previously_executed'] is False,'metadata adoption laundering')
 i=x['tooling_incident'];require(i['literal_exit']==1 and i['literal_source_unchanged'] is False and i['completed_matrix_nodes']==332 and i['total_matrix_nodes']==748 and i['aggregate_execution_ids']==45105 and i['separate_interrupted_checkpoint_ids']==190 and i['union_observed_execution_ids']==45295 and i['full_suite_still_requires_guarded_resume_and_exact_id_audit'] is True,'interruption hidden')
 return {'verified':True,'native_focused_passed':36,'new_controls':7,'existing_controls':29,'old_ids_inventoried':1536,'whole_candidate_acceptance':False,'task_complete':False}
if __name__=='__main__': print(json.dumps(verify(json.loads(Path(sys.argv[1]).read_text()))))
