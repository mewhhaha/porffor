const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/async-function-resume-completion.md");
const TASK: &str = include_str!("../../../tasks/14-promises-jobs-async.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn activation_owner_is_must_use_and_capability_free() {
    let declaration = bounded(
        CONTROL_FLOW_SOURCE,
        "/// The activation layout shared",
        "/// An identifier PutValue",
    );
    assert!(declaration.contains("#[must_use ="));
    assert!(!declaration.contains("#[derive("));
    assert_eq!(
        without_whitespace(bounded(declaration, "enum AsyncContinuationOwner {", "}")),
        "AsyncFunction,AsyncGenerator,"
    );
    for capability in [
        "Clone",
        "Copy",
        "Debug",
        "Default",
        "PartialEq",
        "Eq",
        "PartialOrd",
        "Ord",
        "Hash",
    ] {
        assert!(
            !CONTROL_FLOW_SOURCE.contains(&format!("impl {capability} for AsyncContinuationOwner"))
        );
    }
}

#[test]
fn one_borrowed_owner_selects_typed_activation_resume_and_await_policies() {
    let decoder = without_whitespace(bounded(
        CONTROL_FLOW_SOURCE,
        "fn emit_load_async_continuation_resume(",
        "fn emit_activation_async_dispose_await_reactions(",
    ));
    assert!(decoder.contains("owner:&AsyncContinuationOwner,"));
    assert_eq!(decoder.matches("matchowner{").count(), 1);
    assert!(!decoder.contains("_=>"));
    for marker in [
        "AsyncActivationSchema::RESUME_COMPLETION",
        "AsyncActivationSchema::RESUME_VALUE",
        "AwaitCompletionKind::Normal",
        "AwaitCompletionKind::Throw",
        "AsyncGeneratorActivationSchema::RESUME_KIND",
        "AsyncGeneratorActivationSchema::RESUME_VALUE",
        "AsyncGeneratorResumeKind::Fulfill",
        "AsyncGeneratorResumeKind::Reject",
    ] {
        assert!(decoder.contains(marker), "{marker}");
    }
    assert_eq!(
        decoder.matches("Instruction::Unreachable").count(),
        2,
        "both continuation protocols strictly decode their allowed completion pair"
    );
    assert_eq!(
        decoder
            .matches(".read_into(&stored,value,schema,function)")
            .count(),
        2
    );
    let awaiter = without_whitespace(bounded(
        CONTROL_FLOW_SOURCE,
        "fn emit_async_continuation_await(",
        "fn emit_dispatch_activation_async_dispose_completion(",
    ));
    assert!(awaiter.contains("owner:&AsyncContinuationOwner,"));
    assert_eq!(awaiter.matches("matchowner{").count(), 1);
    assert!(!awaiter.contains("_=>"));
    for marker in [
        "self.emit_save_resumable_environment(function)?;",
        "self.emit_async_await_reactions(&activation,value,function)?;",
        "self.emit_async_generator_await_reactions(&activation,value,function)?;",
        "AsyncGeneratorBodyStatus::Await",
        "AsyncGeneratorExecutionState::Executing",
    ] {
        assert!(awaiter.contains(marker), "{marker}");
    }
    let compiler = without_whitespace(bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) fn compile_async_for_of_iterator(",
        "pub(crate) fn compile_async_disposable_for_of_iterator(",
    ));
    assert_eq!(compiler.matches("letowner=match").count(), 1);
    assert!(compiler.contains("!plan.requires_plain_async_activation()"));
    assert_eq!(
        compiler
            .matches("self.emit_load_async_continuation_resume(&owner,")
            .count(),
        2
    );
    assert_eq!(
        compiler
            .matches("self.emit_async_continuation_await(&owner,")
            .count(),
        2
    );
    assert!(!compiler.contains("owner.clone()"));
    let save = without_whitespace(bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) fn emit_save_resumable_environment(",
        "fn compile_resumable_generator_if(",
    ));
    assert!(save.contains("InvocationFrameSchema::LEXICAL_ENVIRONMENT"));
    assert!(save.contains("GcOperand::reference(self.current_environment(),schema)"));
}

#[test]
fn contract_and_task_record_the_capability_boundary_and_nonclaims() {
    for evidence in [CONTRACT, TASK] {
        let evidence = without_whitespace(evidence);
        assert!(evidence.contains("capability-free"));
        assert!(evidence.contains("must-use"));
        assert!(evidence.contains("fourborrowedexhaustiveprojections"));
        assert!(evidence.contains("twoborrowedstrict-decodercalls"));
        assert!(evidence.contains("noemittedWasmorruntimebehavior"));
        assert!(evidence.contains("BatchAB"));
    }
}

#[test]
fn captured_iteration_cleanup_consumes_the_saved_gc_environment_authority() {
    let source = include_str!("../src/control_flow/for_await_iteration_environment.rs");
    assert!(!source.contains("#[derive("));
    assert_eq!(source.matches("#[must_use =").count(), 2);
    assert!(!source.contains("pub(crate) struct"));
    assert!(source.contains("environment: GcLocal<Environment, Nullable>"));
    let enter = without_whitespace(bounded(
        source,
        "pub(super) fn enter_suspended_for_await_iteration_environment(",
        "pub(super) fn leave_suspended_for_await_iteration_environment(",
    ));
    assert!(enter.contains("saved:SavedForAwaitIterationEnvironment,"));
    assert_eq!(
        enter
            .matches("emit_allocate_lexical_environment_record(")
            .count(),
        1
    );
    assert_eq!(
        enter
            .matches("begin_existing_lexical_environment_scope(")
            .count(),
        1
    );
    assert!(!enter.contains("emit_enter_lexical_environment("));
    let markers = [
        "self.emit_allocate_lexical_environment_record(",
        "Instruction::Else",
        "saved.environment.load(schema,function)",
        "Instruction::End",
        "self.begin_existing_lexical_environment_scope(",
        "saved.environment.clear(function);",
        "self.emit_save_resumable_environment(function)?;",
        "self.finally_stack.push(cleanup)",
    ];
    let mut cursor = 0;
    for marker in markers {
        cursor += enter[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing {marker}"))
            + marker.len();
    }
    let leave = without_whitespace(
        source
            .split_once("pub(super) fn leave_suspended_for_await_iteration_environment(")
            .unwrap()
            .1,
    );
    assert!(leave.contains("active:ActiveForAwaitIterationEnvironment,"));
    assert!(leave.contains("assert_eq!(self.finally_stack.pop(),Some(active.cleanup))"));
    assert_eq!(leave.matches("emit_leave_lexical_environment(").count(), 1);
    let leave_position = leave.find("emit_leave_lexical_environment(").unwrap();
    let save_position = leave.find("emit_save_resumable_environment(").unwrap();
    let dispatch_position = leave.find("emit_dispatch_current_completion(").unwrap();
    assert!(leave_position < save_position && save_position < dispatch_position);
}

#[test]
fn allocation_does_not_attach_the_compiler_binding_view() {
    let source = include_str!("../src/environments.rs");
    let allocate = bounded(
        source,
        "pub(crate) fn emit_allocate_lexical_environment_record(",
        "pub(crate) fn begin_existing_lexical_environment_scope(",
    );
    assert_eq!(
        allocate
            .matches("self.emit_initialize_named_environment_header(")
            .count(),
        1
    );
    assert_eq!(
        allocate
            .matches("self.emit_allocate_environment_cells(")
            .count(),
        1
    );
    for forbidden in [
        "self.begin_existing_lexical_environment_scope(",
        "self.push_scope(",
        "self.binding_scopes",
        "self.environment_depth",
        "emit_heap_alloc_const(",
    ] {
        assert!(!allocate.contains(forbidden), "{forbidden}");
    }
    let ordinary = bounded(
        source,
        "pub(crate) fn emit_enter_lexical_environment(",
        "pub(crate) fn emit_allocate_lexical_environment_record(",
    );
    assert_eq!(
        ordinary
            .matches("self.emit_allocate_lexical_environment_record(")
            .count(),
        1
    );
    assert_eq!(
        ordinary
            .matches("self.begin_existing_lexical_environment_scope(")
            .count(),
        1
    );
}
