const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const ENVIRONMENTS_SOURCE: &str = include_str!("../src/environments.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier = source.find(earlier).expect("earlier operation");
    let later = source.find(later).expect("later operation");
    assert!(earlier < later, "`{earlier}` must precede `{later}`");
}

#[test]
fn resumable_loop_environment_domain_and_activation_offsets_are_exhaustive() {
    let body = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_resumable_async_loop(",
        "    fn compile_async_generator_if(",
    );
    let domain = bounded(
        body,
        "        let fresh_iteration_environment = match iteration_environment {",
        "        let state_local = self.reserve_temp_local();",
    );

    assert!(domain.contains("ResumableLoopIterationEnvironmentIr::StorageOnly => None"));
    assert!(domain.contains("ResumableLoopIterationEnvironmentIr::FreshPerIteration(environment)"));
    assert!(!domain.contains("_ =>"));

    let offsets = bounded(
        body,
        "        let activation_environment_offset = match self",
        "        let fresh_iteration_environment = match iteration_environment {",
    );
    for offset in [
        "HEAP_ASYNC_ENV_OFFSET",
        "HEAP_ASYNC_GENERATOR_LEXICAL_ENV_OFFSET",
        "HEAP_GENERATOR_ENV_OFFSET",
    ] {
        assert!(offsets.contains(offset));
    }
    assert!(!offsets.contains("_ =>"));
}

#[test]
fn resume_attaches_the_saved_record_then_restores_parent_before_update() {
    let body = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_resumable_async_loop(",
        "    fn compile_async_generator_if(",
    );
    let resume = bounded(
        body,
        "        let resume_cleanup_frame =",
        "        if let Some(update) = update {",
    );

    assert_eq!(
        resume
            .matches("begin_existing_lexical_environment_scope(environment)")
            .count(),
        1
    );
    assert!(!resume.contains("emit_enter_lexical_environment(environment"));
    assert_eq!(
        resume
            .matches("self.emit_leave_lexical_environment(function);")
            .count(),
        1
    );
    assert_before(
        resume,
        "begin_existing_lexical_environment_scope(environment)",
        "environment_depth: self.environment_depth",
    );
    assert_before(
        resume,
        "environment_depth: self.environment_depth",
        "self.compile_statement(suspension_statement, function)?",
    );
    assert_before(
        resume,
        "function.instruction(&Instruction::End);",
        "self.emit_leave_lexical_environment(function);",
    );
    assert_before(
        resume,
        "self.emit_leave_lexical_environment(function);",
        "activation_environment_offset",
    );
    assert_before(
        resume,
        "activation_environment_offset",
        "self.emit_dispatch_async_completion(function)?",
    );

    let attach = bounded(
        ENVIRONMENTS_SOURCE,
        "    pub(crate) fn begin_existing_lexical_environment_scope(",
        "    pub(crate) fn emit_leave_lexical_environment(",
    );
    assert!(!attach.contains("emit_heap_alloc_const"));
}

#[test]
fn successful_test_allocates_and_saves_before_binding_initialization() {
    let body = bounded(
        CONTROL_FLOW_SOURCE,
        "    fn compile_resumable_async_loop(",
        "    fn compile_async_generator_if(",
    );
    let entry = bounded(
        body,
        "        let entry_cleanup_frame =",
        "        function.instruction(&Instruction::Else);",
    );

    assert_eq!(
        entry
            .matches("self.emit_enter_lexical_environment(environment, function)?")
            .count(),
        1
    );
    assert_eq!(
        entry
            .matches("self.emit_leave_lexical_environment(function);")
            .count(),
        1
    );
    assert_before(
        entry,
        "self.emit_enter_lexical_environment(environment, function)?",
        "environment_depth: self.environment_depth",
    );
    assert_before(
        entry,
        "environment_depth: self.environment_depth",
        "activation_environment_offset",
    );
    assert_before(
        entry,
        "activation_environment_offset",
        "self.initialize_direct_lexical_bindings(before_suspension, function);",
    );
    assert_before(
        entry,
        "self.initialize_direct_lexical_bindings(after_suspension, function);",
        "for statement in before_suspension {",
    );
    assert_before(
        entry,
        "for statement in before_suspension {",
        "self.compile_statement(suspension_statement, function)?",
    );
    assert_before(
        entry,
        "function.instruction(&Instruction::End);",
        "self.emit_leave_lexical_environment(function);",
    );
    let cleanup = entry
        .split_once("self.emit_leave_lexical_environment(function);")
        .expect("entry cleanup leave")
        .1;
    assert!(
        cleanup.contains("activation_environment_offset"),
        "the parent must be published after the single cleanup leave"
    );
}
