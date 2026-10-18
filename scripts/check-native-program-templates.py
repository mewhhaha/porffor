#!/usr/bin/env python3
"""Native P transport ownership checks; semantic/compiler tests remain separate."""
from pathlib import Path
import re
import sys
A = 'crates/lila-aot-wasm/src/'
E = 'crates/lila-engine/src/'
T = A + 'native_program_template'
N = E + 'wasm_native_program_template'
M = E + 'memory_module_cache/native_templates'
CAPS = {
    T + '/share_bodies.rs': 130, T + '/share_bodies/mapping.rs': 140,
    T + '/share_bodies/names.rs': 80, T + '/share_bodies/tests.rs': 180,
    T + '/tests/shared_emitted_bodies.rs': 100,
    'crates/lila-engine/tests/aot_native_program_templates/shared_function_bodies.rs': 100,
    A + 'program_literal_operands.rs': 180,
    T + '/literal_operands.rs': 50, T + '/plan/literal_operands.rs': 80,
    T + '/tests/literal_operands.rs': 260,
    A + 'native_program_contexts.rs': 150, A + 'gc_types/native_payload.rs': 100,
    T + '/contexts.rs': 200, T + '/contexts/signatures.rs': 180,
    T + '/tests/emitted_names.rs': 100,
    T + '/payload.rs': 200, T + '/tests/program_owned_payload.rs': 150, T + '/error.rs': 30,
    T + '/plan/indices.rs': 120, T + '/reencode/declarations.rs': 160,
    T + '/reencode/payload.rs': 100,
    T + '.rs': 160, T + '/plan.rs': 260, T + '/plan/sites.rs': 120,
    T + '/references.rs': 110, T + '/reencode.rs': 200,
    T + '/reencode/names.rs': 80, T + '/tests.rs': 300,
    N + '/instance.rs': 120, N + '/initializer.rs': 80,
    N + '/bindings/allocation/payload.rs': 100, N + '/bindings/types.rs': 90,
    N + '/bindings/allocation/tests/payload_tests.rs': 120,
    M + '/instances.rs': 110, M + '/instances/tests.rs': 90,
    'crates/lila-engine/tests/aot_native_program_templates/program_owned_payload.rs': 130,
    N + '.rs': 90, N + '/cache.rs': 50, N + '/bindings.rs': 180,
    N + '/bindings/allocation.rs': 100, N + '/bindings/allocation/tests.rs': 160,
    M + '.rs': 70, M + '/tests.rs': 70,
    'crates/lila-engine/tests/aot_native_program_templates.rs': 110,
    'crates/lila-engine/tests/aot_native_program_templates/catalog_overlays.rs': 60,
}
KINDS = ('function', 'global_script', 'direct_eval')
FILES = tuple(CAPS) + (A + 'lib.rs', E + 'lib.rs', E + 'wasm_native_source_compiler.rs',
    E + 'memory_module_cache.rs') + tuple(E + 'wasm_native_' + k + '_compiler/factory.rs' for k in KINDS)
def compact(text):
    return re.sub(r'\s+', '', re.sub(r'//[^\n]*', '', text))
def check(sources):
    failures = []
    def need(ok, key):
        if not ok: failures.append(key)
    c = {name: compact(text) for name, text in sources.items()}
    owner = c[T + '.rs'] + c[T + '/error.rs']
    need('pubstructNativeProgramTemplate{bytes:Arc<[u8]>,runtime:Arc<RuntimeArtifact>,images:Vec<NativeProgramDataImage>,factories:Vec<NativeProgramArrayFactory>,literal_operands:Vec<NativeProgramLiteralOperand>,payload:Option<NativeProgramPayload>,}' in owner and
         'fnfrom_native(' in owner and 'pubfnfrom_native(' not in owner and
         all('pubfnfor_' + kind + '(artifact:&Runtime' + {'function':'Function','global_script':'GlobalScript','direct_eval':'DirectEval'}[kind] + 'Artifact' in owner for kind in KINDS), 'typed_artifact_only')
    need(sources[T + '/tests.rs'].splitlines()[0] == '#![cfg(test)]' and
         '#[cfg(test)]modtests;' in owner and sources[T + '/tests/literal_operands.rs'].splitlines()[0] == '#![cfg(test)]' and '[path="tests/literal_operands.rs"]modliteral_operand_tests;' in c[T + '/tests.rs'], 'exact_fixture_owner_is_statically_test_only')
    need('runtime:Arc::clone(runtime)' in owner and 'NativeProgramTemplateError(String)' in owner and
         'EmitError::unsupported' not in owner, 'opaque_owner_and_error_domain')
    plan = c[T + '/plan.rs'] + c[T + '/plan/indices.rs']
    need('#[cfg(debug_assertions)]' in plan and
         'wasmparser::Validator::new_with_features(features).validate_all(bytes)' in plan and
         'DataKind::Passive' in plan and 'returnOk(None)' in plan and
         all('Operator::' + op in plan for op in ('MemoryInit', 'DataDrop', 'ArrayInitData')) and
         'nativePmaynothaveastart' in plan and 'if!has_types||!has_imports' in plan, 'checked_fallback_and_no_start')
    need('!ty.composite_type.shared' in plan and 'StorageType::I8' in plan and 'StorageType::I16' in plan and
         'referenced.contains(&original)' in plan and 'self.retained_functions()+self.factories.len()asu32' in plan,
         'packed_types_and_function_map')
    sites = c[T + '/plan/sites.rs']
    need('Operator::ArrayNewData' in sites and 'Operator::ArrayNewFixed' in sites and
         'Operator::I32Const{value}' in sites and '(*valueasu16).to_le_bytes()' in sites and
         'self.sites.insert(*offset,Site{image,factory})' in sites and
         'self.suppressed.contains(position)' in sites, 'only_constant_operands_are_extracted')
    encoder = c[T + '/reencode.rs'] + c[T + '/reencode/declarations.rs'] + c[T + '/reencode/payload.rs'] + c[A + 'gc_types/native_payload.rs']
    need('fnfunction_index(' in encoder and 'fnglobal_index(' in encoder and
         'self.plan.function(index)' in encoder and 'index+self.plan.images.len()asu32' in encoder and
         'self.plan.imported_globals+site.image' in encoder and
         'self.plan.retained_functions()+site.factory' in encoder and
         'self.instruction(other)?' in encoder, 'all_original_instruction_indices_remain_mapped')
    need('types.ty().array(&wasm_encoder::StorageType::I8,false)' in encoder and
         'NativeProgramArrayConstruction::Slice=>vec![ValType::I32,ValType::I32,data]' in encoder and
         'Self::reference(factory.target_array_type)' in encoder and
         'mutable:false,shared:false' in encoder and
         'Instruction::GlobalGet' in encoder and 'Instruction::Call' in encoder,
         'immutable_images_and_typed_allocation_calls')
    references = c[T + '/references.rs']
    names = c[T + '/reencode/names.rs']
    need('self.0.insert(index)' in references and 'ElementKind::Declared' in references and
         'ElementItems::Functions' in references and
         all('Operator::' + op in references for op in ('Call', 'ReturnCall', 'RefFunc')) and
         all(section in references for section in ('TableSectionReader', 'GlobalSectionReader', 'ExportSectionReader', 'ElementSectionReader')) and
         'Name::Function(reader)' in names and 'Name::Local(reader)' in names and
         'Name::Label(reader)' in names and 'Name::Data(_)=>Ok(())' in names,
         'declarations_and_debug_names_follow_map')
    binder = c[N + '.rs'] + c[N + '/instance.rs']
    need('!WasmtimeEngine::same(engine,context.linker.engine())' in binder and
         '!Arc::ptr_eq(template.runtime(),context.runtime())' in binder and
         'Instance::new(&mut*store,&module,&imports)?' in binder and
         '.get(&mut*store,import.module(),import.name())' in binder and
         binder.count('context.check_budget()?') >= 4, 'same_store_engine_runtime_and_budget')
    data = c[N + '/bindings.rs'] + c[N + '/bindings/types.rs']
    allocation = c[N + '/bindings/allocation.rs'] + c[N + '/bindings/allocation/payload.rs']
    need('global.mutability()!=Mutability::Const' in data and
         'array.mutability()!=Mutability::Const' in data and
         'array_reference(result)?' in data and 'FuncType::eq(ty,&function)' in data, 'actual_immutable_image_and_target_types')
    need('Arc::clone(context.execution())' in data and data.count('execution.remaining()') == 2 and
         'allocation::fresh(&mutcaller,inputs,outputs,&target,width,construction)?' in data and
         'cache.entries.len()<8' in data and 'Arc<[u8]>' not in data, 'allocator_cache_captures_no_source_data')
    need('ArrayRef::new_fixed(&mut*store,&pre,&values)?' in allocation and
         'u16::from_le_bytes([low,high])' in allocation and
         '(*offsetasu32,Some(*lengthasu32),*image)' in allocation and
         'checked_sub(offset)' in allocation and 'wasmtime::Trap::ArrayOutOfBounds' in allocation and
         '*output=Val::AnyRef(Some(result.to_anyref()))' in allocation, 'fresh_allocations_exact_bits_and_bounds')
    cache = c[N + '/cache.rs']
    memory = c[M + '.rs']
    need('letmodules=memory_wasm_modules()' in cache and '.get_for_engine(&key,engine)' in cache and
         '.retain_compiled_for_engine(key,module)' in cache and
         'WasmModuleSource::Program(bytes),mode' in cache and
         'WasmtimeEngine::same(entry.module.engine(),engine)' in memory and
         '.checked_sub(removed.image_bytes)' in memory and 'self.retain_compiled(key,module)' in memory,
         'existing_aggregate_cache_and_actual_engine')
    for kind in KINDS:
        factory = c[E + 'wasm_native_' + kind + '_compiler/factory.rs']
        need('NativeProgramTemplate::for_' + kind + '(&artifact)' in factory and
             'crate::wasm_native_program_template::instantiate(&mut*store,engine,context,&template,trace,)?' in factory and
             '!Arc::ptr_eq(artifact.runtime(),context.runtime())' in factory and
             'context.validated_runtime()' in factory and 'EngineError::new(e.to_string())' in factory and
             'compile_wasm_module(' not in factory and factory.count('context.check_budget()?') >= 4,
             'actual_factory_route_' + kind)
    need('modnative_program_template;' in c[A + 'lib.rs'] and
         'modwasm_native_program_template;' in c[E + 'lib.rs'] and
         'modnative_templates;' in c[E + 'memory_module_cache.rs'] and
         'native_compilation_mode:WasmNativeCompilationMode' in c[E + 'wasm_native_source_compiler.rs'], 'private_owner_declarations')
    declarations = c[A + 'program_literal_operands.rs']
    scalar_sites = c[T + '/plan/literal_operands.rs']
    need('value:ProgramLiteralCount' in declarations and 'value:ProgramLiteralIndex' in declarations and
         'word(0)!=VERSION' in declarations and 'prior>=(body,operator)' in declarations and
         'Kind::Count' in declarations and 'Kind::Index' in declarations and
         'literaloperandrecordlengthmismatch' in declarations, 'closed_literal_operand_transport')
    need('Operator::I32Const{value}' in scalar_sites and '*value!=site.expected' in scalar_sites and
         'self.suppressed.contains(offset)' in scalar_sites and 'self.sites.contains_key(offset)' in scalar_sites and
         'site.operatorasusize' in scalar_sites and 'literaloperandbodyisoutofrange' in plan,
         'literal_relocations_check_exact_operators_and_disjoint_data_sites')
    need('section.name()==crate::program_literal_operands::SECTION_NAME' in encoder and
         'self.plan.images.len()asu32+self.plan.literal_operands.len()asu32' in encoder and
         'val_type:ValType::I32,mutable:false,shared:false' in encoder and
         'self.plan.literal_sites.get(&offset)' in encoder,
         'literal_operands_are_immutable_globals_and_marker_is_consumed')
    need(".rsplit('.')" in data and '.literal_operands().get(index)' in data and
         'operand.import_name()==import.name()' in data and
         '!matches!(global.content(),ValType::I32)' in data and 'Val::I32(operand.value())' in data,
         'fresh_exact_scalar_binding_with_constant_time_ordinal_lookup')
    need(sources[T + '/tests/program_owned_payload.rs'].splitlines()[0] == '#![cfg(test)]' and
         '[path="tests/program_owned_payload.rs"]modprogram_owned_payload;' in c[T + '/tests.rs'],
         'owned_payload_binary_fixture_is_statically_test_only')
    contexts = c[T + '/contexts.rs'] + c[T + '/contexts/signatures.rs']
    need('signatures.initializer(bootstrap_body,schema)?' in contexts and
         'signatures.local(indexasu32,body,*local,schema)?' in contexts and
         'Operator::LocalSet' in contexts and 'Operator::LocalTee' in contexts and
         'schema.payload_field' in contexts and 'StorageType::I8' in contexts and
         '!field.mutable' in contexts and 'ordinal>=*readyasusize' in contexts,
         'payload_owner_has_actual_typed_initialized_local_and_bootstrap_witness')
    need('FunctionContext::Program{local,..}' in encoder and
         'native_payload::load(function,*local,contexts.schema)' in encoder and
         'field_index:schema.payload_field' in encoder and
         'Instruction::LocalGet(0)' in encoder and 'LocalGet(local_index+1)' in encoder and
         'self.plan.contexts.is_none()' in encoder,
         'owned_operands_use_actual_P_or_explicit_bootstrap_payload')
    store = c[M + '/instances.rs']
    need('entry.code.as_ref()==code' in store and 'Arc::ptr_eq(&entry.authority,authority)' in store and
         'WasmtimeEngine::same(entry.module.engine(),engine)' in store and
         'module_image_bytes(&module).and_then(|bytes|bytes.checked_add(code.len()))' in store and
         'self.limits.has_image_room(self.retained_bytes,bytes)' in store and
         '.native_template_instances.get(&key,template.bytes(),engine,&context.instance_authority)' in binder and
         'payload.permits_instance_reuse()' in binder and
         'native_template_instances:memory_module_cache::MemoryNativeInstanceCache' in c[E + 'lib.rs'],
         'instance_reuse_is_store_engine_code_authority_and_original_budget_bound')
    need('fnfresh_from_payload<T>(' in allocation and 'directory.image_row(ordinal)' in allocation and
         'directory.image_span(&row)' in allocation and 'NativeProgramPayloadDirectory::from_footer' in allocation and
         'Arc::from' not in allocation and 'image_start+offset+index*unit' in allocation,
         'fresh_source_arrays_use_only_the_received_P_payload_image_bounds')
    for name, maximum in CAPS.items():
        need(len(sources[name].splitlines()) <= maximum, 'owner_cap:' + name)
    return failures

def main():
    root = Path(sys.argv[1]) if len(sys.argv) == 2 else Path.cwd()
    failures = check({name:(root/name).read_text() for name in FILES})
    for failure in failures: print('native program templates: ' + failure, file=sys.stderr)
    if not failures: print('native program template source controls passed')
    return bool(failures)
if __name__ == '__main__': raise SystemExit(main())
