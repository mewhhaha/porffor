#!/usr/bin/env python3
"""Source ownership checks for independent global Script prelude/entry."""
import re,sys
from pathlib import Path
I='crates/scriptw-ir/src/'
A='crates/scriptw-aot-wasm/src/'
E='crates/scriptw-engine/src/'
FILES=[I+x for x in ['ir.rs','prepared_script.rs','lowering.rs','diagnostics.rs','lowering/prepared_script.rs','lowering/module_graph.rs','lowering/global_script_prelude.rs']]+[A+x for x in ['emit.rs','gc_types.rs','gc_types/global_script_prelude.rs','module/compiled_module_package.rs','emit/module_assembly.rs','emit/module_assembly/program_entries.rs','emit/body_compilation.rs','prepared_script.rs','lib.rs']]+[E+x for x in ['lib.rs','prepared_source_catalog.rs','graph_observation.rs','global_script_prelude.rs']]
def check(root, overrides=None):
 overrides=overrides or {};c={p:re.sub(r'\s+','',overrides[p] if p in overrides else (Path(root)/p).read_text()) for p in FILES};bad=[]
 def require(ok,name):
  if not ok:bad.append(name)
 def ordered(s,*parts):
  i=0
  for p in parts:
   k=s.find(p,i)
   if k<0:return False
   i=k+len(p)
  return True
 s=c[I+'prepared_script.rs']
 require('pubstructGlobalScriptEntryIr(PreparedScriptUnit);' in s and 'assert_eq!(unit.kind,PreparedScriptKind::RealmScript);' in s,'opaque global entry role')
 s=c[I+'ir.rs']
 require('pubscript_entry_after_prelude:Option<crate::GlobalScriptEntryIr>' in s and 'self.script_entry_after_prelude.iter().map(|entry|entry.unit())' in s,'one closed source-unit inventory')
 s=c[I+'lowering/global_script_prelude.rs']
 require(ordered(s,'fnretain_global_script_entry(','script.script_entry_after_prelude=Some(crate::GlobalScriptEntryIr::new(PreparedScriptUnit{','kind:PreparedScriptKind::RealmScript,','std::mem::replace(&mutscript.body,','global_bindings:std::mem::take(&mutscript.global_bindings)','declarations:std::mem::take(&mutscript.runtime_declarations)','script.global_bindings.require_host_global(*builtin);','script.strict=false;'),'separate test declaration/body owner and bootstrap host surface')
 require('compiled.diagnostics.drain(..).map(IrDiagnostic::in_global_prelude)' in c[I+'lowering/prepared_script.rs'],'typed lowering setup provenance')
 s=c[I+'lowering/global_script_prelude.rs']
 require(ordered(s,'pubfnlower_source_with_prelude(','ScriptInstantiation::GlobalScriptAfterPrelude','retain_global_script_entry(','prepared_script::compile_module_prelude('),'retained independent Script lowering')
 require('ScriptInstantiation::ModuleAfterGlobalScript|ScriptInstantiation::GlobalScriptAfterPrelude' in c[I+'lowering.rs'] and 'lowerer.invalidate_unknown_user_code_effect_facts();' in c[I+'lowering.rs'],'prior user effects invalidate analysis facts')
 s=c[I+'lowering/module_graph.rs']
 require('prelude.is_some()&&entry_is_script{ScriptInstantiation::GlobalScriptAfterPrelude' in s and 'super::global_script_prelude::retain_global_script_entry(&mutprogram,&mutallocations)' in s,'Script graph retains Script role and independent entry')
 s=c[A+'emit/body_compilation.rs']
 require(ordered(s,'phase.enter(&mutfunction);','self.emit_module_prelude(id,&mutfunction)?;','phase.completed(&mutfunction);','self.emit_script_entry_after_prelude(id,&mutfunction)?;','self.compile_block_contents(self.body,&mutfunction)?;'),'actual harness before test evaluation')
 s=c[A+'prepared_script.rs']
 require('enumGlobalScriptResultUse{Discard,Retain,}' in s and 'GlobalScriptResultUse::Retain' in s and 'ifmatches!(result_use,GlobalScriptResultUse::Discard)' in s and 'execution.restore(self,function);self.completion().copy_from(&pending,function);' in s,'same Realm restoration and actual entry completion')
 require('pubconstGLOBAL_SCRIPT_PRELUDE_PHASE_EXPORT:&str="scriptw.global_prelude_phase";' in c[A+'lib.rs'],'one declaration-owned phase export')
 s=c[A+'gc_types/global_script_prelude.rs']
 require('letindex=globals.len();' in s and 'Self(index)' in s and s.count('GlobalSet(self.0)') == 2 and 'I32Const(1)' in s and 'I32Const(2)' in s,'declaration-owned P phase type and transitions')
 s=c[A+'gc_types.rs']
 require('self.section.types(self.runtime_schema.module_guards.first)' in s and 'global_prelude_phase:None,' in s and 'GlobalScriptPreludePhase::declare(&mutself.section)' in s,'P scalar suffix does not alter shared R schema prefix')
 s=c[A+'emit/module_assembly.rs']
 require('.with_global_prelude(compile_program&&script.module_prelude.is_some())' in s and 'module_package.global_prelude_phase(),' in s,'phase only belongs to an actual program with prelude')
 s=c[A+'emit/module_assembly/program_entries.rs']
 require(ordered(s,'ifletSome(main)=self.main{','ifletSome(phase)=global_prelude{phase.export(exports);}'),'phase export only in P main owner')
 s=c[E+'global_script_prelude.rs']
 require('scriptw_aot_wasm::GLOBAL_SCRIPT_PRELUDE_PHASE_EXPORT' in s and 'WasmtimeVal::I32(0|2)=>Ok(false)' in s and 'WasmtimeVal::I32(1)=>Ok(true)' in s and 'Ok(error.in_global_prelude())' in s,'typed host phase projection and error provenance')
 s=c[E+'lib.rs']
 require(ordered(s,'letglobal_prelude_failed=global_script_prelude::failed(','error.in_global_prelude()','ifglobal_prelude_failed{returnErr(global_script_prelude::completion_error(','letmodule_status=read_module_entry_status('),'setup failure precedes module/graph/observed completion matching')
 require('pubconstfnis_global_prelude_failure(&self)->bool{self.global_prelude_failure}' in s and 'global_prelude_failure:diagnostic.is_global_prelude_failure()' in s and 'result.global_prelude_failure=error.is_global_prelude_failure()' in s,'checked error origin independent of classification')
 require('EngineError::from_parse_error(error).in_global_prelude()' in s and 'aModulepreluderequiresaModuleentry' not in s,'independent parse admission for both goals')
 require('letworker_source=source;' in s and 'worker_options.module_prelude=Some(self.prelude.to_string());' in s and 'format!("{}\\n{}",self.prelude,source)' not in s,'workers retain exact source and independent harness')
 require('scriptw_spec_exec::execute_script_with_prelude(source,filename,module_loading_policy,module_prelude,' in s and 'scriptw_spec_exec::observe_script_with_prelude(source,filename,module_loading_policy,module_prelude,' in s,'legacy and observed oracle paths keep separate harness')
 s=c[E+'prepared_source_catalog.rs']
 require('session.lower_source_with_prelude(source,prelude,options.host_surface_policy)' in s,'actual catalog consumes independent parsed harness')
 s=c[E+'graph_observation.rs']
 require('scriptw_spec_exec::observe_script_graph_with_prelude(source,options.filename.as_deref(),options.module_loading_policy,options.module_prelude.as_deref(),' in s,'graph oracle path keeps separate harness')
 return bad
if __name__=='__main__':
 failures=check(Path(sys.argv[1]) if len(sys.argv)>1 else Path.cwd())
 if failures: print('\n'.join(failures),file=sys.stderr)
 raise SystemExit(bool(failures))
