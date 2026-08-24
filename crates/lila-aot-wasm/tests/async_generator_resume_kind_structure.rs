use std::fs;
use std::path::{Path, PathBuf};

const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const DELEGATION_SOURCE: &str = include_str!("../src/generator_delegation.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/async-generator-resume-kind-word.md");

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
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .replace(",)", ")")
        .replace(",]", "]")
}

fn normalized_code(source: &str) -> String {
    normalized(
        &source
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn unique_position(body: &str, needle: &str, label: &str) -> usize {
    assert_eq!(
        body.matches(needle).count(),
        1,
        "{label} must occur exactly once"
    );
    body.find(needle)
        .unwrap_or_else(|| panic!("missing sentinel: {label}"))
}

fn collect_rust_sources(directory: &Path, sources: &mut Vec<(PathBuf, String)>) {
    let mut paths = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
        .map(|entry| entry.expect("failed to read source entry").path())
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        if path.is_dir() {
            collect_rust_sources(&path, sources);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            sources.push((path, source));
        }
    }
}

fn async_generator_builtin_owner() -> &'static str {
    bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::AsyncGeneratorPrototypeNext\n            | StandardBuiltinId::AsyncGeneratorPrototypeReturn\n            | StandardBuiltinId::AsyncGeneratorPrototypeThrow => {",
        "StandardBuiltinId::ArrayIteratorNext => {",
    )
}

fn async_generator_delegation_owner() -> &'static str {
    bounded(
        DELEGATION_SOURCE,
        "pub(crate) fn compile_async_generator_delegation(",
        "pub(crate) fn compile_generator_delegation(",
    )
}

#[test]
fn resume_kind_is_the_exact_five_value_activation_domain() {
    let declaration = bounded(
        HEAP_SOURCE,
        "pub(crate) enum AsyncGeneratorResumeKind {",
        "}\n\nimpl AsyncGeneratorResumeKind {",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        ["Normal,", "Return,", "Throw,", "Fulfill,", "Reject,"],
    );

    let policy = normalized_code(bounded(
        HEAP_SOURCE,
        "impl AsyncGeneratorResumeKind {",
        "/// One strictly validated snapshot of an async-generator resume kind.",
    ));
    assert_eq!(
        policy,
        normalized_code(
            r#"
            const ALL: [Self; 5] = [
                Self::Normal,
                Self::Return,
                Self::Throw,
                Self::Fulfill,
                Self::Reject,
            ];

            const fn word(self) -> u64 {
                match self {
                    Self::Normal => 0,
                    Self::Return => 1,
                    Self::Throw => 2,
                    Self::Fulfill => 3,
                    Self::Reject => 4,
                }
            }
        }
            "#,
        )
    );
    assert_eq!(policy.matches("=>").count(), 5);
    assert!(!policy.contains("_=>"));
    assert!(!policy.contains("unreachable!"));

    let domain = bounded(
        HEAP_SOURCE,
        "/// The closed completion kind supplied when an async-generator body resumes.",
        "pub(crate) const GENERATOR_RESUME_STATE_INITIALIZING",
    );
    assert!(!domain.contains("repr("));
    assert!(!domain.contains("CloseThrow"));
    assert!(!HEAP_SOURCE.contains("impl Default for AsyncGeneratorResumeKind"));
    assert!(!HEAP_SOURCE.contains("impl From<u64> for AsyncGeneratorResumeKind"));
    assert!(!HEAP_SOURCE.contains("impl From<i64> for AsyncGeneratorResumeKind"));
    assert!(!HEAP_SOURCE.contains("impl From<bool> for AsyncGeneratorResumeKind"));
    for raw_name in [
        "ASYNC_GENERATOR_RESUME_KIND_NORMAL",
        "ASYNC_GENERATOR_RESUME_KIND_RETURN",
        "ASYNC_GENERATOR_RESUME_KIND_THROW",
        "ASYNC_GENERATOR_RESUME_KIND_FULFILL",
        "ASYNC_GENERATOR_RESUME_KIND_REJECT",
    ] {
        assert!(!HEAP_SOURCE.contains(raw_name));
    }

    let token = bounded(
        HEAP_SOURCE,
        "/// One strictly validated snapshot of an async-generator resume kind.",
        "pub(crate) const GENERATOR_RESUME_STATE_INITIALIZING",
    );
    assert!(token.contains("#[must_use"));
    assert_eq!(
        token
            .matches("pub(crate) struct LoadedAsyncGeneratorResumeKind(u32);")
            .count(),
        1
    );
    assert!(!token.contains("#[derive(Clone"));
    assert!(!token.contains("#[derive(Copy"));
    assert!(!HEAP_SOURCE.contains("impl LoadedAsyncGeneratorResumeKind"));
    assert!(!HEAP_SOURCE.contains("Deref for LoadedAsyncGeneratorResumeKind"));
    assert_eq!(
        HEAP_SOURCE
            .matches("LoadedAsyncGeneratorResumeKind(")
            .count(),
        2,
        "only the tuple declaration and strict loader may construct the token"
    );
}

#[test]
fn resume_kind_heap_boundary_is_private_strict_and_opaque() {
    assert_eq!(
        HEAP_SOURCE
            .matches("const HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET: u64 = 104;")
            .count(),
        1
    );
    assert!(!HEAP_SOURCE.contains("pub(crate) const HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET"));
    assert_eq!(
        HEAP_SOURCE
            .matches("HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET")
            .count(),
        4,
        "only declaration, layout, typed store and strict load may name the activation offset"
    );
    for source in [
        FUNCTIONS_SOURCE,
        PROMISE_SOURCE,
        STANDARD_SOURCE,
        CONTROL_FLOW_SOURCE,
        DELEGATION_SOURCE,
    ] {
        assert!(!source.contains("HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET"));
    }

    let store = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_store_async_generator_resume_kind(",
        "/// Load and strictly validate one async-generator resume-kind snapshot.",
    );
    assert!(store.contains("kind: AsyncGeneratorResumeKind,"));
    assert_eq!(
        store
            .matches("HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET")
            .count(),
        1
    );
    assert_eq!(store.matches("kind.word()").count(), 1);

    let loader = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_load_async_generator_resume_kind_strict(",
        "/// Emit one comparison against a strictly loaded resume-kind word.",
    );
    assert!(loader.contains(") -> LoadedAsyncGeneratorResumeKind {"));
    assert_eq!(loader.matches("reserve_temp_local()").count(), 1);
    assert_eq!(
        loader
            .matches("HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET")
            .count(),
        1
    );
    assert_eq!(
        loader
            .matches("for kind in AsyncGeneratorResumeKind::ALL")
            .count(),
        1
    );
    assert_eq!(loader.matches("kind.word()").count(), 1);
    assert_eq!(loader.matches("Instruction::Else").count(), 1);
    assert_eq!(loader.matches("Instruction::Unreachable").count(), 1);
    assert_eq!(
        loader
            .matches("LoadedAsyncGeneratorResumeKind(kind_word_local)")
            .count(),
        1
    );
    assert!(!loader.contains("_ =>"));

    let comparison = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_async_generator_resume_kind_equals(",
        "/// Copy a validated activation resume kind into the widened delegation",
    );
    assert!(comparison.contains("loaded: &LoadedAsyncGeneratorResumeKind,"));
    assert!(comparison.contains("expected: AsyncGeneratorResumeKind,"));
    assert_eq!(comparison.matches("LocalGet(loaded.0)").count(), 1);
    assert_eq!(comparison.matches("expected.word()").count(), 1);

    let release = bounded(
        HEAP_SOURCE,
        "pub(crate) fn release_loaded_async_generator_resume_kind(",
        "/// Initialize a Promise record in the sole valid non-terminal state.",
    );
    assert!(release.contains("loaded: LoadedAsyncGeneratorResumeKind,"));
    assert!(!release.contains("&LoadedAsyncGeneratorResumeKind"));
    assert_eq!(release.matches("release_temp_local(loaded.0)").count(), 1);
}

#[test]
fn activation_domain_crosses_into_delegation_only_through_the_named_bridge() {
    let copy = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_copy_async_generator_resume_kind_to_delegate_pending_kind(",
        "/// Initialize the delegation pending-kind transport from one typed resume",
    );
    assert!(copy.contains("loaded: &LoadedAsyncGeneratorResumeKind,"));
    assert!(copy.contains("pending_kind_local: u32,"));
    assert_eq!(copy.matches("LocalGet(loaded.0)").count(), 1);
    assert_eq!(copy.matches("LocalSet(pending_kind_local)").count(), 1);
    assert!(!copy.contains("HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET"));

    let initialization = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_initialize_async_generator_delegate_pending_kind_from_resume_kind(",
        "/// Compare the widened delegation pending-kind transport with one resume",
    );
    assert!(initialization.contains("pending_kind_local: u32,"));
    assert!(initialization.contains("kind: AsyncGeneratorResumeKind,"));
    assert_eq!(initialization.matches("kind.word()").count(), 1);
    assert_eq!(
        initialization
            .matches("LocalSet(pending_kind_local)")
            .count(),
        1
    );
    assert!(!initialization.contains("LoadedAsyncGeneratorResumeKind"));
    assert!(!initialization.contains("HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET"));

    let pending_comparison = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_async_generator_delegate_pending_kind_equals_resume_kind(",
        "/// Release the private local owned by a resume-kind snapshot.",
    );
    assert!(pending_comparison.contains("pending_kind_local: u32,"));
    assert!(pending_comparison.contains("expected: AsyncGeneratorResumeKind,"));
    assert!(!pending_comparison.contains("LoadedAsyncGeneratorResumeKind"));
    assert!(!pending_comparison.contains("HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET"));

    let delegation = normalized_code(async_generator_delegation_owner());
    assert_eq!(
        delegation
            .matches("emit_copy_async_generator_resume_kind_to_delegate_pending_kind(")
            .count(),
        1
    );
    assert_eq!(
        delegation
            .matches("emit_async_generator_delegate_pending_kind_equals_resume_kind(")
            .count(),
        10
    );
    assert_eq!(
        delegation
            .matches("emit_initialize_async_generator_delegate_pending_kind_from_resume_kind(")
            .count(),
        1
    );
    assert_eq!(
        delegation
            .matches("ASYNC_GENERATOR_DELEGATE_PENDING_CLOSE_THROW")
            .count(),
        2,
        "the widened pending transport must retain its declaration-independent sentinel uses"
    );
    assert!(!delegation.contains("resume_kind_local"));
    assert!(!delegation.contains("HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET"));
    assert!(!delegation.contains("emit_async_generator_resume_kind_equals("));

    let initialization = unique_position(
        &delegation,
        "self.emit_initialize_async_generator_delegate_pending_kind_from_resume_kind(next_pending_kind_local,AsyncGeneratorResumeKind::Normal,function);",
        "fresh delegation Normal pending-kind initialization",
    );
    let load = unique_position(
        &delegation,
        "self.emit_load_async_generator_resume_kind_strict(activation_local,function)",
        "resumed delegation strict activation load",
    );
    let bridge = unique_position(
        &delegation,
        "self.emit_copy_async_generator_resume_kind_to_delegate_pending_kind(&resume_kind,next_pending_kind_local,function);",
        "validated resume-to-pending bridge",
    );
    let release = unique_position(
        &delegation,
        "self.release_loaded_async_generator_resume_kind(resume_kind);",
        "resumed delegation activation-token release",
    );
    let fulfill = unique_position(
        &delegation,
        "self.emit_async_generator_delegate_pending_kind_equals_resume_kind(next_pending_kind_local,AsyncGeneratorResumeKind::Fulfill,function);",
        "post-join delegation Fulfill route",
    );
    let branch_split = &delegation[initialization..load];
    assert_eq!(branch_split.matches("Instruction::Else").count(), 1);
    let branch_join = &delegation[release..fulfill];
    assert_eq!(branch_join.matches("Instruction::End").count(), 1);
    assert!(initialization < load && load < bridge && bridge < release && release < fulfill);

    let close_throw = delegation
        .rfind("ASYNC_GENERATOR_DELEGATE_PENDING_CLOSE_THROW")
        .expect("missing widened pending close-throw sentinel");
    let post_widen_comparison = delegation[close_throw..]
        .find("emit_async_generator_delegate_pending_kind_equals_resume_kind(")
        .map(|position| position + close_throw)
        .expect("post-sentinel routing must use the widened pending-kind comparison");
    let pending_publication = unique_position(
        &delegation,
        "self.store_i64_local_at_offset(record_local,HEAP_GENERATOR_DELEGATE_PENDING_KIND_OFFSET,next_pending_kind_local,function);",
        "widened pending-kind publication",
    );
    assert!(fulfill < close_throw && close_throw < post_widen_comparison);
    assert!(post_widen_comparison < pending_publication);
}

#[test]
fn resume_kind_has_nine_typed_store_selections_and_five_strict_readers() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);

    let mut totals = [0; 8];
    for (path, source) in sources {
        let relative = path
            .strip_prefix(&source_root)
            .expect("collected source must remain below src")
            .to_string_lossy();
        let expected = match relative.as_ref() {
            "heap.rs" => (4, 1, 1, 1, 1, 1, 1, 1),
            "functions.rs" => (0, 1, 0, 0, 0, 0, 0, 0),
            "builtins/promise.rs" => (0, 7, 0, 0, 0, 1, 0, 0),
            "builtins/standard.rs" => (0, 1, 0, 0, 0, 0, 0, 0),
            "control_flow.rs" => (0, 0, 4, 9, 4, 0, 0, 0),
            "generator_delegation.rs" => (0, 0, 1, 0, 1, 10, 1, 1),
            _ => (0, 0, 0, 0, 0, 0, 0, 0),
        };
        let actual = (
            source
                .matches("HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET")
                .count(),
            source
                .matches("emit_store_async_generator_resume_kind(")
                .count(),
            source
                .matches("emit_load_async_generator_resume_kind_strict(")
                .count(),
            source
                .matches("emit_async_generator_resume_kind_equals(")
                .count(),
            source
                .matches("release_loaded_async_generator_resume_kind(")
                .count(),
            source
                .matches("emit_async_generator_delegate_pending_kind_equals_resume_kind(")
                .count(),
            source
                .matches("emit_copy_async_generator_resume_kind_to_delegate_pending_kind(")
                .count(),
            source
                .matches("emit_initialize_async_generator_delegate_pending_kind_from_resume_kind(")
                .count(),
        );
        assert_eq!(
            actual, expected,
            "unexpected async-generator resume-kind owner in {relative}"
        );
        totals[0] += actual.0;
        totals[1] += actual.1;
        totals[2] += actual.2;
        totals[3] += actual.3;
        totals[4] += actual.4;
        totals[5] += actual.5;
        totals[6] += actual.6;
        totals[7] += actual.7;
    }
    assert_eq!(totals, [4, 10, 6, 10, 6, 12, 2, 2]);

    let allocation = normalized_code(bounded(
        FUNCTIONS_SOURCE,
        "if can_call_async_generator {",
        "if can_call_async {",
    ));
    let resume_initialization = unique_position(
        &allocation,
        "self.emit_store_async_generator_resume_kind(async_generator_activation_local,AsyncGeneratorResumeKind::Normal,function);",
        "typed Normal allocation",
    );
    let body_initialization = unique_position(
        &allocation,
        "self.emit_store_async_generator_body_status(async_generator_activation_local,AsyncGeneratorBodyStatus::Idle,function);",
        "typed Idle body status",
    );
    let execution_initialization = unique_position(
        &allocation,
        "self.emit_store_async_generator_execution_state(async_generator_activation_local,AsyncGeneratorExecutionState::SuspendedStart,function);",
        "typed suspended-start state",
    );
    assert!(resume_initialization < body_initialization);
    assert!(body_initialization < execution_initialization);

    let builtin = normalized_code(async_generator_builtin_owner());
    assert_eq!(
        builtin
            .matches("emit_store_async_generator_resume_kind(")
            .count(),
        1
    );
    for kind in ["Normal", "Return", "Throw"] {
        let selection = format!("AsyncGeneratorResumeKind::{kind}");
        assert_eq!(builtin.matches(selection.as_str()).count(), 1);
    }

    for (start, end, stores) in [
        (
            "fn emit_run_async_generator_await_job(",
            "fn emit_run_async_generator_await_return_job(",
            2,
        ),
        (
            "fn emit_run_async_generator_yield_return_job(",
            "fn emit_run_async_generator_yield_job(",
            2,
        ),
        (
            "fn emit_run_async_generator_yield_job(",
            "pub(crate) fn emit_complete_async_generator_yield(",
            1,
        ),
        (
            "pub(crate) fn emit_complete_async_generator_yield(",
            "fn emit_run_promise_reaction_callback(",
            2,
        ),
    ] {
        let owner = normalized_code(bounded(PROMISE_SOURCE, start, end));
        assert_eq!(
            owner
                .matches("emit_store_async_generator_resume_kind(")
                .count(),
            stores
        );
        assert!(!owner.contains("HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET"));
        assert!(!owner.contains("resume_kind_local"));
    }
}

#[test]
fn strict_readers_route_one_snapshot_and_release_it() {
    for (start, end, comparisons) in [
        (
            "fn compile_async_generator_yield(",
            "fn compile_async_generator_await(",
            3,
        ),
        (
            "fn compile_async_generator_await(",
            "pub(crate) fn compile_statement(",
            3,
        ),
        (
            "fn emit_load_activation_async_dispose_resume_is_throw(",
            "fn emit_activation_async_dispose_await_reactions(",
            2,
        ),
        (
            "fn emit_load_for_await_resume_is_throw(",
            "pub(crate) fn compile_async_for_of_iterator(",
            1,
        ),
    ] {
        let owner = normalized_code(bounded(CONTROL_FLOW_SOURCE, start, end));
        assert_eq!(
            owner
                .matches("emit_load_async_generator_resume_kind_strict(")
                .count(),
            1
        );
        assert_eq!(
            owner
                .matches("emit_async_generator_resume_kind_equals(")
                .count(),
            comparisons
        );
        assert_eq!(
            owner
                .matches("release_loaded_async_generator_resume_kind(")
                .count(),
            1
        );
        assert!(!owner.contains("HEAP_ASYNC_GENERATOR_RESUME_KIND_OFFSET"));
    }

    let delegation = normalized_code(async_generator_delegation_owner());
    let load = unique_position(
        &delegation,
        "self.emit_load_async_generator_resume_kind_strict(activation_local,function)",
        "delegation strict resume-kind load",
    );
    let bridge = unique_position(
        &delegation,
        "self.emit_copy_async_generator_resume_kind_to_delegate_pending_kind(&resume_kind,next_pending_kind_local,function);",
        "delegation pending-kind bridge",
    );
    let release = unique_position(
        &delegation,
        "self.release_loaded_async_generator_resume_kind(resume_kind);",
        "delegation resume-kind release",
    );
    let fulfill = unique_position(
        &delegation,
        "AsyncGeneratorResumeKind::Fulfill",
        "delegation Fulfill route",
    );
    let reject = unique_position(
        &delegation,
        "AsyncGeneratorResumeKind::Reject",
        "delegation Reject route",
    );
    assert!(load < bridge && bridge < release && release < fulfill && fulfill < reject);
    assert_eq!(
        delegation
            .matches("emit_async_generator_resume_kind_equals(")
            .count(),
        0
    );
}

#[test]
fn async_from_sync_rejection_word_is_projected_to_an_i32_condition() {
    let close_on_rejection = normalized_code(bounded(
        PROMISE_SOURCE,
        "fn emit_async_from_sync_close_on_rejection(",
        "pub(crate) fn emit_async_await_reactions(",
    ));
    let condition = normalized_code(
        r#"
        function.instruction(&Instruction::LocalGet(reaction_is_rejected_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        "#,
    );

    assert_eq!(close_on_rejection.matches(&condition).count(), 1);
}

#[test]
fn contract_records_scope_census_and_verification() {
    let contract = normalized(CONTRACT);
    assert!(contract.contains("five-valueasync-generatorresume-kinddomain"));
    assert!(contract.contains("ninetypedstoreselections"));
    assert!(contract.contains("fivestrictreaders"));
    assert!(contract.contains("wideneddelegationpending-kindtransport"));
    assert!(contract.contains("relatedstructuraltests"));
    assert!(contract.contains("10/10"));
    assert!(contract.contains("everynon-successbucketatzero"));
}
