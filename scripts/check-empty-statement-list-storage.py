#!/usr/bin/env python3
"""Inert lowered entries never occupy the statement-list IR buffers."""
import re,sys
from pathlib import Path
sys.dont_write_bytecode=True
IR='crates/lila-ir/'
ENGINE='crates/lila-engine/src/native_direct_eval_compiler_tests/empty_blocks.rs'
FILES=[IR+'src/lowering.rs',IR+'src/lowering/async_disposable.rs',IR+'src/lowering/statement.rs',IR+'tests/direct_eval_environment.rs',IR+'tests/module_instantiation.rs',ENGINE]
def check(root,overrides=None):
 overrides=overrides or {}
 code={p:re.sub(r'\s+','',overrides[p]if p in overrides else(Path(root)/p).read_text())for p in FILES}
 bad=[]
 def need(ok,label):
  if not ok:bad.append(label)
 lower=code[FILES[0]];helper=code[FILES[1]];statement=code[FILES[2]]
 method=helper.split('fninto_statement_list_entry(self)->Option<Self>{',1)
 need(len(method)==2,'typed post-lowering statement-list entry owner')
 if len(method)==2:
  body=method[1].split('///Lowering-onlyclassification',1)[0]
  need('Self::Statement{statement:StatementIr::Empty,..}=>None,'in body,'only the exact bare Empty IR is discarded')
  need('item@Self::Statement{..}|item@Self::SyncDisposableScope{..}|item@Self::AsyncDisposableScope(_)=>Some(item),'in body,'all nonempty statements and both resource owners retained')
  need(body.count('=>None')==1 and 'if'not in body,'no source/AST/value/content shortcut')
 need(lower.count('.into_statement_list_entry()')==2,'shared list and switch both consume compact entries')
 shared=lower.split('fnlower_statement_list_item_after_hoisting(',1)[-1].split('fnlower_statement_list_item(',1)[0]
 need(shared.endswith('self.lower_statement_list_item(item,scope).into_statement_list_entry()}'),'filter follows ordinary lowering in root/block/function path')
 switch=lower.split('fnlower_statement_items_without_function_initialization(',1)
 need(len(switch)==2,'actual CaseBlock statement-list owner')
 if len(switch)==2:
  body=switch[1].split('fnprepare_annex_b_function_bindings(',1)[0]
  need('letSome(item)=self.lower_statement_list_item(item,scope).into_statement_list_entry()else{continue;};matchitem{'in body,'switch filters lowered entries before retaining statements')
 need('self.lower_module_instantiation_boundary(statement)'in statement and statement.find('self.lower_module_instantiation_boundary(statement)')<statement.find('letmutlowered=matchstatement'),'module lifecycle marker is resolved before any bare-Empty filter')
 need('Statement::Block(block)ifblock.statement_list().statements().is_empty()=>'in statement,'existing zero-scope empty Block lowering remains')
 tests=code[FILES[3]]
 for text,label in [('forcountin[1,128,4096]','growing source list lengths'),('unit.body.statements.is_empty()','prepared eval retains zero inert IR'),('fninert_statement_lists_retain_constant_ir_across_root_function_block_and_switch()','all ordinary accumulation contexts'),('cases[0].body.statements.is_empty()','case body remains compact'),('block.lexical_environment.as_ref().unwrap()','nonempty lexical environment survives')]:need(text in tests,label)
 modules=code[FILES[4]]
 need('"{};importdefer*asfirst' in modules and '"{};exportconst{value}'in modules,'synchronous module golden controls contain inert source entries')
 need('"{};importdefer*asns' in modules and '};await0;'in modules,'async module golden controls contain inert source entries')
 need('assert_eq!(suspension.suspension_points.len(),1)'in modules and 'assert!(contains_boundary(&owner.body.statements))'in modules,'existing exact private lifecycle proofs retained')
 engine=code[ENGINE]
 for text in ['"1;;{};"','"1;varretained;{};"','"varretained;{};retained;"','"letlexical;{};lexical;"','"1;switch(1){case1:{};}"','usingvalue={[Symbol.dispose](){disposed++;}}','(6,6,0)']:
  need(text in engine,'native completion/declaration/resource control '+text)
 need(len((overrides[ENGINE]if ENGINE in overrides else(Path(root)/ENGINE).read_text()).splitlines())<=100,'existing native control owner cap unchanged')
 return bad
if __name__=='__main__':
 errors=check(Path(sys.argv[1])if len(sys.argv)>1 else Path.cwd())
 if errors:print('\n'.join(errors),file=sys.stderr)
 raise SystemExit(bool(errors))
