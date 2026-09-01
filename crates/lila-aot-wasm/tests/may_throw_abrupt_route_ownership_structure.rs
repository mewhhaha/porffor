use std::fs;
use std::path::Path;

const OPERATIONS_SOURCE: &str = include_str!("../src/operations.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/may-throw-operation-abrupt-route-ownership.md"
);
const TASK: &str = include_str!("../../../tasks/04-spec-operations-and-completion-abi.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
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

fn exact_identifier_count(source: &str, identifier: &str) -> usize {
    source
        .match_indices(identifier)
        .filter(|(offset, _)| {
            let before = source[..*offset].chars().next_back();
            let after = source[*offset + identifier.len()..].chars().next();
            [before, after].into_iter().all(|edge| {
                edge.map(|character| !character.is_alphanumeric() && character != '_')
                    .unwrap_or(true)
            })
        })
        .count()
}

fn recursive_rust_identifier_count(root: &Path, identifier: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return recursive_rust_identifier_count(&path, identifier);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            exact_identifier_count(&source, identifier)
        })
        .sum()
}

#[test]
fn generic_abrupt_route_authority_is_absent() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        recursive_rust_identifier_count(&source_root, "AbruptRoute"),
        0
    );
    assert_eq!(
        recursive_rust_identifier_count(&source_root, "finish_may_throw_operation"),
        0
    );
}

#[test]
fn get_v_wrapper_owns_active_handler_propagation() {
    let get_v = normalized(bounded(
        OPERATIONS_SOURCE,
        "    fn compile_property_get_v_to_locals(",
        "    fn emit_builtin_arg_to_number_payload(",
    ));
    let compile = get_v
        .find("self.compile_spec_operation_to_locals(SpecOperationIr::GetV,")
        .expect("GetV wrapper must select its exact descriptor");
    let propagate = get_v
        .find("self.emit_propagate_throw_from_locals_if_needed(payload_local,tag_local,function)")
        .expect("GetV wrapper must route throws to its active handler");
    assert!(compile < propagate);
    assert!(!get_v.contains("emit_return_current_completion_if_throw"));
}

#[test]
fn builtin_to_number_wrapper_owns_current_function_return() {
    let to_number = normalized(bounded(
        OPERATIONS_SOURCE,
        "    fn emit_builtin_arg_to_number_payload(",
        "    pub(crate) fn emit_construct(",
    ));
    let convert = to_number
        .find("self.emit_value_to_number_payload(tag_local,payload_local,function)?")
        .expect("ToNumber wrapper must perform its exact conversion");
    let store = to_number
        .find("function.instruction(&Instruction::LocalSet(payload_local))")
        .expect("ToNumber wrapper must remove the conversion result from the stack");
    let finish = to_number
        .find("self.emit_return_current_completion_if_throw(function)")
        .expect("ToNumber wrapper must return its current function on throw");
    assert!(convert < store && store < finish);
    assert!(!to_number.contains("emit_propagate_throw_from_locals_if_needed"));
}

#[test]
fn contract_and_task_record_named_completion_ownership() {
    for evidence in [CONTRACT, TASK] {
        assert!(evidence.contains("generic `AbruptRoute` is gone"));
        assert!(evidence.contains("GetV"));
        assert!(evidence.contains("ToNumber"));
        assert!(evidence.contains("unrepresentable"));
    }
}
