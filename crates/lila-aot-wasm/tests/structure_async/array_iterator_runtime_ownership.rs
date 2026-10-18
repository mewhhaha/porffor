//! Check the actual P-to-R call edges and the called kernels' record operations.

use lila_aot_wasm::WasmArtifact;
use std::collections::BTreeMap;
use wasmparser::{
    CompositeInnerType, ExternalKind, HeapType, KnownCustom, Name, Operator, Parser, Payload,
    StorageType, TypeRef, ValType,
};

pub(super) fn called_iterator_operations(
    artifact: &WasmArtifact,
    program_cell: u32,
    program_record: u32,
) -> ([usize; 2], usize) {
    let runtime = artifact
        .runtime()
        .expect("array patterns link their shared runtime");
    let mut structures = BTreeMap::new();
    let mut signatures = BTreeMap::new();
    let mut definitions = Vec::new();
    let mut bodies = Vec::new();
    let mut names = BTreeMap::new();
    let mut exports = Vec::new();
    let mut imported_functions = 0u32;
    let mut next_type = 0;
    for payload in Parser::new(0).parse_all(runtime.bytes()) {
        match payload.expect("array-pattern runtime decodes") {
            Payload::TypeSection(reader) => {
                for group in reader {
                    for ty in group.expect("runtime type group decodes").types() {
                        match &ty.composite_type.inner {
                            CompositeInnerType::Struct(structure) => {
                                structures.insert(next_type, structure.clone());
                            }
                            CompositeInnerType::Func(signature) => {
                                signatures.insert(next_type, signature.clone());
                            }
                            _ => {}
                        }
                        next_type += 1;
                    }
                }
            }
            Payload::ImportSection(reader) => {
                for import in reader.into_imports() {
                    if matches!(
                        import.expect("runtime import decodes").ty,
                        TypeRef::Func(_) | TypeRef::FuncExact(_)
                    ) {
                        imported_functions += 1;
                    }
                }
            }
            Payload::FunctionSection(reader) => {
                definitions.extend(
                    reader
                        .into_iter()
                        .map(|row| row.expect("runtime function type decodes")),
                );
            }
            Payload::CodeSectionEntry(body) => bodies.push(body),
            Payload::ExportSection(reader) => {
                exports.extend(
                    reader
                        .into_iter()
                        .map(|row| row.expect("runtime export decodes")),
                );
            }
            Payload::CustomSection(section) => {
                if let KnownCustom::Name(subsections) = section.as_known() {
                    for subsection in subsections {
                        if let Name::Function(map) = subsection.expect("runtime names decode") {
                            for naming in map {
                                let naming = naming.expect("runtime function name decodes");
                                assert!(names.insert(naming.index, naming.name).is_none());
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    assert_eq!(definitions.len(), bodies.len());
    let mut imports = Vec::new();
    let mut resumed_calls = Vec::new();
    let mut program_signatures = BTreeMap::new();
    let mut next_type = 0;
    for payload in Parser::new(0).parse_all(&artifact.bytes) {
        match payload.expect("array-pattern program decodes") {
            Payload::TypeSection(reader) => {
                for group in reader {
                    for ty in group.expect("program type group decodes").types() {
                        if let CompositeInnerType::Func(signature) = &ty.composite_type.inner {
                            program_signatures.insert(next_type, signature.clone());
                        }
                        next_type += 1;
                    }
                }
            }
            Payload::ImportSection(reader) => {
                for import in reader.into_imports() {
                    let import = import.expect("program import decodes");
                    if let TypeRef::Func(signature) | TypeRef::FuncExact(signature) = import.ty {
                        imports.push((import.module, import.name, signature));
                    }
                }
            }
            Payload::CodeSectionEntry(body) => {
                let mut calls = Vec::new();
                let mut reloads_iterator = false;
                for operator in body.get_operators_reader().expect("program body opens") {
                    match operator.expect("program operator decodes") {
                        Operator::Call { function_index }
                        | Operator::ReturnCall { function_index } => calls.push(function_index),
                        Operator::StructGet {
                            struct_type_index,
                            field_index: 11,
                        } if struct_type_index == program_cell => reloads_iterator = true,
                        _ => {}
                    }
                }
                if reloads_iterator {
                    resumed_calls.extend(calls);
                }
            }
            _ => {}
        }
    }
    let mut cached_fields = [0, 0];
    let mut done_writes = 0;
    let mut shared_record = None;
    for (name, reads_next) in [
        ("helper::array_iterator_step_value", true),
        ("helper::array_iterator_close", false),
    ] {
        let indices = names
            .iter()
            .filter_map(|(&index, &actual)| (actual == name).then_some(index))
            .collect::<Vec<_>>();
        assert_eq!(indices.len(), 1, "one physical {name} kernel in R");
        let index = indices[0];
        let exported = exports
            .iter()
            .filter(|row| row.kind == ExternalKind::Func && row.index == index)
            .collect::<Vec<_>>();
        assert_eq!(exported.len(), 1, "one export of the actual {name} body");
        assert_eq!(exported[0].name, format!("f{index}"));
        let imported = imports
            .iter()
            .enumerate()
            .filter_map(|(ordinal, &(module, field, _))| {
                (module == lila_aot_wasm::RUNTIME_IMPORT_NAMESPACE && field == exported[0].name)
                    .then_some(ordinal as u32)
            })
            .collect::<Vec<_>>();
        assert_eq!(imported.len(), 1, "P imports the actual {name} export once");
        assert!(
            resumed_calls.contains(&imported[0]),
            "a P body that reloads the retained iterator calls its imported {name} kernel"
        );
        let program_signature = &program_signatures[&imports[imported[0] as usize].2];
        assert_eq!(
            program_signature.results().len(),
            5,
            "whole Completion result"
        );
        let parameter_count = if reads_next {
            4
        } else {
            4 + program_signature.results().len()
        };
        assert_eq!(program_signature.params().len(), parameter_count);
        if !reads_next {
            assert_eq!(
                &program_signature.params()[1..6],
                program_signature.results(),
                "P passes the whole pending Completion"
            );
        }
        let Some(ValType::Ref(reference)) = program_signature.params().first() else {
            panic!("P must pass its actual retained iterator record to {name}");
        };
        assert!(!reference.is_nullable());
        let HeapType::Concrete(record) = reference.heap_type() else {
            panic!("P must pass a concrete retained iterator record to {name}");
        };
        assert_eq!(record.as_module_index(), Some(program_record));
        let ordinal = index
            .checked_sub(imported_functions)
            .expect("kernel is defined in R") as usize;
        let signature = &signatures[&definitions[ordinal]];
        // Both modules declare the same core GC prefix. Check every ABI operand,
        // including caller Realm, Environment and Program, beyond the arity.
        assert_eq!(
            program_signature, signature,
            "P/R share the complete typed {name} ABI"
        );
        assert_eq!(
            signature.results().len(),
            5,
            "R returns the whole Completion"
        );
        assert_eq!(
            signature.params().len(),
            parameter_count,
            "typed {name} caller operands"
        );
        if !reads_next {
            assert_eq!(
                &signature.params()[1..6],
                signature.results(),
                "R receives the whole pending Completion"
            );
        }
        let Some(ValType::Ref(reference)) = signature.params().first() else {
            panic!("{name} must receive the actual GC iterator record");
        };
        assert!(!reference.is_nullable());
        let HeapType::Concrete(record) = reference.heap_type() else {
            panic!("{name} must receive a concrete iterator record");
        };
        let record = record.as_module_index().expect("module record index");
        assert_eq!(
            *shared_record.get_or_insert(record),
            record,
            "step and close share one R record type"
        );
        // Resolve R's record independently; P's numeric type indices are not authority.
        let fields = &structures[&record].fields;
        assert_eq!(fields.len(), 3);
        assert!(!fields[0].mutable && !fields[1].mutable && fields[2].mutable);
        assert_eq!(fields[0].element_type, fields[1].element_type);
        assert!(matches!(
            fields[0].element_type,
            StorageType::Val(ValType::Ref(_))
        ));
        assert_eq!(fields[2].element_type.unpack(), ValType::I32);
        let mut reads = [0; 3];
        let mut writes = 0;
        for operator in bodies[ordinal]
            .get_operators_reader()
            .expect("called runtime body opens")
        {
            match operator.expect("called runtime operator decodes") {
                Operator::StructGet {
                    struct_type_index,
                    field_index,
                } if struct_type_index == record => reads[field_index as usize] += 1,
                Operator::StructSet {
                    struct_type_index,
                    field_index: 2,
                } if struct_type_index == record => writes += 1,
                _ => {}
            }
        }
        assert!(
            reads[0] > 0 && reads[2] > 0 && writes > 0,
            "{name} consumes its iterator and owns native DONE"
        );
        assert_eq!(
            reads[1] > 0,
            reads_next,
            "only step consumes the cached next method"
        );
        for i in 0..2 {
            cached_fields[i] += reads[i];
        }
        done_writes += writes;
    }
    (cached_fields, done_writes)
}
