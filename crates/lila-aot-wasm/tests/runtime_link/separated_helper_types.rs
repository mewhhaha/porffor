use std::collections::{BTreeMap, BTreeSet};

use super::linked_bodies::module_bodies;
use lila_aot_wasm::{WasmArtifact, RUNTIME_IMPORT_NAMESPACE};
use lila_front::{parse, ParseOptions};
use wasmparser::{
    CompositeInnerType, ExternalKind, GlobalType, HeapType, Operator, Parser, Payload, SubType,
    TypeRef, ValType, Validator, WasmFeatures,
};

#[derive(Default)]
struct ModuleShape {
    groups: Vec<Vec<SubType>>,
    functions: Vec<u32>,
    globals: Vec<GlobalType>,
    runtime_imports: BTreeMap<String, TypeRef>,
    exports: BTreeMap<String, (ExternalKind, u32)>,
    reference_calls: BTreeSet<u32>,
    global_reference_calls: Vec<(u32, u32)>,
}

fn shape(bytes: &[u8]) -> ModuleShape {
    Validator::new_with_features(WasmFeatures::all())
        .validate_all(bytes)
        .expect("the actual R/P module validates");
    let mut shape = ModuleShape::default();
    for payload in Parser::new(0).parse_all(bytes) {
        match payload.expect("module decodes") {
            Payload::TypeSection(reader) => {
                for group in reader {
                    shape.groups.push(group.unwrap().types().cloned().collect());
                }
            }
            Payload::ImportSection(reader) => {
                for import in reader.into_imports() {
                    let import = import.unwrap();
                    match import.ty {
                        TypeRef::Func(index) | TypeRef::FuncExact(index) => {
                            shape.functions.push(index)
                        }
                        TypeRef::Global(global) => shape.globals.push(global),
                        _ => {}
                    }
                    if import.module == RUNTIME_IMPORT_NAMESPACE {
                        assert!(shape
                            .runtime_imports
                            .insert(import.name.to_owned(), import.ty)
                            .is_none());
                    }
                }
            }
            Payload::FunctionSection(reader) => shape
                .functions
                .extend(reader.into_iter().map(Result::unwrap)),
            Payload::GlobalSection(reader) => shape
                .globals
                .extend(reader.into_iter().map(|global| global.unwrap().ty)),
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export.unwrap();
                    shape
                        .exports
                        .insert(export.name.to_owned(), (export.kind, export.index));
                }
            }
            Payload::CodeSectionEntry(body) => {
                let mut previous = None;
                for operator in body.get_operators_reader().unwrap() {
                    let operator = operator.unwrap();
                    if let Operator::CallRef { type_index }
                    | Operator::ReturnCallRef { type_index } = &operator
                    {
                        shape.reference_calls.insert(*type_index);
                        if let Some(Operator::GlobalGet { global_index }) = previous {
                            shape
                                .global_reference_calls
                                .push((global_index, *type_index));
                        }
                    }
                    previous = Some(operator);
                }
            }
            _ => {}
        }
    }
    shape
}

fn emit(source: &str) -> WasmArtifact {
    let parsed = parse(source, ParseOptions::script()).unwrap();
    let program = lila_ir::lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    lila_aot_wasm::emit(&program).expect("source emits through the product compiler")
}

#[test]
fn direct_array_program_helpers_preserve_runtime_bytes_types_imports_and_hook_call_abi() {
    std::thread::Builder::new()
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let plain = emit("var record = { value: 1 }; record.value;");
            let small_arrays =
                emit("function build() { return [['slot',1],['other',2]]; } build();");
            // Each pure nested pair contributes four recipe nodes; 32 pairs plus
            // the outer Begin/End exceed the unchanged 128-node admission floor.
            let pairs = (0..32)
                .map(|index| match index {
                    0 => "['slot',1]".to_owned(),
                    1 => "['other',2]".to_owned(),
                    _ => format!("['slot',{}]", index + 1),
                })
                .collect::<Vec<_>>()
                .join(",");
            let arrays = emit(&format!(
                "function build() {{ return [{pairs}]; }} build();"
            ));
            let runtime = plain.runtime().expect("ordinary object uses linked R");
            let array_runtime = arrays.runtime().expect("array program uses linked R");
            let small_array_runtime = small_arrays.runtime().expect("small array uses linked R");
            assert_eq!(runtime.key(), small_array_runtime.key());
            assert_eq!(runtime.bytes(), small_array_runtime.bytes());
            assert_eq!(runtime.key(), array_runtime.key());
            assert_eq!(
                runtime.bytes(),
                array_runtime.bytes(),
                "using a direct program helper cannot change R bytes"
            );
            let core = shape(runtime.bytes());
            let core_type_count: usize = core.groups.iter().map(Vec::len).sum();
            assert!(!core.groups.is_empty());
            assert!(
                core.groups.last().unwrap().len() > 1,
                "R ends at its actual GC recursion group"
            );
            assert!(
                !core.reference_calls.is_empty(),
                "actual R still invokes its typed hooks"
            );
            assert!(core
                .reference_calls
                .iter()
                .all(|index| (*index as usize) < core_type_count));
            let core_types = core.groups.iter().flatten().collect::<Vec<_>>();
            let hook_types = core
                .globals
                .iter()
                .enumerate()
                .filter_map(|(global_index, global)| {
                    let ValType::Ref(reference) = global.content_type else {
                        return None;
                    };
                    let HeapType::Concrete(index) = reference.heap_type() else {
                        return None;
                    };
                    let index = index
                        .as_module_index()
                        .expect("encoded core reference uses its module index");
                    matches!(
                        &core_types[index as usize].composite_type.inner,
                        CompositeInnerType::Func(_)
                    )
                    .then_some((global_index as u32, index))
                })
                .collect::<BTreeMap<_, _>>();
            assert_eq!(
                hook_types.len(),
                5,
                "all five typed hook globals remain present"
            );
            assert!(
                !core.global_reference_calls.is_empty(),
                "R actually calls its typed program hooks"
            );
            for &(global_index, type_index) in &core.global_reference_calls {
                assert_eq!(
                    hook_types.get(&global_index),
                    Some(&type_index),
                    "the actual GlobalGet operand and call_ref use the same core hook ABI"
                );
            }
            assert!(
                module_bodies(runtime.bytes())
                    .iter()
                    .all(|body| body.name != "helper::compact_array_literal"),
                "the direct array helper body belongs only to P"
            );

            for artifact in [&plain, &small_arrays, &arrays] {
                let program = shape(&artifact.bytes);
                assert_eq!(
                    &program.groups[..core.groups.len()],
                    &core.groups,
                    "core canonical recursion groups must match before import linking"
                );
                assert!(program.groups.len() > core.groups.len());
                assert!(
                    program.groups[core.groups.len()..]
                        .iter()
                        .all(|group| group.len() == 1),
                    "program signatures are independent suffix types"
                );
                assert!(!program.runtime_imports.is_empty());
                for (name, import) in &program.runtime_imports {
                    let &(kind, index) = core
                        .exports
                        .get(name)
                        .expect("P imports an actual R export");
                    match import {
                        TypeRef::Func(ty) | TypeRef::FuncExact(ty) => {
                            assert_eq!(kind, ExternalKind::Func);
                            assert_eq!(*ty, core.functions[index as usize], "{name} function ABI");
                            assert!((*ty as usize) < core_type_count);
                        }
                        TypeRef::Global(global) => {
                            assert_eq!(kind, ExternalKind::Global);
                            assert_eq!(*global, core.globals[index as usize], "{name} global ABI");
                        }
                        _ => panic!("unexpected runtime import {name}"),
                    }
                }
                let array_helper = module_bodies(&artifact.bytes)
                    .into_iter()
                    .find(|body| body.name == "helper::compact_array_literal")
                    .expect("P keeps its dense direct array helper slot");
                assert!(
                    (program.functions[array_helper.index as usize] as usize) >= core_type_count,
                    "the actual array helper declaration consumes a suffix signature"
                );
                assert!(
                    program
                        .reference_calls
                        .iter()
                        .all(|index| (*index as usize) < core_type_count),
                    "program call_ref roles and hooks consume stable core signatures"
                );
            }
            let plain_body = module_bodies(&plain.bytes)
                .into_iter()
                .find(|body| body.name == "helper::compact_array_literal")
                .unwrap()
                .bytes();
            let small_array_body = module_bodies(&small_arrays.bytes)
                .into_iter()
                .find(|body| body.name == "helper::compact_array_literal")
                .unwrap()
                .bytes();
            assert_eq!(
                small_array_body, plain_body,
                "the original two-pair input stays below recipe admission"
            );
            let array_body = module_bodies(&arrays.bytes)
                .into_iter()
                .find(|body| body.name == "helper::compact_array_literal")
                .unwrap()
                .bytes();
            assert_eq!(
                plain_body, 3,
                "without recipes the helper contains zero locals, unreachable and end"
            );
            assert!(
                array_body > plain_body,
                "the paired array program uses the real recipe constructor"
            );
        })
        .unwrap()
        .join()
        .unwrap();
}
