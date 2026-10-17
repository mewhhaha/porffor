#![cfg(test)]
use super::*;
// The wire fixtures deliberately bypass the typed emitter to exercise malformed
// transport rejection as well as the checked relocation/reencoding boundary.
fn operand_section(records: &[(u32, u32, u8, i32)]) -> wasm_encoder::CustomSection<'static> {
    let mut data = 1_u32.to_le_bytes().to_vec();
    data.extend_from_slice(&(records.len() as u32).to_le_bytes());
    for (body, operator, kind, value) in records {
        data.extend_from_slice(&body.to_le_bytes());
        data.extend_from_slice(&operator.to_le_bytes());
        data.push(*kind);
        data.extend_from_slice(&value.to_le_bytes());
    }
    wasm_encoder::CustomSection {
        name: Cow::Borrowed(crate::program_literal_operands::SECTION_NAME),
        data: Cow::Owned(data),
    }
}
fn append_section(bytes: &mut Vec<u8>, section: &wasm_encoder::CustomSection<'_>) {
    bytes.push(0);
    wasm_encoder::Encode::encode(section, bytes);
}
fn scalar_image(count: i32, ordinal: i32, number: i32) -> Vec<u8> {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I32]);
    module.section(&types);
    let mut imports = ImportSection::new();
    imports.import("R", "unused", EntityType::Function(0));
    let global_type = GlobalType {
        val_type: ValType::I32,
        mutable: false,
        shared: false,
    };
    imports.import("R", "existing", EntityType::Global(global_type));
    module.section(&imports);
    let mut functions = FunctionSection::new();
    functions.function(0);
    module.section(&functions);
    let mut globals = GlobalSection::new();
    globals.global(global_type, &ConstExpr::i32_const(19));
    module.section(&globals);
    let mut exports = ExportSection::new();
    exports.export("result", ExportKind::Func, 1);
    exports.export("existing", ExportKind::Global, 1);
    module.section(&exports);
    let mut function = Function::new([]);
    for instruction in [
        Instruction::I32Const(count),
        Instruction::I32Const(ordinal),
        Instruction::I32Add,
        Instruction::I32Const(number),
        Instruction::I32Add,
        Instruction::GlobalGet(1),
        Instruction::I32Add,
        Instruction::End,
    ] {
        function.instruction(&instruction);
    }
    let mut code = CodeSection::new();
    code.function(&function);
    module.section(&code);
    let mut names = wasm_encoder::NameSection::new();
    let mut globals = wasm_encoder::NameMap::new();
    globals.append(1, "original");
    names.globals(&globals);
    module.section(&names);
    module.section(&wasm_encoder::CustomSection {
        name: Cow::Borrowed("unrelated"),
        data: Cow::Borrowed(&[31]),
    });
    module.finish()
}

#[test]
fn checked_literal_operands_share_code_but_ordinary_numbers_remain_code() {
    let mut first = scalar_image(7, 3, 1);
    let mut second = scalar_image(9, 5, 1);
    let mut number = scalar_image(9, 5, 2);
    append_section(&mut first, &operand_section(&[(0, 0, 0, 7), (0, 1, 1, 3)]));
    for bytes in [&mut second, &mut number] {
        append_section(bytes, &operand_section(&[(0, 0, 0, 9), (0, 1, 1, 5)]));
    }
    let a = plan::Plan::read(&first).unwrap().unwrap();
    let b = plan::Plan::read(&second).unwrap().unwrap();
    let c = plan::Plan::read(&number).unwrap().unwrap();
    let encoded = reencode::encode(&first, &a).unwrap();
    assert_eq!(encoded, reencode::encode(&second, &b).unwrap());
    assert_ne!(encoded, reencode::encode(&number, &c).unwrap());
    assert_eq!(
        a.literal_operands
            .iter()
            .map(|v| v.value())
            .collect::<Vec<_>>(),
        [7, 3]
    );
    assert_eq!(
        b.literal_operands
            .iter()
            .map(|v| v.value())
            .collect::<Vec<_>>(),
        [9, 5]
    );
    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(&encoded)
        .unwrap();
    let mut defined_global_export = None;
    let mut global_name_index = None;
    let mut unrelated = false;
    let mut relocated_gets = Vec::new();
    for payload in wasmparser::Parser::new(0).parse_all(&encoded) {
        match payload.unwrap() {
            wasmparser::Payload::ExportSection(exports) => {
                for export in exports {
                    let export = export.unwrap();
                    if export.name == "existing" {
                        defined_global_export = Some(export.index);
                    }
                }
            }
            wasmparser::Payload::CodeSectionEntry(body) => {
                for op in body.get_operators_reader().unwrap() {
                    if let wasmparser::Operator::GlobalGet { global_index } = op.unwrap() {
                        relocated_gets.push(global_index);
                    }
                }
            }
            wasmparser::Payload::CustomSection(section) => {
                assert_ne!(
                    section.name(),
                    crate::program_literal_operands::SECTION_NAME
                );
                if section.name() == "unrelated" {
                    assert_eq!(section.data(), [31]);
                    unrelated = true;
                }
                if let wasmparser::KnownCustom::Name(reader) = section.as_known() {
                    for name in reader {
                        if let wasmparser::Name::Global(names) = name.unwrap() {
                            for name in names {
                                global_name_index = Some(name.unwrap().index);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    assert_eq!(defined_global_export, Some(3));
    assert_eq!(global_name_index, Some(3));
    assert_eq!(relocated_gets, [1, 2, 3]);
    assert!(unrelated);
}

#[test]
fn malformed_literal_relocations_are_rejected_before_reencoding() {
    for records in [
        vec![(1, 0, 0, 7)],
        vec![(0, 99, 0, 7)],
        vec![(0, 2, 0, 7)],
        vec![(0, 0, 0, 8)],
        vec![(0, 0, 2, 7)],
        vec![(0, 0, 0, 7), (0, 0, 0, 7)],
        vec![(0, 1, 1, 3), (0, 0, 0, 7)],
    ] {
        let mut bytes = scalar_image(7, 3, 1);
        append_section(&mut bytes, &operand_section(&records));
        assert!(plan::Plan::read(&bytes).is_err(), "accepted {records:?}");
    }
    let valid = operand_section(&[(0, 0, 0, 7)]);
    let mut duplicate = scalar_image(7, 3, 1);
    append_section(&mut duplicate, &valid);
    append_section(&mut duplicate, &valid);
    assert!(plan::Plan::read(&duplicate).is_err());
    for payload in [
        vec![1, 0, 0],
        vec![2, 0, 0, 0, 0, 0, 0, 0],
        vec![1, 0, 0, 0, 1, 0, 0, 0],
    ] {
        let mut bytes = scalar_image(7, 3, 1);
        append_section(
            &mut bytes,
            &wasm_encoder::CustomSection {
                name: Cow::Borrowed(crate::program_literal_operands::SECTION_NAME),
                data: Cow::Owned(payload),
            },
        );
        assert!(plan::Plan::read(&bytes).is_err());
    }
}

#[test]
fn literal_relocations_cannot_overlap_packed_data_relocations() {
    let mut bytes = image(7, true);
    append_section(&mut bytes, &operand_section(&[(0, 1, 1, 7)]));
    assert!(plan::Plan::read(&bytes).is_err());
}
