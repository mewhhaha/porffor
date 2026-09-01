use std::fs;
use std::path::Path;

const IR_SOURCE: &str = include_str!("../../lila-ir/src/ir.rs");
const ANALYSIS_SOURCE: &str = include_str!("../../lila-ir/src/analysis.rs");
const LOWERING_SOURCE: &str = include_str!("../../lila-ir/src/lowering/for_of.rs");
const OBLIGATIONS_SOURCE: &str = include_str!("../../lila-ir/src/iterator_obligations.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const ASYNC_FUNCTION_FOR_OF_ITERATOR_SOURCE: &str =
    include_str!("../src/control_flow/async_function_for_of_iterator.rs");
const DATA_SOURCE: &str = include_str!("../src/data.rs");
const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const EMISSION_SITES_SOURCE: &str = include_str!("../src/emission_sites.rs");
const PLANNING_SOURCE: &str = include_str!("../src/planning.rs");
const PROTOCOL_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_plain_async_sync_for_of_iterator_protocol.js");
const CLOSE_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_plain_async_sync_for_of_iterator_close.js");
const ERROR_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_plain_async_sync_for_of_iterator_errors.js");
const MEMBER_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_plain_async_sync_for_of_member_heads.js");
const PATTERN_FIXTURE: &str = include_str!(
    "../../lila-cli/tests/fixtures/wasm_plain_async_sync_for_of_nonlexical_pattern_heads.js"
);
const LEXICAL_PATTERN_FIXTURE: &str = include_str!(
    "../../lila-cli/tests/fixtures/wasm_plain_async_sync_for_of_lexical_pattern_heads.js"
);
const CAPTURE_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_async_for_of_closure_capture.js");
const ITERATOR_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/iterator.rs");
const FUNCTION_CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/functions.rs");
const CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/synchronous-array-for-of-iterator-protocol.md"
);
const MEMBER_CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/plain-async-synchronous-for-of-member-heads.md"
);
const PATTERN_CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/plain-async-synchronous-for-of-nonlexical-pattern-heads.md"
);
const LEXICAL_PATTERN_CONTRACT: &str = include_str!(
    "../../../docs/rust-rewrite/contracts/plain-async-synchronous-for-of-lexical-pattern-heads.md"
);
const README: &str = include_str!("../../../README.md");
const TASK: &str = include_str!("../../../tasks/15-generators-iterators-resource-management.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

fn positions_in_order(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let offset = source[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing marker: {marker}"));
        cursor += offset + marker.len();
    }
}

fn compact(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn rust_source_occurrences(root: &Path, needle: &str) -> usize {
    fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return rust_source_occurrences(&path, needle);
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
fn resumable_sync_for_of_emitter_has_one_private_child_owner() {
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("\nmod async_function_for_of_iterator;\n")
            .count(),
        1
    );
    assert!(!CONTROL_FLOW_SOURCE.contains("pub mod async_function_for_of_iterator;"));
    assert!(!CONTROL_FLOW_SOURCE.contains("pub(crate) mod async_function_for_of_iterator;"));
    for source in [CONTROL_FLOW_SOURCE, ASYNC_FUNCTION_FOR_OF_ITERATOR_SOURCE] {
        assert!(!source.contains("include!("));
        assert!(!source.contains("#[path"));
    }

    let owner_declaration = "    pub(crate) fn compile_async_function_for_of_iterator(";
    assert_eq!(
        ASYNC_FUNCTION_FOR_OF_ITERATOR_SOURCE
            .matches(owner_declaration)
            .count(),
        1
    );
    assert!(!ASYNC_FUNCTION_FOR_OF_ITERATOR_SOURCE
        .contains("    pub(super) fn compile_async_function_for_of_iterator("));
    assert!(!ASYNC_FUNCTION_FOR_OF_ITERATOR_SOURCE
        .contains("    pub fn compile_async_function_for_of_iterator("));
    assert!(!CONTROL_FLOW_SOURCE.contains("fn compile_async_function_for_of_iterator("));
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("self.compile_async_function_for_of_iterator(iterable, plan, function)?;")
            .count(),
        1
    );
    assert_eq!(
        EMISSION_SITES_SOURCE
            .matches("FunctionBuilder::compile_async_function_for_of_iterator")
            .count(),
        1
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        rust_source_occurrences(&source_root, "mod async_function_for_of_iterator;"),
        1
    );
    assert_eq!(
        rust_source_occurrences(&source_root, "fn compile_async_function_for_of_iterator("),
        1
    );
    assert_eq!(
        rust_source_occurrences(&source_root, "compile_async_function_for_of_iterator"),
        3
    );
}

#[test]
fn closed_plan_couples_the_iterator_record_body_split_states_and_environments() {
    assert!(IR_SOURCE.contains(
        "#[must_use = \"a resumable synchronous for-of plan must be attached to its statement\"]"
    ));
    let head = bounded(
        IR_SOURCE,
        "pub(crate) enum AsyncFunctionForOfIteratorHeadIr {",
        "/// Where one resumable synchronous `for-of` stores IteratorValue.",
    );
    for marker in [
        "Binding(ForOfAssignmentIr)",
        "PreparedAssignment { value_name: String }",
        "LexicalPattern {",
        "mode: BindingMode",
        "value_name: String",
        "iteration_storage_names: Vec<String>",
        "tdz_placeholder_names: Vec<String>",
        "initialization: Vec<StatementIr>",
    ] {
        assert!(head.contains(marker), "closed head input: {marker}");
    }

    let value_storage = bounded(
        IR_SOURCE,
        "pub enum AsyncFunctionForOfIteratorValueStorageIr {",
        "pub(crate) enum AsyncFunctionForOfIteratorEnvironmentError {",
    );
    for variant in [
        "Activation(ForOfAssignmentIr)",
        "IterationEnvironment(ForOfAssignmentIr)",
        "EntryLocal { name: String }",
    ] {
        assert!(value_storage.contains(variant), "value storage: {variant}");
    }

    let plan = bounded(
        IR_SOURCE,
        "pub struct AsyncFunctionForOfIteratorPlanIr {",
        "pub struct AsyncForOfIteratorPlanIr {",
    );
    for field in [
        "value_storage: AsyncFunctionForOfIteratorValueStorageIr",
        "value_mode: BindingMode",
        "record: IteratorRecordIr",
        "head_environment: Option<ForInOfEnvironmentIr>",
        "iteration_environment: ResumableLoopIterationEnvironmentIr",
        "before_await: Vec<StatementIr>",
        "await_statement: Box<StatementIr>",
        "after_await: Vec<StatementIr>",
        "entry_state: u32",
        "resume_state: u32",
        "exit_state: u32",
    ] {
        assert!(plan.contains(field), "{field}");
        assert!(!plan.contains(&format!("pub {field}")), "public {field}");
    }
    assert!(!plan.contains("binding: ForOfAssignmentIr"));
    assert!(plan.contains("pub(crate) fn new("));
    assert!(!plan.contains("pub fn new("));
    assert!(plan.contains("head: AsyncFunctionForOfIteratorHeadIr"));
    let head_derivation = bounded(
        plan,
        "let (value_storage, value_mode, iteration_environment, mut initialization) = match head {",
        "        initialization.append(&mut before_await);",
    );
    for variant in [
        "AsyncFunctionForOfIteratorHeadIr::Binding(binding)",
        "AsyncFunctionForOfIteratorHeadIr::PreparedAssignment { value_name }",
        "AsyncFunctionForOfIteratorHeadIr::LexicalPattern {",
    ] {
        assert!(
            head_derivation.contains(variant),
            "head derivation: {variant}"
        );
    }
    assert!(!head_derivation.contains("_ =>"));
    positions_in_order(
        plan,
        &[
            "let StatementIr::AsyncAwait",
            "*suspend_state != entry_state",
            "*resume_state != expected_resume_state",
            "let exit_state = resume_state",
            ".checked_add(1)",
            "let (value_storage, value_mode, iteration_environment, mut initialization) = match head",
            "AsyncFunctionForOfIteratorHeadIr::PreparedAssignment",
            "AsyncFunctionForOfIteratorHeadIr::LexicalPattern",
            "AsyncFunctionForOfIteratorPlanError::CapturedTdzEnvironment",
            "initialization.append(&mut before_await)",
            "Ok(Self",
        ],
    );
    for accessor in [
        "pub fn value_storage(&self) -> &AsyncFunctionForOfIteratorValueStorageIr",
        "pub fn value_name(&self) -> &str",
        "pub fn value_mode(&self) -> BindingMode",
    ] {
        assert!(plan.contains(accessor), "plan accessor: {accessor}");
    }

    let statement = bounded(
        IR_SOURCE,
        "    AsyncFunctionForOfIterator {",
        "    ForInArray {",
    );
    assert!(statement.contains("iterable: TypedExpr"));
    assert!(statement.contains("plan: AsyncFunctionForOfIteratorPlanIr"));
    assert!(!statement.contains("body:"));
    assert!(!statement.contains("lexical_environment:"));
}

#[test]
fn lowering_allocates_typed_record_slots_and_never_synthesizes_an_array_walk() {
    for retired in [
        "AsyncForOfArrayWalkForm",
        "lower_async_for_of_array_with_body_await",
        "ARRAY_INDEX_WALK_RESUMABLE",
    ] {
        assert!(!IR_SOURCE.contains(retired), "IR still contains {retired}");
        assert!(
            !LOWERING_SOURCE.contains(retired),
            "lowering still contains {retired}"
        );
        assert!(
            !OBLIGATIONS_SOURCE.contains(retired),
            "obligations still contain {retired}"
        );
        assert!(
            !CONTROL_FLOW_SOURCE.contains(retired),
            "backend still contains {retired}"
        );
        assert!(
            !ASYNC_FUNCTION_FOR_OF_ITERATOR_SOURCE.contains(retired),
            "resumable backend still contains {retired}"
        );
        assert!(
            !PLANNING_SOURCE.contains(retired),
            "planner still contains {retired}"
        );
    }

    let lowerer = bounded(
        LOWERING_SOURCE,
        "    fn lower_async_function_for_of_iterator_with_body_await(",
        "    /// Lowers a `for`-`of` head.",
    );
    positions_in_order(
        lowerer,
        &[
            "Self::split_resumable_loop_body(body)",
            "IteratorRecordIr::new(",
            "self.alloc_iterator_slot()",
            "self.alloc_next_method_slot()",
            "self.alloc_done_slot()",
            "AsyncFunctionForOfIteratorPlanIr::new(",
            "self.current_async_resume_state = Some(plan.exit_state())",
            "ForOfLoweringIr::async_function_iterator(iterable, plan, body_kind)",
        ],
    );
    assert!(!lowerer.contains("PropertyKeyIr::ArrayLength"));
    assert!(!lowerer.contains("PropertyKeyIr::ArrayIndex"));
    assert!(lowerer.contains("head: AsyncFunctionForOfIteratorHeadIr"));
    let activation_ownership = bounded(
        lowerer,
        "        match plan.value_storage() {",
        "        self.current_async_resume_state = Some(plan.exit_state());",
    );
    for variant in [
        "AsyncFunctionForOfIteratorValueStorageIr::Activation(binding)",
        "AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(_)",
        "AsyncFunctionForOfIteratorValueStorageIr::EntryLocal { .. }",
    ] {
        assert!(
            activation_ownership.contains(variant),
            "activation ownership: {variant}"
        );
    }
    assert!(!activation_ownership.contains("_ =>"));

    assert!(!LOWERING_SOURCE.contains("does not support let or const pattern heads"));
    let lexical_pattern_classification = bounded(
        LOWERING_SOURCE,
        "        let lexical_pattern_bindings = match pattern_initializer.as_ref() {",
        "        let resumable_sync_head_is_assignment = match head_kind {",
    );
    for marker in [
        "Some((BindingMode::Let | BindingMode::Const, pattern))",
        "supported_bound_names(self.interner, &binding)",
        "LexicalForOfPatternBinding",
        "for_of_loop_binding_storage_name(",
        "Some((BindingMode::Var, _)) | None => None",
    ] {
        assert!(
            lexical_pattern_classification.contains(marker),
            "lexical pattern classification: {marker}"
        );
    }

    let bare_assignment_prefix = bounded(
        LOWERING_SOURCE,
        "        let mut pattern_prefix = if let ForOfBareIdentifierHead::AssignmentTarget",
        "        } else if let Some(access) = access_initializer.as_ref() {",
    );
    positions_in_order(
        bare_assignment_prefix,
        &[
            "source_name",
            "ExprIr::Identifier(storage_name.clone())",
            "self.locate_identifier_reference(source_name)",
            "self.lower_located_identifier_assign_value",
            "vec![StatementIr::Expression(assignment)]",
        ],
    );

    let access_assignment_prefix = bounded(
        LOWERING_SOURCE,
        "        } else if let Some(access) = access_initializer.as_ref() {",
        "        } else if let Some(pattern) = assignment_pattern_initializer.as_ref() {",
    );
    positions_in_order(
        access_assignment_prefix,
        &[
            "ExprIr::Identifier(storage_name.clone())",
            "let access = access.clone()",
            "self.lower_property_assign_value(&access, value)",
        ],
    );

    let assignment_pattern_prefix = bounded(
        LOWERING_SOURCE,
        "        } else if let Some(pattern) = assignment_pattern_initializer.as_ref() {",
        "        } else if let Some((pattern_mode, pattern)) = pattern_initializer.as_ref() {",
    );
    positions_in_order(
        assignment_pattern_prefix,
        &[
            "ExprIr::Identifier(storage_name.clone())",
            "self.lower_pattern_assign_value(pattern, value)",
            "vec![StatementIr::Expression(assign)]",
        ],
    );

    let declaration_pattern_prefix = bounded(
        LOWERING_SOURCE,
        "        } else if let Some((pattern_mode, pattern)) = pattern_initializer.as_ref() {",
        "        } else {\n            Vec::new()",
    );
    assert!(declaration_pattern_prefix.contains("*pattern_mode == BindingMode::Var"));
    assert!(declaration_pattern_prefix
        .contains("self.lower_pattern_var_binding_from_value(pattern, init)"));
    positions_in_order(
        declaration_pattern_prefix,
        &[
            "let bindings = lexical_pattern_bindings",
            "let storage_names = bindings",
            "Initialization::Uninitialized(",
            "UninitializedStorage::Allocated",
            ".lower_pattern_lexical_binding_from_value_with_storage_names(",
        ],
    );

    let resumable_head = bounded(
        LOWERING_SOURCE,
        "        if plain_async_await_body {\n            let head = if let (",
        "            return self.lower_async_function_for_of_iterator_with_body_await(",
    );
    positions_in_order(
        resumable_head,
        &[
            "AsyncFunctionForOfIteratorHeadIr::LexicalPattern",
            "iteration_storage_names: bindings",
            "tdz_placeholder_names: bindings",
            "initialization: lexical_pattern_initialization",
            "AsyncFunctionForOfIteratorHeadIr::PreparedAssignment",
            "AsyncFunctionForOfIteratorHeadIr::Binding(ForOfAssignmentIr",
        ],
    );

    let statement_capture_scan = bounded(
        ANALYSIS_SOURCE,
        "    fn scan_statement(",
        "    fn scan_array_pattern_expressions(",
    );
    let for_of_capture_scan = bounded(
        statement_capture_scan,
        "            Statement::ForOfLoop(for_of) => {",
        "            Statement::ForInLoop(for_in) => {",
    );
    let access_capture_scan = bounded(
        for_of_capture_scan,
        "IterableLoopInitializer::Access(access) => {",
        "_ => {}",
    );
    assert!(access_capture_scan.contains("self.scan_property_access("));
    assert!(access_capture_scan.contains("&body_aliases"));
    assert!(ANALYSIS_SOURCE.contains("fn scan_object_assignment_pattern_expressions("));
    let assignment_pattern_scan = bounded(
        ANALYSIS_SOURCE,
        "    fn scan_assignment_pattern_expressions(",
        "    fn scan_expression(",
    );
    assert!(assignment_pattern_scan.contains("Pattern::Array(pattern)"));
    assert!(assignment_pattern_scan.contains("Pattern::Object(pattern)"));
    assert!(assignment_pattern_scan.contains("self.scan_object_assignment_pattern_expressions("));

    let generic_value = LOWERING_SOURCE
        .split_once("// A generic iterator can yield values unrelated to the iterable's")
        .expect("generic iterator value boundary")
        .1
        .split_once("        };")
        .expect("generic iterator value boundary end")
        .0;
    assert!(generic_value.contains("kind: ValueKind::Dynamic"));
    assert!(generic_value.contains("possible_kinds: KindSet::all_runtime_tags()"));
    assert!(generic_value.contains("heap_shape: None"));
    assert!(generic_value.contains("function_targets: FunctionTargetKnowledge::unknown()"));
    assert!(!LOWERING_SOURCE.contains("let iterable_is_array ="));

    assert!(OBLIGATIONS_SOURCE
        .contains("RESUMABLE_SYNC_ITERATOR_PROTOCOL => IteratorProtocolWitness::emitted_by("));
    assert!(OBLIGATIONS_SOURCE.contains("EmissionSite::ResumableSyncForOfIterator"));
    assert!(EMISSION_SITES_SOURCE.contains(
        "EmissionSite::ResumableSyncForOfIterator => {\n            let _ = FunctionBuilder::compile_async_function_for_of_iterator;"
    ));
}

#[test]
fn backend_exhaustively_uses_each_resumable_value_storage_lifetime() {
    let emitter = bounded(
        ASYNC_FUNCTION_FOR_OF_ITERATOR_SOURCE,
        "    pub(crate) fn compile_async_function_for_of_iterator(",
        "\n    }\n}",
    );
    let allocation = bounded(
        emitter,
        "        let entry_local_storage = match plan.value_storage() {",
        "        let iterator_storage = self.allocate_binding(",
    );
    for variant in [
        "AsyncFunctionForOfIteratorValueStorageIr::Activation(binding)",
        "AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(binding)",
        "AsyncFunctionForOfIteratorValueStorageIr::EntryLocal { name }",
    ] {
        assert_eq!(
            allocation.matches(variant).count(),
            1,
            "allocation: {variant}"
        );
    }
    assert!(!allocation.contains("_ =>"));
    positions_in_order(
        allocation,
        &[
            "AsyncFunctionForOfIteratorValueStorageIr::Activation(binding)",
            "self.lookup_binding(&binding.name)",
            "self.allocate_binding(binding.name.clone(), binding.mode, ValueKind::Dynamic)",
            "BindingStorage::EnvSlot",
            "AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(binding)",
            "iteration_environment_owns_binding(plan.head_environment(), &binding.name)",
            "AsyncFunctionForOfIteratorValueStorageIr::EntryLocal { name }",
            "self.allocate_binding(name.clone(), BindingMode::Let, ValueKind::Dynamic)",
            "BindingStorage::Dynamic",
        ],
    );
    assert!(!allocation.contains("storage_without_iteration_environment"));

    let resolution = bounded(
        emitter,
        "        let (value_storage, value_is_entry_local) = match plan.value_storage() {",
        "        let close_frame = self.open_frame(ControlFrameKind::Block, function);",
    );
    for variant in [
        "AsyncFunctionForOfIteratorValueStorageIr::Activation(binding)",
        "AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(binding)",
        "AsyncFunctionForOfIteratorValueStorageIr::EntryLocal { .. }",
    ] {
        assert_eq!(
            resolution.matches(variant).count(),
            1,
            "resolution: {variant}"
        );
    }
    assert!(!resolution.contains("_ =>"));
    positions_in_order(
        resolution,
        &[
            "AsyncFunctionForOfIteratorValueStorageIr::Activation(binding)",
            "self.lookup_binding(&binding.name)",
            "AsyncFunctionForOfIteratorValueStorageIr::IterationEnvironment(binding)",
            "self.lookup_current_scope_binding(&binding.name)",
            "AsyncFunctionForOfIteratorValueStorageIr::EntryLocal { .. }",
            "entry_local_storage.expect(",
        ],
    );
    positions_in_order(
        emitter,
        &[
            "let entry_local_storage = match plan.value_storage()",
            "ResumableLoopIterationEnvironmentIr::FreshPerIteration(environment)",
            "self.emit_enter_lexical_environment(environment, function)?",
            "let (value_storage, value_is_entry_local) = match plan.value_storage()",
        ],
    );

    let entry_write = bounded(
        emitter,
        "        let close_frame = self.open_frame(ControlFrameKind::Block, function);",
        "        for statement in plan.before_await() {",
    );
    positions_in_order(
        entry_write,
        &[
            "plan.entry_state()",
            "self.open_frame(ControlFrameKind::If, function)",
            "if !value_is_entry_local && plan.value_mode() != BindingMode::Var",
            "self.write_binding_from_locals(",
            "if !value_is_entry_local",
            "self.mirror_binding_to_global_object(plan.value_name(), value_storage, function)?",
        ],
    );
}

#[test]
fn backend_steps_only_on_entry_and_closes_only_body_owned_completions() {
    let emitter = bounded(
        ASYNC_FUNCTION_FOR_OF_ITERATOR_SOURCE,
        "    pub(crate) fn compile_async_function_for_of_iterator(",
        "\n    }\n}",
    );
    assert_eq!(
        emitter
            .matches("self.emit_sync_iterator_step_value(")
            .count(),
        1
    );
    assert_eq!(
        emitter
            .matches("let consumer = SyncIteratorConsumer::ForOf;")
            .count(),
        1
    );
    assert_eq!(emitter.matches("&consumer,").count(), 2);

    let acquisition = bounded(
        emitter,
        "        function.instruction(&Instruction::LocalGet(state_local));\n        function.instruction(&Instruction::I64Const(i64::from(plan.entry_state())));",
        "        let break_frame = self.open_frame(ControlFrameKind::Block, function);",
    );
    positions_in_order(
        acquisition,
        &[
            "self.compile_expr_to_locals(",
            "self.emit_get_iterator_from_value_locals(",
            "&consumer",
            "self.write_binding_from_locals(\n            iterator_storage",
            "self.write_binding_from_locals(\n            next_storage",
        ],
    );
    assert!(!acquisition.contains("Instruction::Else"));

    let step = bounded(
        emitter,
        "        let loop_frame = self.open_frame(ControlFrameKind::Loop, function);",
        "        let (value_storage, value_is_entry_local) = match plan.value_storage() {",
    );
    positions_in_order(
        step,
        &[
            "plan.entry_state()",
            "self.open_frame(ControlFrameKind::If, function);",
            "self.emit_sync_iterator_step_value(",
            "self.write_binding_from_locals(\n            done_storage",
            "function.branch_if_to_label(break_frame.label);",
            "ResumableLoopIterationEnvironmentIr::FreshPerIteration(environment)",
            "self.emit_enter_lexical_environment(environment, function)?",
            "function.instruction(&Instruction::Else);",
            "let resumed_iterator_storage = self",
            "self.read_binding_to_locals(\n            resumed_iterator_storage",
            "self.read_binding_to_locals(\n            resumed_next_storage",
        ],
    );

    let body_finalizer = bounded(
        emitter,
        "        let close_frame = self.open_frame(ControlFrameKind::Block, function);",
        "        self.save_current_completion(",
    );
    positions_in_order(
        body_finalizer,
        &[
            "self.finally_stack.push(close_frame)",
            "self.write_binding_from_locals(",
            "for statement in plan.before_await()",
            "self.compile_statement(plan.await_statement(), function)?",
            "for statement in plan.after_await()",
            "self.finally_stack.pop()",
        ],
    );
    assert!(!step.contains("finally_stack.push"));

    let cleanup = bounded(
        emitter,
        "        self.save_current_completion(",
        "        self.emit_set_async_resume_state(activation_local, plan.entry_state(), function);",
    );
    positions_in_order(
        cleanup,
        &[
            "self.emit_leave_lexical_environment(function)",
            "HEAP_ASYNC_ENV_OFFSET",
            "COMPLETION_KIND_THROW",
            "self.emit_iterator_close_preserving_current_throw(",
            "function.instruction(&Instruction::Else);",
            "self.emit_iterator_close(",
            "self.emit_dispatch_async_completion(function)?",
        ],
    );
}

#[test]
fn planner_adds_every_persistent_record_local_to_the_deepest_child() {
    let counter = bounded(
        PLANNING_SOURCE,
        "pub(crate) fn count_statement_temp_locals(statement: &StatementIr) -> usize {",
        "const SYNC_FOR_OF_ITERATOR_PERSISTENT_TEMP_LOCALS:",
    );
    let arm = bounded(
        counter,
        "        StatementIr::AsyncFunctionForOfIterator { iterable, plan } => {",
        "        StatementIr::GeneratorIf {",
    );
    assert!(arm.contains("RESUMABLE_SYNC_FOR_OF_ITERATOR_PERSISTENT_TEMP_LOCALS"));
    assert!(arm.contains("+ count_expr_temp_locals(iterable)"));
    assert!(arm.contains("plan.before_await()"));
    assert!(arm.contains("std::iter::once(plan.await_statement())"));
    assert!(arm.contains(".chain(plan.after_await())"));
    assert!(arm.contains(".map(count_statement_temp_locals)"));
    assert!(arm.contains(".max(FOR_OF_ITERATOR_HELPER_TEMP_LOCALS)"));
    assert!(compact(PLANNING_SOURCE).contains(
        "constRESUMABLE_SYNC_FOR_OF_ITERATOR_PERSISTENT_TEMP_LOCALS:usize=1+11+2+1+2*4;"
    ));

    for consumer in [DATA_SOURCE, EMIT_SOURCE, PLANNING_SOURCE] {
        assert!(
            consumer.contains("StatementIr::AsyncFunctionForOfIterator"),
            "statement traversal omitted the resumable iterator form"
        );
    }
}

#[test]
fn runtime_oracles_cover_acquisition_close_errors_strings_and_fresh_bindings() {
    for marker in [
        "get next()",
        "arrayIteratorMethodReads, 1",
        "arrayNextReads, 1",
        "arrayReturnCalls, 0",
        "return { value: \"4\", done: false }",
        "for (assignedValue of customIterable)",
        "assignedValue, \"assigned-2\"",
        "assignmentIteratorCalls, 1",
        "assignmentNextCalls, 3",
        "for (const value of \"native\")",
        "plain-async-sync-for-of:protocol=ok",
    ] {
        assert!(
            PROTOCOL_FIXTURE.contains(marker),
            "protocol fixture: {marker}"
        );
    }
    for marker in [
        "await Promise.reject(bodyError)",
        "throw bodyCloseError",
        "bodyError,\n      \"body rejection identity\"",
        "same(bodyCloseCalls, 1",
        "return \"unobservable return\"",
        "throw returnCloseError",
        "returnCloseError,\n      \"return close error identity\"",
        "same(returnCloseCalls, 1",
        "plain-async-sync-for-of:close=ok",
    ] {
        assert!(CLOSE_FIXTURE.contains(marker), "close fixture: {marker}");
    }
    for marker in [
        "throw nextError",
        "get done()",
        "get value()",
        "same(nextCloseCalls, 0",
        "same(doneCloseCalls, 0",
        "same(valueCloseCalls, 0",
        "plain-async-sync-for-of:protocol-errors=ok",
    ] {
        assert!(ERROR_FIXTURE.contains(marker), "error fixture: {marker}");
    }
    for marker in [
        "for (staticTarget.value of [3, 5])",
        "for (memberBase()[memberKey()] of memberIterable)",
        "for (this.#value of [7, 9])",
        "for (throwingTarget.value of closingIterable)",
        "for (wrong.#value of privateClosingIterable)",
        "plain-async-sync-for-of:member-heads=ok",
    ] {
        assert!(MEMBER_FIXTURE.contains(marker), "member fixture: {marker}");
    }
    for marker in [
        "for (var [selected = arrayDefault(), ...remaining] of [",
        "for (var { value: objectValue = objectDefault(), ...objectRest } of [",
        "[assignmentSourceKey()]: assignmentTargetBase()[assignmentTargetKey()]",
        "...assignmentRestBase().rest",
        "for ([abruptTarget.value = throwPatternError()] of failingOuterIterable)",
        "plain-async-sync-for-of:nonlexical-pattern-heads=ok",
    ] {
        assert!(
            PATTERN_FIXTURE.contains(marker),
            "pattern fixture: {marker}"
        );
    }
    for marker in [
        "carried = captured + 1",
        "...remaining",
        "beforeClosures.push(function ()",
        "afterClosures.push(function ()",
        "for (let [first = later, later = outerLater] of tdzOuterIterable)",
        "tdzError instanceof ReferenceError",
        "capturedHeadReader = function ()",
        "capturedHeadError instanceof ReferenceError",
        "for (const { locked } of constOuterIterable)",
        "locked = 8",
        "constWriteError instanceof TypeError",
        "[selectObjectPatternKey()]: selected, ...remaining",
        "same(objectRestCloseCalls, 1",
        "for (const [value = throwPatternError()] of abruptOuterIterable)",
        "same(abruptInnerCloseCalls, 1",
        "same(abruptOuterCloseCalls, 1",
        "for (const [] of [emptyArrayInnerIterable])",
        "for (const {} of emptyObjectOuterIterable)",
        "plain-async-sync-for-of:lexical-pattern-heads=ok",
    ] {
        assert!(
            LEXICAL_PATTERN_FIXTURE.contains(marker),
            "lexical pattern fixture: {marker}"
        );
    }
    for marker in [
        "closures.push(() => v)",
        "asyncValues = closures.map((f) => f())",
        "asyncValues.join(\",\") !== \"1,2,3,4,5,6\"",
    ] {
        assert!(
            CAPTURE_FIXTURE.contains(marker),
            "capture fixture: {marker}"
        );
    }

    for fixture in [
        "wasm_plain_async_sync_for_of_iterator_protocol.js",
        "wasm_plain_async_sync_for_of_iterator_close.js",
        "wasm_plain_async_sync_for_of_iterator_errors.js",
        "wasm_plain_async_sync_for_of_member_heads.js",
        "wasm_plain_async_sync_for_of_nonlexical_pattern_heads.js",
        "wasm_plain_async_sync_for_of_lexical_pattern_heads.js",
    ] {
        assert!(ITERATOR_CLI_TESTS.contains(fixture), "CLI test: {fixture}");
    }
    assert!(FUNCTION_CLI_TESTS.contains("wasm_async_for_of_closure_capture.js"));
    for source in [CONTRACT, README, TASK] {
        assert!(source.contains("AsyncFunctionForOfIteratorPlanIr"));
        assert!(source.contains("19/19"));
        assert!(source.contains("18/18"));
        assert!(source.contains("4/4"));
    }
    for source in [MEMBER_CONTRACT, README, TASK] {
        assert!(source.contains("member-reference heads"));
        assert!(source.contains("wasm_plain_async_sync_for_of_member_heads.js"));
    }
    for source in [PATTERN_CONTRACT, README, TASK] {
        assert!(source.contains("assignment patterns and `var` binding patterns"));
        assert!(source.contains("wasm_plain_async_sync_for_of_nonlexical_pattern_heads.js"));
    }
    assert!(LEXICAL_PATTERN_CONTRACT.contains("public storage enum has exactly those three cases"));
    for source in [LEXICAL_PATTERN_CONTRACT, README, TASK] {
        for marker in [
            "wasm_plain_async_sync_for_of_lexical_pattern_heads.js",
            "27/27",
            "28/28",
            "5/5",
        ] {
            assert!(
                source.contains(marker),
                "lexical pattern evidence: {marker}"
            );
        }
    }
}
