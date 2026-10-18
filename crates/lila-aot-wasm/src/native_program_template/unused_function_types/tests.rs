use super::*;
use wasm_encoder::{
    CodeSection, CompositeInnerType as Inner, CompositeType, EntityType, ExportKind, ExportSection,
    FieldType, FuncType, Function, FunctionSection, HeapType, ImportSection, Instruction, RefType,
    StorageType, StructType, SubType, TypeSection, ValType,
};

fn reference(index: u32) -> ValType {
    ValType::Ref(RefType {
        nullable: true,
        heap_type: HeapType::Concrete(index),
    })
}

fn checked(module: Module) -> Vec<u8> {
    let bytes = module.finish();
    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(&bytes)
        .unwrap();
    bytes
}

fn declarations(bytes: &[u8]) -> Vec<wasmparser::SubType> {
    Parser::new(0)
        .parse_all(bytes)
        .filter_map(|payload| {
            if let Payload::TypeSection(reader) = payload.unwrap() {
                Some(
                    reader
                        .into_iter()
                        .flat_map(|group| group.unwrap().into_types())
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        })
        .flatten()
        .collect()
}

fn transform(bytes: Vec<u8>) -> Vec<u8> {
    let result = encode(bytes).unwrap();
    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(&result)
        .unwrap();
    assert_eq!(encode(result.clone()).unwrap(), result);
    result
}

#[test]
fn dead_slots_become_structs_without_changing_live_types_or_recursive_groups() {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I32], []); // Dead standalone slot 0.
    types.ty().function([], []); // Defined function uses slot 1.
    types.ty().rec([
        SubType {
            is_final: true,
            supertype_idx: None,
            composite_type: CompositeType {
                inner: Inner::Struct(StructType { fields: [].into() }),
                shared: false,
                descriptor: None,
                describes: None,
            },
        },
        SubType {
            is_final: true,
            supertype_idx: None,
            composite_type: CompositeType {
                inner: Inner::Func(FuncType::new([ValType::I64], [])),
                shared: false,
                descriptor: None,
                describes: None,
            },
        },
    ]);
    module.section(&types);
    let mut functions = FunctionSection::new();
    functions.function(1);
    module.section(&functions);
    let mut exports = ExportSection::new();
    exports.export("entry", ExportKind::Func, 0);
    module.section(&exports);
    let mut code = CodeSection::new();
    let mut body = Function::new([]);
    body.instruction(&Instruction::End);
    code.function(&body);
    module.section(&code);
    let bytes = checked(module);
    let result = transform(bytes.clone());
    let before = declarations(&bytes);
    let after = declarations(&result);
    assert!(matches!(
        after[0].composite_type.inner,
        wasmparser::CompositeInnerType::Struct(_)
    ));
    assert_eq!(before[1..], after[1..]);
    let groups = |bytes: &[u8]| {
        Parser::new(0)
            .parse_all(bytes)
            .filter_map(|payload| {
                if let Payload::TypeSection(reader) = payload.unwrap() {
                    let end = reader.range().end;
                    let groups = reader
                        .into_iter_with_offsets()
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap();
                    Some(
                        groups
                            .iter()
                            .enumerate()
                            .filter(|(_, (_, g))| g.is_explicit_rec_group())
                            .map(|(i, (start, _))| {
                                bytes[*start..groups.get(i + 1).map_or(end, |(s, _)| *s)].to_vec()
                            })
                            .collect::<Vec<_>>(),
                    )
                } else {
                    None
                }
            })
            .flatten()
            .collect::<Vec<_>>()
    };
    assert_eq!(groups(&bytes), groups(&result));
    let other_sections = |bytes: &[u8]| {
        Parser::new(0)
            .parse_all(bytes)
            .filter_map(|p| p.unwrap().as_section())
            .filter(|(id, _)| *id != 1)
            .map(|(id, range)| (id, bytes[range].to_vec()))
            .collect::<Vec<_>>()
    };
    assert_eq!(other_sections(&bytes), other_sections(&result));

    let mut unknown = Module::new();
    unknown.section(&RawSection { id: 99, data: &[] });
    assert!(encode(unknown.finish()).is_err());
}

#[test]
fn references_inside_type_definitions_and_subtypes_keep_their_targets() {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I32], []);
    types.ty().function([ValType::I64], []); // Only dead slot.
    types.ty().struct_([FieldType {
        element_type: StorageType::Val(reference(0)),
        mutable: false,
    }]);
    types.ty().subtype(&SubType {
        is_final: false,
        supertype_idx: None,
        composite_type: CompositeType {
            inner: Inner::Func(FuncType::new([], [])),
            shared: false,
            descriptor: None,
            describes: None,
        },
    });
    types.ty().subtype(&SubType {
        is_final: true,
        supertype_idx: Some(3),
        composite_type: CompositeType {
            inner: Inner::Func(FuncType::new([], [])),
            shared: false,
            descriptor: None,
            describes: None,
        },
    });
    module.section(&types);
    let bytes = checked(module);
    let before = declarations(&bytes);
    let after = declarations(&transform(bytes));
    assert!(matches!(
        after[1].composite_type.inner,
        wasmparser::CompositeInnerType::Struct(_)
    ));
    for index in [0, 2, 3, 4] {
        assert_eq!(before[index], after[index]);
    }
}

#[test]
fn imports_locals_blocks_and_instruction_types_are_all_live() {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([], []); // Function import.
    types.ty().function([ValType::I32], []); // Typed reference local.
    types.ty().function([], [ValType::I32]); // Block type immediate.
    types.ty().function([], [ValType::I64]); // ref.null heap type immediate.
    types.ty().function([ValType::F32], []); // Dead.
    types.ty().function([], []); // Defined function.
    module.section(&types);
    let mut imports = ImportSection::new();
    imports.import("R", "function", EntityType::Function(0));
    module.section(&imports);
    let mut functions = FunctionSection::new();
    functions.function(5);
    module.section(&functions);
    let mut body = Function::new([(1, reference(1))]);
    body.instruction(&Instruction::Block(wasm_encoder::BlockType::FunctionType(
        2,
    )));
    body.instruction(&Instruction::I32Const(0));
    body.instruction(&Instruction::End);
    body.instruction(&Instruction::Drop);
    body.instruction(&Instruction::RefNull(HeapType::Concrete(3)));
    body.instruction(&Instruction::Drop);
    body.instruction(&Instruction::End);
    let mut code = CodeSection::new();
    code.function(&body);
    module.section(&code);
    let bytes = checked(module);
    let before = declarations(&bytes);
    let after = declarations(&transform(bytes));
    assert!(matches!(
        after[4].composite_type.inner,
        wasmparser::CompositeInnerType::Struct(_)
    ));
    for index in [0, 1, 2, 3, 5] {
        assert_eq!(before[index], after[index]);
    }
}
