#![cfg(test)]

use crate::gc_types::*;
use crate::Function;
use crate::{code_sink::LocalDeclarations, module::StaticSignature};
use wasm_encoder::{CodeSection, FunctionSection, Instruction, Module};
use wasmparser::{Operator, Parser, Payload, Validator, WasmFeatures};

fn module_with_body(runtime: &FinalizedRuntimeModule, function: Function) -> Vec<u8> {
    let mut functions = FunctionSection::new();
    functions.function(StaticSignature::HostCollectGc.type_index());
    let mut code = CodeSection::new();
    code.function(&function.into_body());
    let mut module = Module::new();
    module.section(runtime.types());
    module.section(&functions);
    module.section(&code);
    let bytes = module.finish();
    Validator::new_with_features(WasmFeatures::all())
        .validate_all(&bytes)
        .expect("typed-root body must validate against the actual recursive schema");
    bytes
}

#[test]
fn typed_roots_validate_nullable_clears_and_reuse_only_the_same_heap_type() {
    let runtime = RuntimeModuleTypes::register(crate::runtime_artifact::ModuleKind::Runtime(
        crate::RuntimeSourceCompilationProfile::Portable,
    ))
    .finalize_globals(GlobalLedger::new(), false, 0);
    let schema = runtime.globals().schema();
    let mut function = Function::new(LocalDeclarations::from_types([]));
    let length = schema.reserve_i32_local(&mut function);
    length.set_constant(0, &mut function);
    let array = schema
        .reserve_gc_local::<CodeUnitArray, NonNullable>(&mut function)
        .initialize(
            schema
                .array_type::<CodeUnitArray>()
                .filled(GcOperand::i32(0), length, &mut function),
            &mut function,
        );
    let live_local_count = function.allocated_local_count();
    let nullable = schema
        .reserve_gc_local::<Environment, Nullable>(&mut function)
        .initialize_null(schema, &mut function);
    assert_eq!(
        function.allocated_local_count(),
        live_local_count + 1,
        "different live heap types cannot alias"
    );
    array.load(schema, &mut function);
    function.instruction(&Instruction::Drop);
    nullable.load(schema, &mut function);
    function.instruction(&Instruction::RefIsNull);
    function.instruction(&Instruction::Drop);
    nullable.clear(&mut function);
    array.clear(&mut function);
    let construction = StringConstruction::allocate(
        schema,
        schema.reserve_gc_local::<CodeUnitArray, NonNullable>(&mut function),
        length,
        &mut function,
    );
    let _ = construction.publish(schema, &mut function);
    function.instruction(&Instruction::Drop);
    let reused = schema
        .reserve_gc_local::<CodeUnitArray, Nullable>(&mut function)
        .initialize_null(schema, &mut function);
    assert_eq!(
        function.allocated_local_count(),
        live_local_count + 1,
        "logical nullability can reuse its nullable declaration"
    );
    reused.clear(&mut function);
    schema.release_i32_local(length, &mut function);
    function.instruction(&Instruction::End);
    let bytes = module_with_body(&runtime, function);
    let body = Parser::new(0)
        .parse_all(&bytes)
        .find_map(|payload| match payload.unwrap() {
            Payload::CodeSectionEntry(body) => Some(body),
            _ => None,
        })
        .unwrap();
    let roots = body
        .get_locals_reader()
        .unwrap()
        .into_iter()
        .map(Result::unwrap)
        .filter_map(|(count, ty)| match ty {
            wasmparser::ValType::Ref(reference) => Some((count, reference)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), 2);
    assert!(roots.iter().all(|(count, reference)| *count == 1
        && reference.is_nullable()
        && matches!(reference.heap_type(), wasmparser::HeapType::Concrete(_))));
    let mut refinements = 0;
    for operator in body.get_operators_reader().unwrap() {
        match operator.unwrap() {
            Operator::RefAsNonNull => refinements += 1,
            Operator::RefCastNullable { .. } | Operator::RefCastNonNull { .. } => {
                panic!("known roots must not repeat a heap-type check")
            }
            _ => {}
        }
    }
    assert_eq!(
        refinements, 2,
        "both non-null loads retain their null check"
    );
}

#[test]
fn dynamic_values_retain_their_checked_heap_cast() {
    let runtime = RuntimeModuleTypes::register(crate::runtime_artifact::ModuleKind::Runtime(
        crate::RuntimeSourceCompilationProfile::Portable,
    ))
    .finalize_globals(GlobalLedger::new(), false, 0);
    let schema = runtime.globals().schema();
    let mut function = Function::new(LocalDeclarations::from_types([]));
    let value = schema.reserve_value_local(&mut function);
    value.set_undefined(&mut function);
    let _ = value.cast_reference::<StringValue>(schema, &mut function);
    function.instruction(&Instruction::Drop);
    value.clear(&mut function);
    function.instruction(&Instruction::End);
    let bytes = module_with_body(&runtime, function);
    let body = Parser::new(0)
        .parse_all(&bytes)
        .find_map(|payload| match payload.unwrap() {
            Payload::CodeSectionEntry(body) => Some(body),
            _ => None,
        })
        .unwrap();
    assert!(body
        .get_locals_reader()
        .unwrap()
        .into_iter()
        .map(Result::unwrap)
        .any(|(_, ty)| ty == wasmparser::ValType::Ref(wasmparser::RefType::EQREF)));
    assert!(body
        .get_operators_reader()
        .unwrap()
        .into_iter()
        .map(Result::unwrap)
        .any(|operator| matches!(operator, Operator::RefCastNonNull { .. })));
}
