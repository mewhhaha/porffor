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
fn activation_layout_is_must_use_and_capability_free() {
    let declaration = bounded(
        CONTROL_FLOW_SOURCE,
        "/// The activation layout shared by the two execution kinds",
        "impl ForAwaitActivationLayout {",
    );
    assert!(declaration.contains(
        "#[must_use = \"a for-await activation layout must be consumed by all suspension policies\"]"
    ));
    assert!(!declaration.contains("#[derive("));
    assert_eq!(
        without_whitespace(bounded(declaration, "enum ForAwaitActivationLayout {", "}",)),
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
            !CONTROL_FLOW_SOURCE
                .contains(&format!("impl {capability} for ForAwaitActivationLayout")),
            "for-await activation layout must not manually implement {capability}"
        );
    }
}

#[test]
fn one_borrowed_layout_owns_every_suspension_policy() {
    let projection = bounded(
        CONTROL_FLOW_SOURCE,
        "impl ForAwaitActivationLayout {",
        "impl DestructuringIteratorLocals {",
    );
    for method in [
        "const fn environment_offset(&self) -> u64",
        "const fn resume_state_offset(&self) -> u64",
        "const fn resume_payload_offset(&self) -> u64",
        "const fn resume_tag_offset(&self) -> u64",
    ] {
        assert!(projection.contains(method), "missing borrowed {method}");
    }
    assert_eq!(projection.matches("match self {").count(), 4);
    assert!(!projection.contains("is_async_generator"));
    assert!(!projection.contains("-> bool"));

    let decoder = bounded(
        CONTROL_FLOW_SOURCE,
        "fn emit_load_for_await_resume_is_throw(",
        "pub(crate) fn compile_async_for_of_iterator(",
    );
    assert!(decoder.contains("layout: &ForAwaitActivationLayout"));
    assert_eq!(decoder.matches("match layout {").count(), 1);
    assert!(!decoder.contains("layout: ForAwaitActivationLayout"));

    let compiler = bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) fn compile_async_for_of_iterator(",
        "pub(crate) fn compile_async_disposable_for_of_iterator(",
    );
    assert_eq!(compiler.matches("let resume_layout = match").count(), 1);
    assert_eq!(compiler.matches("match &resume_layout {").count(), 4);
    let normalized_compiler = without_whitespace(compiler);
    assert_eq!(
        normalized_compiler
            .matches("emit_load_for_await_resume_is_throw(&resume_layout,")
            .count(),
        2
    );
    assert_eq!(
        compiler
            .matches("ForAwaitActivationLayout::AsyncFunction")
            .count(),
        5
    );
    assert_eq!(
        compiler
            .matches("ForAwaitActivationLayout::AsyncGenerator")
            .count(),
        5
    );
    assert!(!compiler.contains("is_async_generator"));
    assert!(!compiler.contains("match resume_layout"));
    assert!(!compiler.contains("resume_layout.clone()"));
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
fn captured_iteration_cleanup_consumes_the_same_activation_authority() {
    let lifecycle = include_str!("../src/control_flow/for_await_iteration_environment.rs");
    assert!(!lifecycle.contains("#[derive("));
    assert_eq!(lifecycle.matches("#[must_use =").count(), 2);
    assert!(!lifecycle.contains("pub(crate) struct"));
    assert!(lifecycle.contains("environment_offset: layout.environment_offset()"));
    let enter = bounded(
        lifecycle,
        "pub(super) fn enter_suspended_for_await_iteration_environment(",
        "pub(super) fn leave_suspended_for_await_iteration_environment(",
    );
    assert!(enter.contains("saved: SavedForAwaitIterationEnvironment"));
    assert!(enter.contains("environment_offset: saved.environment_offset"));
    assert!(enter.contains("activation_local: saved.activation_local"));
    assert_eq!(enter.matches("emit_enter_lexical_environment(").count(), 1);
    assert!(
        enter.find("emit_enter_lexical_environment(").unwrap()
            < enter.find("self.finally_stack.push(cleanup)").unwrap(),
        "the cleanup must record the child depth, not unwind the child twice"
    );
    let leave = lifecycle
        .split_once("pub(super) fn leave_suspended_for_await_iteration_environment(")
        .unwrap()
        .1;
    assert!(leave.contains("active: ActiveForAwaitIterationEnvironment"));
    assert!(leave.contains("assert_eq!(self.finally_stack.pop(), Some(active.cleanup))"));
    assert_eq!(leave.matches("emit_leave_lexical_environment(").count(), 1);
    assert!(leave.contains("active.activation_local"));
    assert!(leave.contains("active.environment_offset"));
    let leave_position = leave.find("emit_leave_lexical_environment(").unwrap();
    let publish_position = leave.find("store_i64_local_at_offset(").unwrap();
    let dispatch_position = leave.find("emit_dispatch_current_completion(").unwrap();
    assert!(leave_position < publish_position && publish_position < dispatch_position);
}
