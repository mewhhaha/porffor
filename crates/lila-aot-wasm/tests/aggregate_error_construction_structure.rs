use std::fs;
use std::path::Path;

const ERRORS_SOURCE: &str = include_str!("../src/builtins/errors.rs");
const MESSAGE_CONSTRUCTOR_SOURCE: &str = include_str!("../src/builtins/errors/constructor.rs");
const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source.find(earlier).expect("earlier operation");
    let later_offset = source.find(later).expect("later operation");
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn count_in_rust_sources(root: &Path, fragment: &str) -> usize {
    let mut count = 0;
    for entry in fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
    {
        let path = entry.expect("failed to read Rust source entry").path();
        if path.is_dir() {
            count += count_in_rust_sources(&path, fragment);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            count += source.matches(fragment).count();
        }
    }
    count
}

#[test]
fn aggregate_error_construction_has_a_closed_cause_options_role() {
    let variants = ERRORS_SOURCE
        .split_once("enum ErrorCauseOptionsArgument {")
        .expect("cause options role")
        .1
        .split_once('}')
        .expect("cause options role end")
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(variants, ["MessageError,", "AggregateError,"]);

    let projection = ERRORS_SOURCE
        .split_once("impl ErrorCauseOptionsArgument {")
        .expect("cause options projection")
        .1
        .split_once("struct PreparedAggregateErrorLocal")
        .expect("cause options projection end")
        .0;
    assert!(projection.contains("Self::MessageError => 1"));
    assert!(projection.contains("Self::AggregateError => 2"));
    assert!(!projection.contains("_ =>"));

    let installer = ERRORS_SOURCE
        .split_once("fn emit_install_error_cause_from_arg(")
        .expect("cause installer")
        .1
        .split_once("fn emit_prepare_aggregate_error_instance(")
        .expect("cause installer end")
        .0;
    assert!(installer.contains("options_argument: ErrorCauseOptionsArgument"));
    assert!(installer.contains("options_argument.index()"));
    assert!(!installer.contains("options_arg_index"));

    let product_sources =
        format!("{ERRORS_SOURCE}\n{MESSAGE_CONSTRUCTOR_SOURCE}\n{PROMISE_SOURCE}");
    assert_eq!(
        product_sources
            .matches("ErrorCauseOptionsArgument::MessageError")
            .count(),
        2
    );
    assert_eq!(
        product_sources
            .matches("ErrorCauseOptionsArgument::AggregateError")
            .count(),
        1
    );
}

#[test]
fn aggregate_error_construction_requires_a_prepared_object_before_errors() {
    assert_eq!(
        ERRORS_SOURCE
            .matches("\nstruct PreparedAggregateErrorLocal {")
            .count(),
        1
    );
    assert!(!ERRORS_SOURCE.contains("pub struct PreparedAggregateErrorLocal"));
    assert!(!ERRORS_SOURCE.contains("pub(crate) struct PreparedAggregateErrorLocal"));
    assert_eq!(
        ERRORS_SOURCE.matches("PreparedAggregateErrorLocal").count(),
        7
    );
    assert!(!MESSAGE_CONSTRUCTOR_SOURCE.contains("PreparedAggregateErrorLocal"));
    assert!(!PROMISE_SOURCE.contains("PreparedAggregateErrorLocal"));

    let declaration = ERRORS_SOURCE
        .split_once("struct PreparedAggregateErrorLocal {")
        .expect("prepared AggregateError token")
        .0
        .rsplit_once("\n\n")
        .expect("prepared token attribute boundary")
        .1;
    assert!(declaration.contains("#[must_use"));
    assert!(!declaration.contains("derive"));
    let fields = ERRORS_SOURCE
        .split_once("struct PreparedAggregateErrorLocal {")
        .expect("prepared AggregateError token fields")
        .1
        .split_once('}')
        .expect("prepared AggregateError token fields end")
        .0
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(fields, ["object: u32,"]);
    assert!(!ERRORS_SOURCE.contains("impl Copy for PreparedAggregateErrorLocal"));
    assert!(!ERRORS_SOURCE.contains("impl Clone for PreparedAggregateErrorLocal"));
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_alloc_aggregate_error_instance_from_locals",
        ),
        0,
        "the old untyped combined allocator must have no definition or caller",
    );
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_promise_any_aggregate_error_from_locals(",
        ),
        3,
        "the Promise.any-only wrapper must have one definition and two callers",
    );
    assert_eq!(
        count_in_rust_sources(
            &source_root,
            "emit_prepare_promise_any_aggregate_error_instance(",
        ),
        2,
        "the private Promise.any producer must have one definition and only its wrapper caller",
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "emit_finish_aggregate_error_instance("),
        3,
        "the finalizer must have one definition, one constructor caller and one Promise.any caller",
    );
    assert_eq!(
        ERRORS_SOURCE
            .matches("\n    fn emit_prepare_promise_any_aggregate_error_instance(")
            .count(),
        1,
        "the Promise.any token producer must remain private",
    );
    assert!(!ERRORS_SOURCE
        .contains("pub(crate) fn emit_prepare_promise_any_aggregate_error_instance(",));
    assert_eq!(
        ERRORS_SOURCE
            .matches("\n    fn emit_finish_aggregate_error_instance(")
            .count(),
        1,
        "the shared token consumer must remain private",
    );

    let prepare = ERRORS_SOURCE
        .split_once("fn emit_prepare_aggregate_error_instance(")
        .expect("AggregateError preparation phase")
        .1
        .split_once("fn emit_prepare_promise_any_aggregate_error_instance(")
        .expect("AggregateError preparation phase end")
        .0;
    assert!(prepare.contains(") -> Result<PreparedAggregateErrorLocal, EmitError> {"));
    assert_eq!(
        prepare.matches("Ok(PreparedAggregateErrorLocal {").count(),
        1
    );

    let promise_prepare = ERRORS_SOURCE
        .split_once("fn emit_prepare_promise_any_aggregate_error_instance(")
        .expect("Promise.any AggregateError preparation phase")
        .1
        .split_once("fn emit_finish_aggregate_error_instance(")
        .expect("Promise.any AggregateError preparation phase end")
        .0;
    assert!(promise_prepare.contains(") -> Result<PreparedAggregateErrorLocal, EmitError> {"));
    assert_eq!(
        promise_prepare
            .matches("Ok(PreparedAggregateErrorLocal {")
            .count(),
        1
    );
    assert_eq!(
        promise_prepare
            .matches("emit_alloc_plain_object_with_prototype(")
            .count(),
        1
    );
    assert_eq!(
        promise_prepare
            .matches("OBJECT_INTERNAL_BRAND_ERROR")
            .count(),
        1
    );
    for forbidden in [
        "emit_value_to_string_payload(",
        "emit_install_error_cause_from_arg(",
        "emit_object_define_data(",
        "strings.payload(\"message\")",
        "strings.payload(\"errors\")",
    ] {
        assert!(
            !promise_prepare.contains(forbidden),
            "Promise.any preparation must not run constructor-only phase {forbidden}",
        );
    }
    assert_eq!(
        without_whitespace(promise_prepare),
        without_whitespace(
            r#"
            &mut self,
                prototype_payload_local: u32,
                function: &mut Function,
            ) -> Result<PreparedAggregateErrorLocal, EmitError> {
                let object_local = self.reserve_temp_local();
                self.emit_alloc_plain_object_with_prototype(
                    Some(prototype_payload_local),
                    None,
                    function
                )?;
                function.instruction(&Instruction::LocalSet(object_local));
                self.store_i64_const_at_offset(
                    object_local,
                    HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                    OBJECT_INTERNAL_BRAND_ERROR,
                    function,
                );
                Ok(PreparedAggregateErrorLocal {
                    object: object_local,
                })
            }
            "#,
        ),
        "Promise.any may prepare only a fresh branded AggregateError object",
    );

    let finish = ERRORS_SOURCE
        .split_once("fn emit_finish_aggregate_error_instance(")
        .expect("AggregateError finalization phase")
        .1
        .split_once("pub(crate) fn emit_promise_any_aggregate_error_from_locals(")
        .expect("AggregateError finalization phase end")
        .0;
    assert_eq!(
        finish
            .matches("prepared: PreparedAggregateErrorLocal")
            .count(),
        1
    );
    assert_eq!(
        finish.matches("let PreparedAggregateErrorLocal {").count(),
        1
    );

    let promise_wrapper = ERRORS_SOURCE
        .split_once("pub(crate) fn emit_promise_any_aggregate_error_from_locals(")
        .expect("Promise.any AggregateError wrapper")
        .1
        .split_once("pub(crate) fn emit_alloc_suppressed_error_instance_from_locals(")
        .expect("Promise.any AggregateError wrapper end")
        .0;
    assert_eq!(
        promise_wrapper
            .matches("emit_prepare_promise_any_aggregate_error_instance(")
            .count(),
        1
    );
    assert_eq!(
        promise_wrapper
            .matches("emit_finish_aggregate_error_instance(")
            .count(),
        1
    );
    assert_before(
        promise_wrapper,
        "emit_prepare_promise_any_aggregate_error_instance(",
        "emit_finish_aggregate_error_instance(",
    );
    assert_eq!(
        without_whitespace(promise_wrapper),
        without_whitespace(
            r#"
            &mut self,
                errors_payload_local: u32,
                prototype_payload_local: u32,
                payload_local: u32,
                tag_local: u32,
                function: &mut Function,
            ) -> Result<(), EmitError> {
                let prepared = self.emit_prepare_promise_any_aggregate_error_instance(
                    prototype_payload_local,
                    function
                )?;
                self.emit_finish_aggregate_error_instance(
                    prepared,
                    errors_payload_local,
                    payload_local,
                    tag_local,
                    function,
                )
            }
            "#,
        ),
        "the crate-visible Promise.any boundary may only prepare and consume the private token",
    );

    let reject_element = PROMISE_SOURCE
        .split_once("pub(crate) fn emit_promise_any_reject_element(")
        .expect("Promise.any reject-element body")
        .1
        .split_once("pub(crate) fn emit_promise_race(")
        .expect("Promise.any reject-element body end")
        .0;
    assert_eq!(
        reject_element
            .matches("emit_promise_any_aggregate_error_from_locals(")
            .count(),
        1
    );
    assert!(
        without_whitespace(reject_element).contains(&without_whitespace(
            r#"
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_PAYLOAD_OFFSET,
            reject_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_RESOLVE_TAG_OFFSET,
            reject_tag_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(aggregate_prototype_local));
        self.emit_promise_any_aggregate_error_from_locals(
            errors_payload_local,
            aggregate_prototype_local,
            aggregate_payload_local,
            aggregate_tag_local,
            function,
        )?;
        "#,
        ))
    );

    let combinator = PROMISE_SOURCE
        .split_once("fn emit_promise_combinator(")
        .expect("Promise combinator body")
        .1
        .split_once("pub(crate) fn emit_promise_resolving_function(")
        .expect("Promise combinator body end")
        .0;
    assert_eq!(
        combinator
            .matches("emit_promise_any_aggregate_error_from_locals(")
            .count(),
        1
    );
    assert!(without_whitespace(combinator).contains(&without_whitespace(
        r#"
        self.load_i64_to_local_from_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(remaining_local));
        self.store_i64_local_at_offset(
            shared_context_local,
            HEAP_PROMISE_ALL_SHARED_REMAINING_OFFSET,
            remaining_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(remaining_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        if mode == PromiseCombinatorMode::FirstFulfillment {
            function.instruction(&Instruction::GlobalGet(
                AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            ));
            function.instruction(&Instruction::LocalSet(element_context_local));
            self.emit_promise_any_aggregate_error_from_locals(
                values_payload_local,
                element_context_local,
                next_value_payload_local,
                next_value_tag_local,
                function,
            )?;
        "#,
    )));

    let arm = ERRORS_SOURCE
        .split_once("NativeErrorKind::AggregateError => {")
        .expect("AggregateError constructor arm")
        .1
        .split_once("NativeErrorKind::SuppressedError => {")
        .expect("AggregateError constructor arm end")
        .0;
    assert_eq!(
        arm.matches("emit_prepare_aggregate_error_instance(")
            .count(),
        1
    );
    assert_eq!(
        arm.matches("emit_aggregate_error_iterable_to_list_payload(")
            .count(),
        1
    );
    assert_eq!(
        arm.matches("emit_finish_aggregate_error_instance(").count(),
        1
    );
    assert_before(
        arm,
        "emit_prepare_aggregate_error_instance(",
        "emit_aggregate_error_iterable_to_list_payload(",
    );
    assert_before(
        arm,
        "emit_aggregate_error_iterable_to_list_payload(",
        "emit_finish_aggregate_error_instance(",
    );
}

#[test]
fn aggregate_error_construction_emits_message_cause_then_errors() {
    let prepare = ERRORS_SOURCE
        .split_once("fn emit_prepare_aggregate_error_instance(")
        .expect("AggregateError preparation phase")
        .1
        .split_once("fn emit_prepare_promise_any_aggregate_error_instance(")
        .expect("AggregateError preparation phase end")
        .0;
    assert_eq!(
        prepare
            .matches("emit_alloc_plain_object_with_prototype(")
            .count(),
        1
    );
    assert_eq!(prepare.matches("OBJECT_INTERNAL_BRAND_ERROR").count(), 1);
    assert_eq!(prepare.matches("emit_value_to_string_payload(").count(), 1);
    assert_eq!(prepare.matches("strings.payload(\"message\")").count(), 1);
    assert_eq!(prepare.matches("emit_object_define_data(").count(), 1);
    assert_eq!(
        prepare
            .matches("emit_install_error_cause_from_arg(")
            .count(),
        1
    );
    assert_before(
        prepare,
        "emit_alloc_plain_object_with_prototype(",
        "OBJECT_INTERNAL_BRAND_ERROR",
    );
    assert_before(
        prepare,
        "OBJECT_INTERNAL_BRAND_ERROR",
        "emit_value_to_string_payload(",
    );
    let compact_prepare = without_whitespace(prepare);
    let compact_message_then_cause = without_whitespace(
        r#"
        function.instruction(&Instruction::LocalGet(message_arg_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_value_to_string_payload(
            message_arg_payload_local,
            message_arg_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(message_payload_local));
        function.instruction(&Instruction::I64Const(self.strings.payload("message")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            message_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.emit_install_error_cause_from_arg(
            object_local,
            ErrorCauseOptionsArgument::AggregateError,
            function,
        )?;
        "#,
    );
    assert!(
        compact_prepare.contains(&compact_message_then_cause),
        "the optional message conversion and definition must complete before cause installation"
    );

    let finish = ERRORS_SOURCE
        .split_once("fn emit_finish_aggregate_error_instance(")
        .expect("AggregateError finalization phase")
        .1
        .split_once("pub(crate) fn emit_promise_any_aggregate_error_from_locals(")
        .expect("AggregateError finalization phase end")
        .0;
    assert!(finish.contains("prepared: PreparedAggregateErrorLocal"));
    assert!(finish.contains("let PreparedAggregateErrorLocal"));
    assert_eq!(finish.matches("strings.payload(\"errors\")").count(), 1);
    assert_eq!(finish.matches("emit_object_define_data(").count(), 1);
    let compact_finish = without_whitespace(finish);
    let compact_publication = without_whitespace(
        r#"
        self.emit_object_define_data(
            object_local,
            key_local,
            errors_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
        "#,
    );
    assert!(
        compact_finish.contains(&compact_publication),
        "errors definition must publish the object/tag pair before reverse local release"
    );
}
