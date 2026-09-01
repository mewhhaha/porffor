use std::fs;
use std::path::Path;

const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const BOOTSTRAP_SOURCE: &str = include_str!("../src/builtins/bootstrap.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/function-prototype-materialization.md");
const TASK: &str = include_str!("../../../tasks/09-functions-classes-private-elements.md");

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
fn materialization_policy_is_the_exact_non_capability_domain() {
    let declaration_marker = "pub(crate) enum FunctionPrototypeMaterialization {";
    let declaration_offset = FUNCTIONS_SOURCE
        .find(declaration_marker)
        .expect("materialization-policy declaration");
    let preceding_item_end = FUNCTIONS_SOURCE[..declaration_offset]
        .rfind('}')
        .expect("item before materialization-policy declaration");
    let declaration_end = FUNCTIONS_SOURCE[declaration_offset..]
        .find("\n}")
        .map(|offset| declaration_offset + offset + 2)
        .expect("materialization-policy declaration end");
    let declaration_region = &FUNCTIONS_SOURCE[preceding_item_end + 1..declaration_end];
    let expected_declaration = r#"

/// Whether function allocation also creates the default own `prototype`
/// property. This policy is deliberately separate from semantic
/// constructability; realm bootstrap supplies a few intrinsic prototypes.
pub(crate) enum FunctionPrototypeMaterialization {
    Automatic,
    BootstrapSupplied,
}
"#;
    assert_eq!(
        normalized(declaration_region),
        normalized(expected_declaration),
        "the exact adjacent declaration must remain attribute-free"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "FunctionPrototypeMaterialization"),
        12,
        "one declaration, one import, two parameters, six producers and two exhaustive arms own every mention"
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "FunctionPrototypeMaterialization::Automatic"),
        3
    );
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "FunctionPrototypeMaterialization::BootstrapSupplied"
        ),
        5
    );
    for forbidden in [
        "impl FunctionPrototypeMaterialization",
        "for FunctionPrototypeMaterialization",
    ] {
        assert!(
            !FUNCTIONS_SOURCE.contains(forbidden) && !BOOTSTRAP_SOURCE.contains(forbidden),
            "found `{forbidden}`"
        );
    }
}

#[test]
fn six_producers_select_the_exact_materialization_policy() {
    let ordinary_wrapper = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn emit_function_value_payload(\n",
        "    pub(crate) fn emit_function_value_payload_with_prototype_materialization(\n",
    );
    assert_eq!(
        ordinary_wrapper
            .matches("FunctionPrototypeMaterialization::Automatic")
            .count(),
        1
    );
    assert!(!ordinary_wrapper.contains("FunctionPrototypeMaterialization::BootstrapSupplied"));

    let realm_wrapper = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn emit_function_value_payload_in_realm(\n",
        "    /// Materialize the created realm's `%Array%` constructor",
    );
    assert_eq!(
        realm_wrapper
            .matches("FunctionPrototypeMaterialization::Automatic")
            .count(),
        1
    );
    assert!(!realm_wrapper.contains("FunctionPrototypeMaterialization::BootstrapSupplied"));

    let realm_array_wrapper = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn emit_realm_array_constructor_value_payload(\n",
        "    fn emit_function_value_payload_in_realm_with_prototype_materialization(\n",
    );
    assert_eq!(
        realm_array_wrapper
            .matches("FunctionPrototypeMaterialization::BootstrapSupplied")
            .count(),
        1
    );
    assert!(!realm_array_wrapper.contains("FunctionPrototypeMaterialization::Automatic"));

    for (start, end) in [
        (
            "    pub(crate) fn init_typed_array_intrinsic(\n",
            "    pub(crate) fn repair_typed_array_constructor_graph(\n",
        ),
        (
            "    pub(crate) fn init_generator_function_intrinsics(\n",
            "    pub(crate) fn init_async_function_intrinsics(\n",
        ),
        (
            "    pub(crate) fn init_async_function_intrinsics(\n",
            "    pub(crate) fn init_runtime_roots(",
        ),
    ] {
        let bootstrap_producer = bounded(BOOTSTRAP_SOURCE, start, end);
        assert_eq!(
            bootstrap_producer
                .matches("FunctionPrototypeMaterialization::BootstrapSupplied")
                .count(),
            1
        );
        assert!(!bootstrap_producer.contains("FunctionPrototypeMaterialization::Automatic"));
    }
}

#[test]
fn exhaustive_policy_projection_preserves_the_automatic_allocation_gate() {
    let materializer = bounded(
        FUNCTIONS_SOURCE,
        "    pub(crate) fn emit_function_value_payload_with_prototype_materialization(\n",
        "    pub(crate) fn emit_function_value_payload_in_realm(\n",
    );
    let expected_gate = "if(matchprototype_materialization{FunctionPrototypeMaterialization::Automatic=>true,FunctionPrototypeMaterialization::BootstrapSupplied=>false,})&&!is_html_dda&&(meta.protocol.is_constructable()||instance_prototype_global_index.is_some()){";
    assert_eq!(normalized(materializer).matches(expected_gate).count(), 1);
    for forbidden in [
        "prototype_materialization ==",
        "prototype_materialization !=",
        "matches!(prototype_materialization",
        "_ =>",
    ] {
        assert!(!materializer.contains(forbidden), "found `{forbidden}`");
    }

    let materialization_start = materializer
        .find("        let instance_prototype_global_index =")
        .expect("instance-prototype projection start");
    let publication_end_marker = "        Ok(())";
    let publication_end = materializer[materialization_start..]
        .find(publication_end_marker)
        .map(|offset| materialization_start + offset + publication_end_marker.len())
        .expect("function-object publication end");
    let materialization_and_publication = &materializer[materialization_start..publication_end];
    let expected_materialization_and_publication = r#"        let instance_prototype_global_index =
            syntax_function_instance_prototype_global_index(meta.protocol.execution_kind());
        if (match prototype_materialization {
            FunctionPrototypeMaterialization::Automatic => true,
            FunctionPrototypeMaterialization::BootstrapSupplied => false,
        }) && !is_html_dda
            && (meta.protocol.is_constructable() || instance_prototype_global_index.is_some())
        {
            self.emit_alloc_plain_object_with_prototype(
                None,
                Some(instance_prototype_global_index.unwrap_or(OBJECT_PROTOTYPE_GLOBAL_INDEX)),
                function,
            )?;
            function.instruction(&Instruction::LocalSet(prototype_local));
            function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
            function.instruction(&Instruction::LocalSet(proto_tag_local));
            self.store_i64_local_at_offset(
                object_local,
                HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,
                proto_tag_local,
                function,
            );
            self.store_i64_local_at_offset(
                object_local,
                HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,
                prototype_local,
                function,
            );
            function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::LocalGet(prototype_local));
            function.instruction(&Instruction::LocalSet(proto_value_local));
            self.emit_object_append_data_property_with_flags(
                object_local,
                key_local,
                proto_value_local,
                proto_tag_local,
                true,
                false,
                false,
                function,
            )?;

            if !matches!(
                meta.protocol.execution_kind(),
                FunctionExecutionKind::Generator | FunctionExecutionKind::AsyncGenerator
            ) {
                function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
                function.instruction(&Instruction::LocalSet(key_local));
                function.instruction(&Instruction::LocalGet(object_local));
                function.instruction(&Instruction::LocalSet(proto_value_local));
                function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
                function.instruction(&Instruction::LocalSet(proto_tag_local));
                self.emit_object_append_data_property_with_flags(
                    prototype_local,
                    key_local,
                    proto_value_local,
                    proto_tag_local,
                    true,
                    false,
                    true,
                    function,
                )?;
            }
        }

        function.instruction(&Instruction::LocalGet(object_local));
        self.release_temp_local(proto_tag_local);
        self.release_temp_local(proto_value_local);
        self.release_temp_local(key_local);
        self.release_temp_local(prototype_local);
        if let Some(function_context_local) = function_context_local {
            self.release_temp_local(function_context_local);
        }
        if let Some(named_context_local) = named_context_local {
            self.release_temp_local(named_context_local);
        }
        self.release_temp_local(object_local);
        Ok(())"#;
    assert_eq!(
        normalized(materialization_and_publication),
        normalized(expected_materialization_and_publication),
        "automatic prototype materialization and function-object publication must keep their exact calls, arguments, flags and order"
    );
}

#[test]
fn contract_and_t09_record_the_exhaustive_source_equivalence() {
    for marker in [
        "six producer sites",
        "exhaustive two-arm projection",
        "changes no emitted instruction",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker `{marker}`"
        );
    }
    assert!(TASK.contains("FunctionPrototypeMaterialization::{Automatic, BootstrapSupplied}"));
    assert!(TASK.contains("function-prototype-materialization.md"));
}
