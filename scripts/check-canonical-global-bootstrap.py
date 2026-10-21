#!/usr/bin/env python3
"""Canonical Realm publication stays separate from delayed Script declarations."""
import re,sys
from pathlib import Path

PREFIX='crates/scriptw-ir/src/'
FILES=[PREFIX+'lowering.rs',PREFIX+'lowering/builtin_catalog.rs',PREFIX+'lowering/global_script_prelude.rs',PREFIX+'lowering/prepared_function.rs']

def check(root,overrides=None):
 overrides=overrides or {}
 code={p:re.sub(r'\s+','',overrides[p] if p in overrides else (Path(root)/p).read_text())for p in FILES}
 bad=[]
 def need(ok,label):
  if not ok:bad.append(label)
 def ordered(text,*parts):
  at=0
  for part in parts:
   found=text.find(part,at)
   if found<0:return False
   at=found+len(part)
  return True
 catalog=code[PREFIX+'lowering/builtin_catalog.rs']
 recipe=catalog.split('fncanonical_standard_global_bindings()',1)[-1].split('fnretain_canonical_standard_globals(',1)[0]
 for name,initializer in [('GLOBAL_THIS_NAME','Intrinsic'),('"Infinity"','Infinity'),('"NaN"','NaN'),('"undefined"','Undefined'),('REFLECT_NAME','ReflectObject'),('MATH_NAME','MathObject'),('JSON_NAME','JsonObject'),('ATOMICS_NAME','AtomicsObject'),('TEMPORAL_NAME','TemporalObject'),('INTL_NAME','IntlObject')]:
  need(recipe.count('('+name+',GlobalPropertyInitializerIr::'+initializer+')')==1,'one canonical '+name+' initializer')
 need('StandardBuiltinId::all_globals().into_iter().filter_map('in recipe and 'GlobalPropertyInitializerIr::BuiltinFunction(*builtin)'in recipe,'complete closed builtin catalog')
 need('declarations:GlobalDeclarationSetIr::None'in recipe,'canonical recipe has no source declarations')
 fill=catalog.split('fnretain_canonical_standard_globals(',1)[-1].split('#[cfg(test)]',1)[0]
 need('ifplan.get(&binding.name).is_none(){plan.insert_initial(binding);}'in fill,'fill only missing canonical names without replacing source owners')
 need('builtin_catalog::canonical_standard_global_bindings(),lexical_bindings,'in code[PREFIX+'lowering.rs'],'ordinary Script consumes same canonical recipe')
 entry=code[PREFIX+'lowering/global_script_prelude.rs']
 need(ordered(entry,'global_bindings:std::mem::take(&mutscript.global_bindings)','declarations:std::mem::take(&mutscript.runtime_declarations)','super::builtin_catalog::retain_canonical_standard_globals(&mutscript.global_bindings)','HostBuiltinId::every_realm_globals().chain(script.host_builtins.iter().copied())','script.global_bindings.require_host_global(builtin);','script.strict=false;'),'delayed entry declarations and canonical bootstrap are independent owners')
 prepared=code[PREFIX+'lowering/prepared_function.rs']
 need(ordered(prepared,'fnappend_prepared_unit(','super::builtin_catalog::retain_canonical_standard_globals(&mutscript.global_bindings)','script.functions.append(&mutunit.functions);'),'prepared-only canonical uses reach initial Realm')
 need('script.global_bindings.require_host_global(builtin);'in prepared,'existing host surface admission retained')
 return bad

if __name__=='__main__':
 errors=check(Path(sys.argv[1])if len(sys.argv)>1 else Path.cwd())
 if errors:print('\n'.join(errors),file=sys.stderr)
 raise SystemExit(bool(errors))
