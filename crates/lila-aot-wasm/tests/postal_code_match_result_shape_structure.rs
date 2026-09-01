use std::fs;
use std::path::Path;

const STRING: &str = include_str!("../src/builtins/string.rs");
const POSTAL_CODE_MATCH_RESULT_SHAPE: &str =
    include_str!("../src/builtins/string/postal_code_match_result_shape.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn postal_code_match_result_shape_is_a_private_non_copyable_two_variant_domain() {
    let domain = bounded(
        POSTAL_CODE_MATCH_RESULT_SHAPE,
        "enum PostalCodeMatchResultShape {",
        "\n\nimpl<'a> FunctionBuilder<'a> {",
    );
    let variants = domain
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && *line != "}")
        .collect::<Vec<_>>();

    assert_eq!(variants, ["GlobalMatchArray,", "ExecMatchArray,"]);
    let declaration_start = POSTAL_CODE_MATCH_RESULT_SHAPE
        .find("enum PostalCodeMatchResultShape {")
        .expect("missing postal-code result domain");
    let preceding_declaration = POSTAL_CODE_MATCH_RESULT_SHAPE[..declaration_start].trim();
    assert_eq!(preceding_declaration, "use super::*;");
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!domain.contains(capability));
        assert!(!POSTAL_CODE_MATCH_RESULT_SHAPE
            .contains(&format!("impl {capability} for PostalCodeMatchResultShape")));
    }
    assert!(!POSTAL_CODE_MATCH_RESULT_SHAPE.contains("pub enum PostalCodeMatchResultShape"));
    assert!(!POSTAL_CODE_MATCH_RESULT_SHAPE.contains("pub(crate) enum PostalCodeMatchResultShape"));
    assert!(!POSTAL_CODE_MATCH_RESULT_SHAPE.contains("pub(super) enum PostalCodeMatchResultShape"));
    assert_eq!(
        STRING
            .matches("mod postal_code_match_result_shape;")
            .count(),
        1
    );
    assert!(!STRING.contains("mod postal_code_match_result_shape {"));
    assert!(!STRING.contains("postal_code_match_result_shape::"));
    assert!(!STRING.contains("PostalCodeMatchResultShape"));
    assert!(!STRING.contains("emit_string_match_postal_code_from_string_locals"));

    assert_eq!(
        POSTAL_CODE_MATCH_RESULT_SHAPE
            .matches("PostalCodeMatchResultShape")
            .count(),
        8
    );
    assert_eq!(
        POSTAL_CODE_MATCH_RESULT_SHAPE
            .matches("GlobalMatchArray")
            .count(),
        4
    );
    assert_eq!(
        POSTAL_CODE_MATCH_RESULT_SHAPE
            .matches("ExecMatchArray")
            .count(),
        4
    );
}

#[test]
fn postal_code_emitter_projects_array_length_and_exec_publication_exhaustively() {
    let emitter = bounded(
        POSTAL_CODE_MATCH_RESULT_SHAPE,
        "    fn emit_string_match_postal_code_from_string_locals(",
        "\n}\n",
    );
    let normalized = without_whitespace(emitter);

    assert!(emitter.contains("result_shape: PostalCodeMatchResultShape,"));
    assert_eq!(emitter.matches("match &result_shape {").count(), 2);
    assert!(normalized.contains(
        "Instruction::I64Const(match&result_shape{PostalCodeMatchResultShape::GlobalMatchArray=>1,PostalCodeMatchResultShape::ExecMatchArray=>3,})"
    ));
    assert!(normalized.contains(
        "match&result_shape{PostalCodeMatchResultShape::GlobalMatchArray=>{}PostalCodeMatchResultShape::ExecMatchArray=>{"
    ));
    for forbidden in [
        "global: bool",
        "if global",
        "if !global",
        "matches!(result_shape",
        "_ =>",
        "unreachable!",
        "Default::default",
    ] {
        assert!(!emitter.contains(forbidden));
    }

    let discovery_end = emitter
        .find("match &result_shape {")
        .expect("missing result-shape projection");
    let discovery = &emitter[..discovery_end];
    assert_eq!(
        discovery
            .matches("self.emit_ascii_digit_run_match_to_local(")
            .count(),
        4
    );
    assert!(!discovery.contains("PostalCodeMatchResultShape::"));

    let publication = bounded(
        emitter,
        "        match &result_shape {\n            PostalCodeMatchResultShape::GlobalMatchArray => {}",
        "        function.instruction(&Instruction::LocalGet(array_local));",
    );
    let publication_normalized = without_whitespace(publication);
    assert_eq!(
        publication
            .matches("PostalCodeMatchResultShape::ExecMatchArray => {")
            .count(),
        1
    );
    assert!(publication.contains("capture1_payload_local"));
    assert!(publication.contains("capture2_payload_local"));
    assert!(publication.contains("ValueKind::Undefined.tag()"));
    assert!(publication.contains("self.emit_utf16_code_unit_len_from_utf8_locals("));
    assert!(publication.contains("HEAP_ARRAY_INDEX_PROP_DESCRIPTOR_KIND_OFFSET"));
    assert!(publication.contains("self.strings.payload(\"index\")"));
    assert!(publication.contains("HEAP_ARRAY_INPUT_PROP_DESCRIPTOR_KIND_OFFSET"));
    assert!(publication.contains("self.strings.payload(\"input\")"));
    assert!(publication_normalized.contains(
        "Instruction::I64Const(1));function.instruction(&Instruction::LocalSet(array_index_local));function.instruction(&Instruction::I64Const(ValueKind::String.tag()asi64));function.instruction(&Instruction::LocalSet(value_tag_local));self.emit_array_write(array_local,array_index_local,capture1_payload_local,value_tag_local,function,)?;"
    ));
    assert!(publication_normalized.contains(
        "Instruction::I64Const(2));function.instruction(&Instruction::LocalSet(array_index_local));function.instruction(&Instruction::LocalGet(has_capture2_local))"
    ));
    assert!(publication_normalized.contains(
        "function.instruction(&Instruction::LocalSet(capture2_payload_local));function.instruction(&Instruction::I64Const(ValueKind::String.tag()asi64));function.instruction(&Instruction::LocalSet(value_tag_local));self.emit_array_write(array_local,array_index_local,capture2_payload_local,value_tag_local,function,)?;"
    ));
    assert!(publication_normalized.contains(
        "Instruction::Else);function.instruction(&Instruction::I64Const(0));function.instruction(&Instruction::LocalSet(capture2_payload_local));function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag()asi64));function.instruction(&Instruction::LocalSet(value_tag_local));self.emit_array_write(array_local,array_index_local,capture2_payload_local,value_tag_local,function,)?;"
    ));
    assert!(publication_normalized.contains(
        "self.emit_utf16_code_unit_len_from_utf8_locals(src_offset_local,match_start_local,array_len_local,function,)"
    ));
    assert!(publication_normalized.contains(
        "self.emit_array_define_builtin_named_data_property(array_local,HEAP_ARRAY_INDEX_PROP_DESCRIPTOR_KIND_OFFSET,HEAP_ARRAY_INDEX_PROP_DATA_TAG_OFFSET,HEAP_ARRAY_INDEX_PROP_DATA_PAYLOAD_OFFSET,index_payload_local,value_tag_local,function,)"
    ));
    assert!(publication_normalized.contains(
        "self.emit_array_define_named_data_property(array_local,key_local,index_payload_local,value_tag_local,function,)?;"
    ));
    assert!(publication_normalized.contains(
        "self.emit_array_define_builtin_named_data_property(array_local,HEAP_ARRAY_INPUT_PROP_DESCRIPTOR_KIND_OFFSET,HEAP_ARRAY_INPUT_PROP_DATA_TAG_OFFSET,HEAP_ARRAY_INPUT_PROP_DATA_PAYLOAD_OFFSET,string_local,value_tag_local,function,)"
    ));
    assert!(publication_normalized.contains(
        "self.emit_array_define_named_data_property(array_local,key_local,string_local,value_tag_local,function,)?;"
    ));

    assert!(normalized.contains("ValueKind::Array.tag()asi64"));
    assert!(normalized.contains("ValueKind::Null.tag()asi64"));
}

#[test]
fn exactly_two_postal_code_producers_choose_their_result_shapes() {
    let normalized = without_whitespace(STRING);
    assert_eq!(
        normalized
            .matches("self.emit_string_match_postal_code_global_from_string_locals(input_string_local,self.result_local,self.result_tag_local,function,)?;")
            .count(),
        1
    );
    assert_eq!(
        normalized
            .matches("self.emit_string_match_postal_code_exec_from_string_locals(input_string_local,self.result_local,self.result_tag_local,function,)?;")
            .count(),
        1
    );

    let owner = without_whitespace(POSTAL_CODE_MATCH_RESULT_SHAPE);
    assert_eq!(
        owner.matches("self.emit_string_match_postal_code_from_string_locals(string_local,PostalCodeMatchResultShape::GlobalMatchArray,payload_local,tag_local,function,)").count(),
        1
    );
    assert_eq!(
        owner.matches("self.emit_string_match_postal_code_from_string_locals(string_local,PostalCodeMatchResultShape::ExecMatchArray,payload_local,tag_local,function,)").count(),
        1
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_string_match_postal_code_from_string_locals(",
        ),
        3,
        "the private emitter definition and exactly two semantic-wrapper calls must stay inventoried"
    );
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_string_match_postal_code_global_from_string_locals(",
        ),
        2,
        "the global semantic wrapper and its sole parent call must stay inventoried"
    );
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_string_match_postal_code_exec_from_string_locals(",
        ),
        2,
        "the exec semantic wrapper and its sole parent call must stay inventoried"
    );
}
