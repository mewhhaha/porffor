use std::fs;
use std::path::{Path, PathBuf};

const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

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
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .replace(",)", ")")
        .replace(",]", "]")
}

fn normalized_code(source: &str) -> String {
    let code = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    normalized(&code)
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

fn positions(body: &str, needle: &str) -> Vec<usize> {
    body.match_indices(needle).map(|(index, _)| index).collect()
}

fn collect_rust_sources(dir: &Path, sources: &mut Vec<(PathBuf, String)>) {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
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

fn generator_allocation_owner() -> &'static str {
    let dispatcher = bounded(
        FUNCTIONS_SOURCE,
        "pub(crate) fn emit_function_handle_call_with_argv_inner(",
        "pub(crate) fn emit_prepare_super_construct_to_locals(",
    );
    bounded(
        dispatcher,
        "if can_call_generator {",
        "if can_call_async_generator {",
    )
}

fn generator_resume_owner() -> &'static str {
    bounded(
        STANDARD_SOURCE,
        "fn emit_generator_resume_call(",
        "pub(crate) fn compile_standard_builtin(",
    )
}

fn generator_dispatch_owner() -> &'static str {
    bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::GeneratorPrototypeNext\n            | StandardBuiltinId::GeneratorPrototypeReturn\n            | StandardBuiltinId::GeneratorPrototypeThrow => {",
        "StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose => {",
    )
}

#[test]
fn generator_state_is_one_closed_domain_with_one_stable_projection() {
    let declaration = bounded(
        HEAP_SOURCE,
        "pub(crate) enum GeneratorState {",
        "}\n\nimpl GeneratorState {",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        [
            "SuspendedStart,",
            "Executing,",
            "Completed,",
            "SuspendedYield,",
        ],
        "the synchronous generator state must remain the four ECMA-262 states"
    );

    let policy = bounded(
        HEAP_SOURCE,
        "impl GeneratorState {",
        "/// One strictly validated snapshot of a synchronous generator's state word.",
    );
    let policy = normalized_code(policy);
    let exact_policy = normalized_code(
        r#"
        const ALL: [Self; 4] = [
            Self::SuspendedStart,
            Self::Executing,
            Self::Completed,
            Self::SuspendedYield,
        ];

        const fn word(self) -> u64 {
            match self {
                Self::SuspendedStart => 0,
                Self::Executing => 1,
                Self::Completed => 2,
                Self::SuspendedYield => 3,
            }
        }
    }
        "#,
    );
    assert_eq!(
        policy, exact_policy,
        "GeneratorState must have exactly one closed list and one integer projection, with no decoder"
    );
    assert_eq!(policy.matches("constALL:[Self;4]").count(), 1);
    assert_eq!(
        policy
            .matches("[Self::SuspendedStart,Self::Executing,Self::Completed,Self::SuspendedYield]")
            .count(),
        1,
        "GeneratorState::ALL must enumerate every state in stable word order"
    );
    assert_eq!(policy.matches("constfnword(self)->u64").count(), 1);
    for arm in [
        "Self::SuspendedStart=>0",
        "Self::Executing=>1",
        "Self::Completed=>2",
        "Self::SuspendedYield=>3",
    ] {
        assert_eq!(policy.matches(arm).count(), 1, "missing stable arm {arm}");
    }
    assert_eq!(policy.matches("=>").count(), 4);
    assert!(!policy.contains("_=>"));
    assert!(!policy.contains("unreachable!"));

    let domain = bounded(
        HEAP_SOURCE,
        "/// The closed `[[GeneratorState]]` domain persisted in a synchronous",
        "pub(crate) const GENERATOR_RESUME_STATE_INITIALIZING",
    );
    assert!(!domain.contains("repr("));
    assert!(!HEAP_SOURCE.contains("impl Default for GeneratorState"));
    assert!(!HEAP_SOURCE.contains("impl From<u64> for GeneratorState"));
    assert!(!HEAP_SOURCE.contains("impl From<GeneratorState> for u64"));

    let heap_code = normalized_code(HEAP_SOURCE);
    assert_eq!(
        heap_code.matches("GeneratorState").count(),
        10,
        "heap.rs may name the state domain only at its declaration, policy, and typed boundary"
    );
    assert_eq!(
        heap_code.matches("LoadedGeneratorState").count(),
        5,
        "the opaque token may appear only at its declaration and strict typed boundary"
    );
    assert_eq!(heap_code.matches("implGeneratorState{").count(), 1);
    assert_eq!(heap_code.matches("GeneratorState::ALL").count(), 1);
    assert_eq!(heap_code.matches(":GeneratorState").count(), 2);

    let token = bounded(
        HEAP_SOURCE,
        "/// One strictly validated snapshot of a synchronous generator's state word.",
        "pub(crate) const GENERATOR_RESUME_STATE_INITIALIZING",
    );
    assert_eq!(
        token
            .matches("pub(crate) struct LoadedGeneratorState(u32);")
            .count(),
        1
    );
    assert!(token.contains("#[must_use"));
    assert!(!token.contains("#[derive(Clone"));
    assert!(!token.contains("#[derive(Copy"));
    assert!(!HEAP_SOURCE.contains("impl LoadedGeneratorState"));
    assert_eq!(
        heap_code.matches("LoadedGeneratorState(").count(),
        2,
        "only the tuple-struct declaration and strict loader may spell token construction"
    );
    assert_eq!(
        heap_code.matches("LoadedGeneratorState{").count(),
        1,
        "only the strict loader return type may place an opening body brace after the token name"
    );
}

#[test]
fn generator_state_heap_boundary_is_private_strict_and_opaque() {
    assert_eq!(
        HEAP_SOURCE
            .matches("const HEAP_GENERATOR_STATE_OFFSET: u64 = 80;")
            .count(),
        1
    );
    assert!(!HEAP_SOURCE.contains("pub(crate) const HEAP_GENERATOR_STATE_OFFSET"));
    assert_eq!(
        HEAP_SOURCE.matches("HEAP_GENERATOR_STATE_OFFSET").count(),
        4,
        "only the declaration, layout, typed store, and strict load own the raw offset"
    );
    assert!(!FUNCTIONS_SOURCE.contains("HEAP_GENERATOR_STATE_OFFSET"));
    assert!(!STANDARD_SOURCE.contains("HEAP_GENERATOR_STATE_OFFSET"));

    let store = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_store_generator_state(",
        "/// Load and strictly validate one snapshot of `[[GeneratorState]]`.",
    );
    assert!(store.contains("state: GeneratorState,"));
    assert_eq!(store.matches("HEAP_GENERATOR_STATE_OFFSET").count(), 1);
    assert_eq!(store.matches("state.word()").count(), 1);

    let loader = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_load_generator_state_strict(",
        "/// Emit one comparison against a strictly loaded generator-state word.",
    );
    assert!(loader.contains(") -> LoadedGeneratorState {"));
    assert_eq!(loader.matches("reserve_temp_local()").count(), 1);
    assert_eq!(loader.matches("HEAP_GENERATOR_STATE_OFFSET").count(), 1);
    assert_eq!(
        loader.matches("for state in GeneratorState::ALL").count(),
        1
    );
    assert_eq!(loader.matches("state.word()").count(), 1);
    assert_eq!(loader.matches("Instruction::Unreachable").count(), 1);
    assert_eq!(
        loader
            .matches("LoadedGeneratorState(state_word_local)")
            .count(),
        1
    );

    let strict_dispatch = normalized_code(
        r#"
        let mut open_dispatch_arms = 0;
        for state in GeneratorState::ALL {
            function.instruction(&Instruction::LocalGet(state_word_local));
            function.instruction(&Instruction::I64Const(state.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            open_dispatch_arms += 1;
        }
        function.instruction(&Instruction::Unreachable);
        for _ in 0..open_dispatch_arms {
            function.instruction(&Instruction::End);
        }

        LoadedGeneratorState(state_word_local)
        "#,
    );
    let normalized_loader = normalized_code(loader);
    assert_eq!(
        normalized_loader.matches(strict_dispatch.as_str()).count(),
        1,
        "each valid word must escape its nested else, while the sole trap follows all four misses and precedes every closing End"
    );
    for instruction in [
        "Instruction::If",
        "Instruction::Else",
        "Instruction::Unreachable",
        "Instruction::End",
    ] {
        assert_eq!(
            loader.matches(instruction).count(),
            1,
            "strict decoder must have one source emission site for {instruction}"
        );
    }
    for bypass in [
        "Instruction::Block",
        "Instruction::Loop",
        "Instruction::Br(",
        "Instruction::BrIf",
    ] {
        assert!(
            !loader.contains(bypass),
            "strict decoder must not bypass its closed nested dispatch with {bypass}"
        );
    }

    let comparer = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_generator_state_equals(",
        "/// Release the private local owned by a loaded generator-state snapshot.",
    );
    assert!(comparer.contains("loaded: &LoadedGeneratorState,"));
    assert!(comparer.contains("expected: GeneratorState,"));
    assert_eq!(comparer.matches("LocalGet(loaded.0)").count(), 1);
    assert_eq!(comparer.matches("expected.word()").count(), 1);

    let release = bounded(
        HEAP_SOURCE,
        "pub(crate) fn release_loaded_generator_state(",
        "/// Store one kind from the closed synchronous-generator resume domain.",
    );
    assert!(release.contains("loaded: LoadedGeneratorState"));
    assert!(!release.contains("&LoadedGeneratorState"));
    assert_eq!(release.matches("release_temp_local(loaded.0)").count(), 1);

    let typed_boundary = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_store_generator_state(",
        "/// Store one kind from the closed synchronous-generator resume domain.",
    );
    let typed_boundary = normalized_code(typed_boundary);
    for forbidden in ["stateas", "expectedas", "transmute"] {
        assert!(
            !typed_boundary.contains(forbidden),
            "generator-state projection and validation must not use {forbidden}"
        );
    }
}

#[test]
fn generator_state_product_owners_preserve_the_exact_census_and_order() {
    let allocation = normalized(generator_allocation_owner());
    let resume = normalized(generator_resume_owner());
    let dispatch = normalized(generator_dispatch_owner());

    let start_store =
        "self.emit_store_generator_state(payload_local,GeneratorState::SuspendedStart,function);";
    assert_eq!(allocation.matches(start_store).count(), 1);
    assert_eq!(allocation.matches("emit_store_generator_state(").count(), 1);
    let brand = unique_position(
        &allocation,
        "OBJECT_INTERNAL_BRAND_GENERATOR",
        "generator allocation brand",
    );
    let start = unique_position(&allocation, start_store, "suspended-start initialization");
    let function_field = unique_position(
        &allocation,
        "HEAP_GENERATOR_FUNCTION_OFFSET",
        "generator function field",
    );
    assert!(brand < start && start < function_field);

    let resumed_executing = "self.emit_store_generator_state(generator_payload_local,GeneratorState::Executing,function);";
    let resumed_yield = "self.emit_store_generator_state(generator_payload_local,GeneratorState::SuspendedYield,function);";
    let resumed_completed = "self.emit_store_generator_state(generator_payload_local,GeneratorState::Completed,function);";
    assert_eq!(resume.matches("emit_store_generator_state(").count(), 3);
    assert_eq!(resume.matches("emit_generator_state_equals(").count(), 0);
    let resumed_executing = unique_position(&resume, resumed_executing, "resumed executing store");
    let call = unique_position(&resume, "Instruction::CallIndirect", "resumed body call");
    let resumed_yield = unique_position(&resume, resumed_yield, "resumed yield store");
    let yielded_result = unique_position(
        &resume,
        "self.emit_iterator_result_object_from_locals(self.result_local,self.result_tag_local,false,self.result_local,self.result_tag_local,function)?;",
        "resumed yielded result",
    );
    let resumed_completed = unique_position(&resume, resumed_completed, "resumed completed store");
    let terminal_exit = unique_position(
        &resume,
        "self.emit_return_current_completion_if_throw(function);",
        "resumed terminal exit",
    );
    assert!(resumed_executing < call);
    assert!(call < resumed_yield && resumed_yield < yielded_result);
    assert!(yielded_result < resumed_completed && resumed_completed < terminal_exit);

    assert_eq!(
        dispatch
            .matches("emit_load_generator_state_strict(")
            .count(),
        1
    );
    assert_eq!(dispatch.matches("emit_generator_state_equals(").count(), 4);
    assert_eq!(dispatch.matches("emit_store_generator_state(").count(), 4);
    assert_eq!(
        dispatch.matches("release_loaded_generator_state(").count(),
        1
    );
    assert!(!dispatch.contains("state_local"));

    let load = unique_position(
        &dispatch,
        "self.emit_load_generator_state_strict(this_payload_local,function)",
        "strict generator-state load",
    );
    let brand_check = unique_position(
        &dispatch,
        "OBJECT_INTERNAL_BRAND_GENERATOR",
        "generator brand check",
    );
    let brand_release = unique_position(
        &dispatch,
        "self.release_temp_local(brand_local);",
        "generator brand-local release",
    );
    assert!(brand_check < brand_release && brand_release < load);

    let executing_compare =
        "self.emit_generator_state_equals(&generator_state,GeneratorState::Executing,function);";
    let yield_compare = "self.emit_generator_state_equals(&generator_state,GeneratorState::SuspendedYield,function);";
    let completed_compare =
        "self.emit_generator_state_equals(&generator_state,GeneratorState::Completed,function);";
    let executing_compare = unique_position(&dispatch, executing_compare, "executing comparison");
    let yield_compares = positions(&dispatch, yield_compare);
    assert_eq!(
        yield_compares.len(),
        2,
        "the dominated comparison remains inventoried"
    );
    let completed_compare = unique_position(&dispatch, completed_compare, "completed comparison");
    let resume_call = unique_position(
        &dispatch,
        "self.emit_generator_resume_call(this_payload_local,function)?;",
        "shared suspended-yield resume call",
    );
    assert!(load < executing_compare);
    assert!(executing_compare < yield_compares[0]);
    assert!(yield_compares[0] < resume_call && resume_call < completed_compare);
    assert!(completed_compare < yield_compares[1]);

    let payload_offsets = positions(&dispatch, "HEAP_GENERATOR_RESUME_PAYLOAD_OFFSET");
    let tag_offsets = positions(&dispatch, "HEAP_GENERATOR_RESUME_TAG_OFFSET");
    assert_eq!(payload_offsets.len(), 2);
    assert_eq!(tag_offsets.len(), 2);
    let resume_kind = unique_position(
        &dispatch,
        "self.emit_store_generator_resume_kind(this_payload_local,matchbuiltin{StandardBuiltinId::GeneratorPrototypeNext=>GeneratorResumeKind::Normal,StandardBuiltinId::GeneratorPrototypeReturn=>GeneratorResumeKind::Return,StandardBuiltinId::GeneratorPrototypeThrow=>GeneratorResumeKind::Throw,_=>unreachable!(),},function);",
        "typed resume-kind store",
    );
    assert!(payload_offsets[0] < tag_offsets[0]);
    assert!(tag_offsets[0] < resume_kind && resume_kind < resume_call);

    let inline_executing =
        "self.emit_store_generator_state(this_payload_local,GeneratorState::Executing,function);";
    let inline_yield = "self.emit_store_generator_state(this_payload_local,GeneratorState::SuspendedYield,function);";
    let completed_store =
        "self.emit_store_generator_state(this_payload_local,GeneratorState::Completed,function);";
    let inline_executing = unique_position(&dispatch, inline_executing, "initial executing store");
    let inline_call = unique_position(&dispatch, "Instruction::CallIndirect", "initial body call");
    let inline_yield = unique_position(&dispatch, inline_yield, "initial yield store");
    let yielded_result = unique_position(
        &dispatch,
        "self.emit_iterator_result_object_from_locals(self.result_local,self.result_tag_local,false,self.result_local,self.result_tag_local,function)?;",
        "initial yielded result",
    );
    let completed_stores = positions(&dispatch, completed_store);
    assert_eq!(completed_stores.len(), 2);
    let terminal_throw = unique_position(
        &dispatch,
        "self.emit_return_current_completion_if_throw(function);",
        "initial terminal throw",
    );
    let release = unique_position(
        &dispatch,
        "self.release_loaded_generator_state(generator_state);",
        "loaded-state release",
    );
    let non_resuming_argument = unique_position(
        &dispatch,
        "self.emit_builtin_arg_to_locals(0,value_payload_local,value_tag_local,function);",
        "non-resuming return/throw argument",
    );
    let non_resuming_branch = unique_position(
        &dispatch,
        "ifbuiltin==StandardBuiltinId::GeneratorPrototypeThrow{",
        "non-resuming return/throw branch",
    );
    let non_resuming_throw = unique_position(
        &dispatch,
        "self.set_completion_kind(CompletionKind::Throw,function);",
        "non-resuming throw completion",
    );
    let completed_results = positions(
        &dispatch,
        "self.emit_iterator_result_object_from_locals(value_payload_local,value_tag_local,true,self.result_local,self.result_tag_local,function)?;",
    );
    assert_eq!(
        completed_results.len(),
        2,
        "completed next and non-resuming return each materialize one done result"
    );
    assert!(yield_compares[1] < inline_executing && inline_executing < inline_call);
    assert!(inline_call < inline_yield && inline_yield < yielded_result);
    assert!(yielded_result < completed_stores[0] && completed_stores[0] < terminal_throw);
    assert!(terminal_throw < non_resuming_argument);
    assert!(non_resuming_argument < completed_stores[1]);
    assert!(completed_stores[1] < non_resuming_branch);
    assert!(non_resuming_branch < non_resuming_throw && non_resuming_throw < release);
    assert!(non_resuming_branch < completed_results[1] && completed_results[1] < release);
}

#[test]
fn generator_state_has_no_uninventoried_source_bypass() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);

    let mut stores = 0;
    let mut loads = 0;
    let mut comparisons = 0;
    let mut releases = 0;
    let mut resume_helper_mentions = 0;
    for (path, source) in sources {
        let relative = path
            .strip_prefix(&source_root)
            .expect("collected source must remain below src")
            .to_string_lossy();
        let expected = match relative.as_ref() {
            "heap.rs" => (1, 1, 1, 1, 4, 0),
            "functions.rs" => (1, 0, 0, 0, 0, 0),
            "builtins/standard.rs" => (7, 1, 4, 1, 0, 2),
            _ => (0, 0, 0, 0, 0, 0),
        };
        let actual = (
            source.matches("emit_store_generator_state(").count(),
            source.matches("emit_load_generator_state_strict(").count(),
            source.matches("emit_generator_state_equals(").count(),
            source.matches("release_loaded_generator_state(").count(),
            source.matches("HEAP_GENERATOR_STATE_OFFSET").count(),
            source.matches("emit_generator_resume_call(").count(),
        );
        assert_eq!(
            actual, expected,
            "unexpected synchronous-generator state owner in {relative}"
        );
        stores += actual.0;
        loads += actual.1;
        comparisons += actual.2;
        releases += actual.3;
        resume_helper_mentions += actual.5;

        for line in source.lines() {
            let names_sync_raw_state = line.contains("GENERATOR_STATE_")
                && !line.contains("ASYNC_GENERATOR_STATE_")
                && !line.contains("HEAP_GENERATOR_STATE_OFFSET");
            assert!(
                !names_sync_raw_state,
                "retired raw synchronous-generator state name in {relative}: {line}"
            );
        }
    }

    assert_eq!(stores, 9, "one definition plus eight product stores");
    assert_eq!(loads, 2, "one definition plus the sole product load");
    assert_eq!(
        comparisons, 5,
        "one definition plus four product comparisons"
    );
    assert_eq!(releases, 2, "one definition plus the sole product release");
    assert_eq!(
        resume_helper_mentions, 2,
        "one resume-helper definition plus its sole suspended-yield dispatcher caller"
    );

    assert_eq!(
        FUNCTIONS_SOURCE
            .matches("GeneratorState::SuspendedStart")
            .count(),
        1
    );
    assert_eq!(
        STANDARD_SOURCE.matches("GeneratorState::Executing").count(),
        3
    );
    assert_eq!(
        STANDARD_SOURCE
            .matches("GeneratorState::SuspendedYield")
            .count(),
        4
    );
    assert_eq!(
        STANDARD_SOURCE.matches("GeneratorState::Completed").count(),
        4
    );
}
