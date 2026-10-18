#![cfg(test)]
use super::*;
use std::borrow::Cow;
use wasm_encoder::{
    CodeSection, ElementSection, Elements, EntityType, ExportKind, ExportSection, Function,
    FunctionSection, ImportSection, Instruction, Module, NameMap, NameSection, TypeSection,
};

fn fixture(second_type: u32, second_value: i32) -> Vec<u8> {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([], [wasm_encoder::ValType::I32]);
    types.ty().function([], [wasm_encoder::ValType::I32]);
    module.section(&types);
    let mut imports = ImportSection::new();
    imports.import("R", "host", EntityType::Function(0));
    module.section(&imports);
    let mut functions = FunctionSection::new();
    for ty in [0, second_type, 0] {
        functions.function(ty);
    }
    module.section(&functions);
    let mut exports = ExportSection::new();
    exports.export("second", ExportKind::Func, 2);
    exports.export("wrapper", ExportKind::Func, 3);
    module.section(&exports);
    let mut elements = ElementSection::new();
    elements.declared(Elements::Functions(Cow::Borrowed(&[1, 2])));
    module.section(&elements);
    let mut code = CodeSection::new();
    for value in [17, second_value] {
        let mut body = Function::new([]);
        body.instruction(&Instruction::I32Const(value));
        body.instruction(&Instruction::End);
        code.function(&body);
    }
    let mut wrapper = Function::new([]);
    wrapper.instruction(&Instruction::I64Const(2)); // Original SOURCE_ID stays 2.
    wrapper.instruction(&Instruction::Drop);
    wrapper.instruction(&Instruction::RefFunc(2));
    wrapper.instruction(&Instruction::Drop);
    wrapper.instruction(&Instruction::Call(2));
    wrapper.instruction(&Instruction::End);
    code.function(&wrapper);
    module.section(&code);
    let mut functions = NameMap::new();
    functions.append(0, "host::retained");
    for (index, name) in [(1, "js::alpha"), (2, "js::beta"), (3, "js::wrapper")] {
        functions.append(index, name);
    }
    let mut names = NameSection::new();
    names.functions(&functions);
    module.section(&names);
    module.finish()
}

fn owners(second_local: u32) -> BTreeMap<u32, Owner> {
    [
        (1, Owner { local: 1, ready: 2 }),
        (
            2,
            Owner {
                local: second_local,
                ready: 2,
            },
        ),
    ]
    .into()
}

#[test]
fn equal_typed_owned_bodies_remap_references_but_preserve_numeric_source_ids() {
    let input = fixture(0, 17);
    let output = transform(input.clone(), &owners(1)).unwrap();
    assert_ne!(output, input);
    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(&output)
        .unwrap();
    let mut count = 0;
    let mut refs = Vec::new();
    let mut names = Vec::new();
    let mut source_ids = Vec::new();
    for payload in wasmparser::Parser::new(0).parse_all(&output) {
        match payload.unwrap() {
            wasmparser::Payload::FunctionSection(reader) => count = reader.count(),
            wasmparser::Payload::ExportSection(reader) => {
                assert_eq!(
                    reader
                        .into_iter()
                        .map(|x| x.unwrap().index)
                        .collect::<Vec<_>>(),
                    [1, 2]
                );
            }
            wasmparser::Payload::CodeSectionEntry(body) => {
                let mut reader = body.get_operators_reader().unwrap();
                while !reader.eof() {
                    match reader.read().unwrap() {
                        wasmparser::Operator::RefFunc { function_index }
                        | wasmparser::Operator::Call { function_index } => {
                            refs.push(function_index)
                        }
                        wasmparser::Operator::I64Const { value } => source_ids.push(value),
                        _ => {}
                    }
                }
            }
            wasmparser::Payload::CustomSection(section) => {
                if let wasmparser::KnownCustom::Name(reader) = section.as_known() {
                    for name in reader {
                        if let wasmparser::Name::Function(reader) = name.unwrap() {
                            names.extend(reader.into_iter().map(|row| {
                                let row = row.unwrap();
                                (row.index, row.name.to_owned())
                            }));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    assert_eq!(count, 2);
    assert_eq!(refs, [1, 1]);
    assert_eq!(source_ids, [2]);
    assert_eq!(
        names,
        [
            (0, "host::retained".into()),
            (1, "js::native_function_1".into()),
            (2, "js::native_function_2".into())
        ]
    );
}

#[test]
fn different_types_bodies_owners_or_unknown_relocations_do_not_alias() {
    for (ty, value, local) in [(1, 17, 1), (0, 29, 1), (0, 17, 2)] {
        let input = fixture(ty, value);
        assert_eq!(transform(input.clone(), &owners(local)).unwrap(), input);
    }
    let mut input = fixture(0, 17);
    let section = wasm_encoder::CustomSection {
        name: Cow::Borrowed("unknown.relocations"),
        data: Cow::Borrowed(&[2]),
    };
    input.push(0);
    wasm_encoder::Encode::encode(&section, &mut input);
    assert_eq!(transform(input.clone(), &owners(1)).unwrap(), input);
}
