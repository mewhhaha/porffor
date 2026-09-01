use std::fs;
use std::path::Path;

const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/pending-to-primitive-operation-identity.md");
const TASK: &str = include_str!("../../../tasks/04-spec-operations-and-completion-abi.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn recursive_rust_source_count(root: &Path, needle: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return recursive_rust_source_count(&path, needle);
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
fn named_operation_boundaries_replace_the_ignored_generic_marker() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        recursive_rust_source_count(&source_root, "MayThrowOperation"),
        0,
        "an ignored operation marker must not masquerade as a type proof",
    );

    let get_v = normalized(bounded(
        OPERATIONS_SOURCE,
        "    fn compile_property_get_v_to_locals(",
        "    fn emit_builtin_arg_to_number_payload(",
    ));
    assert!(get_v.contains("self.compile_spec_operation_to_locals(SpecOperationIr::GetV,"));
    assert!(get_v.contains(
        "self.emit_propagate_throw_from_locals_if_needed(payload_local,tag_local,function)"
    ));

    let to_number = normalized(bounded(
        OPERATIONS_SOURCE,
        "    fn emit_builtin_arg_to_number_payload(",
        "    pub(crate) fn emit_construct(",
    ));
    assert!(to_number.contains("self.emit_return_current_completion_if_throw(function)"));
}

#[test]
fn pending_completion_represents_only_to_primitive() {
    let declaration_prefix = bounded(
        OPERATIONS_SOURCE,
        "/// A tagged `ToPrimitive` result whose possible throw still needs an owner.",
        "struct PendingToPrimitiveCompletion {",
    );
    assert!(!declaration_prefix.contains("#[derive"));

    let fields = normalized(bounded(
        OPERATIONS_SOURCE,
        "struct PendingToPrimitiveCompletion {",
        "}\n\nimpl PendingToPrimitiveCompletion",
    ));
    assert_eq!(fields, "payload_local:u32,tag_local:u32,");
    assert!(!fields.contains("operation"));

    let implementation = normalized(bounded(
        OPERATIONS_SOURCE,
        "impl PendingToPrimitiveCompletion {",
        "fn validate_spec_operation_operands(",
    ));
    assert!(implementation
        .contains("fnnew(payload_local:u32,tag_local:u32)->Self{Self{payload_local,tag_local,}}"));
    assert!(!implementation.contains("MayThrowOperation"));
    assert_eq!(
        implementation
            .matches("letSelf{payload_local,tag_local,}=self;")
            .count(),
        6
    );
    assert!(!implementation.contains("debug_assert"));
    assert!(!implementation.contains("operation:"));
}

#[test]
fn every_raw_producer_constructs_the_fixed_identity_token() {
    let raw_emitters = normalized(bounded(
        OPERATIONS_SOURCE,
        "fn emit_tagged_to_primitive_locals_pending(",
        "fn emit_object_to_primitive_locals_inner(",
    ));
    assert_eq!(
        raw_emitters
            .matches("PendingToPrimitiveCompletion::new(payload_local,tag_local)")
            .count(),
        3,
    );
    assert!(!raw_emitters.contains("letoperation="));
    assert!(!raw_emitters.contains("MayThrowOperation"));

    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("PendingToPrimitiveCompletion"));
        assert!(evidence.contains("operation boundaries now own identity"));
        assert!(evidence.contains("ToPrimitive"));
        assert!(evidence.contains("unrepresentable"));
    }
}
