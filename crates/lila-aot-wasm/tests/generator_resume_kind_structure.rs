use std::fs;
use std::path::{Path, PathBuf};

const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const DELEGATION_SOURCE: &str = include_str!("../src/generator_delegation.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/generator-resume-kind-word.md");

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

fn positions(body: &str, needle: &str) -> Vec<usize> {
    body.match_indices(needle)
        .map(|(position, _)| position)
        .collect()
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

fn generator_dispatch_owner() -> &'static str {
    bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::GeneratorPrototypeNext\n            | StandardBuiltinId::GeneratorPrototypeReturn\n            | StandardBuiltinId::GeneratorPrototypeThrow => {",
        "StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose => {",
    )
}

fn plain_generator_yield_owner() -> &'static str {
    bounded(
        CONTROL_FLOW_SOURCE,
        "StatementIr::GeneratorYield {\n                value,\n                form,\n                suspend_state,\n                resume_state,\n                resume_mode,\n            } => {",
        "StatementIr::AsyncAwait {",
    )
}

fn generator_delegation_owner() -> &'static str {
    bounded(
        DELEGATION_SOURCE,
        "pub(crate) fn compile_generator_delegation(",
        "fn emit_generator_delegate_property_read(",
    )
}

#[test]
fn resume_kind_is_the_exact_three_value_generator_domain() {
    let declaration = bounded(
        HEAP_SOURCE,
        "pub(crate) enum GeneratorResumeKind {",
        "}\n\nimpl GeneratorResumeKind {",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(variants, ["Normal,", "Return,", "Throw,"]);

    let policy = normalized_code(bounded(
        HEAP_SOURCE,
        "impl GeneratorResumeKind {",
        "/// One strictly validated snapshot of a synchronous generator resume kind.",
    ));
    assert_eq!(
        policy
            .matches("constALL:[Self;3]=[Self::Normal,Self::Return,Self::Throw];")
            .count(),
        1
    );
    assert_eq!(
        policy
            .matches("Self::Normal=>0,Self::Return=>1,Self::Throw=>2,")
            .count(),
        1
    );
    assert_eq!(policy.matches("=>").count(), 3);
    assert!(!policy.contains("_=>"));
    assert!(!policy.contains("unreachable!"));

    let domain = bounded(
        HEAP_SOURCE,
        "/// The closed completion kind supplied when a synchronous generator resumes.",
        "/// The closed Completion Record subset persisted in an async-generator",
    );
    assert!(!domain.contains("repr("));
    assert!(!HEAP_SOURCE.contains("impl Default for GeneratorResumeKind"));
    assert!(!HEAP_SOURCE.contains("impl From<u64> for GeneratorResumeKind"));
    assert!(!HEAP_SOURCE.contains("impl From<i64> for GeneratorResumeKind"));
    assert!(!HEAP_SOURCE.contains("impl From<bool> for GeneratorResumeKind"));
    for retired in [
        "GENERATOR_RESUME_KIND_NORMAL",
        "GENERATOR_RESUME_KIND_RETURN",
        "GENERATOR_RESUME_KIND_THROW",
    ] {
        assert!(!HEAP_SOURCE.contains(retired));
    }

    let loaded = bounded(
        HEAP_SOURCE,
        "/// One strictly validated snapshot of a synchronous generator resume kind.",
        "/// The exact resume-kind transport joining fresh and resumed delegation.",
    );
    assert!(loaded.contains("#[must_use"));
    assert!(loaded.contains("pub(crate) struct LoadedGeneratorResumeKind(u32);"));
    assert!(!loaded.contains("#[derive(Clone"));
    assert!(!loaded.contains("#[derive(Copy"));

    let transport = bounded(
        HEAP_SOURCE,
        "/// The exact resume-kind transport joining fresh and resumed delegation.",
        "/// The closed Completion Record subset persisted in an async-generator",
    );
    assert!(transport.contains("#[must_use"));
    assert!(transport.contains("pub(crate) struct GeneratorResumeKindTransport(u32);"));
    assert!(!transport.contains("#[derive(Clone"));
    assert!(!transport.contains("#[derive(Copy"));
    assert_eq!(HEAP_SOURCE.matches("LoadedGeneratorResumeKind(").count(), 2);
    assert_eq!(
        HEAP_SOURCE.matches("GeneratorResumeKindTransport(").count(),
        2
    );
}

#[test]
fn resume_kind_heap_boundary_is_private_strict_and_opaque() {
    assert_eq!(
        HEAP_SOURCE
            .matches("const HEAP_GENERATOR_RESUME_KIND_OFFSET: u64 = 168;")
            .count(),
        1
    );
    assert!(!HEAP_SOURCE.contains("pub(crate) const HEAP_GENERATOR_RESUME_KIND_OFFSET"));
    assert_eq!(
        HEAP_SOURCE
            .matches("HEAP_GENERATOR_RESUME_KIND_OFFSET")
            .count(),
        4,
        "only declaration, layout, typed store and strict load may name the offset"
    );
    for source in [
        FUNCTIONS_SOURCE,
        STANDARD_SOURCE,
        CONTROL_FLOW_SOURCE,
        DELEGATION_SOURCE,
    ] {
        assert!(!source.contains("HEAP_GENERATOR_RESUME_KIND_OFFSET"));
    }

    let store = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_store_generator_resume_kind(",
        "/// Load and strictly validate one generator resume-kind snapshot.",
    );
    assert!(store.contains("kind: GeneratorResumeKind,"));
    assert_eq!(
        store.matches("HEAP_GENERATOR_RESUME_KIND_OFFSET").count(),
        1
    );
    assert_eq!(store.matches("kind.word()").count(), 1);

    let loader = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_load_generator_resume_kind_strict(",
        "/// Compare one strictly loaded generator resume kind.",
    );
    assert!(loader.contains(") -> LoadedGeneratorResumeKind {"));
    assert_eq!(loader.matches("reserve_temp_local()").count(), 1);
    assert_eq!(
        loader.matches("HEAP_GENERATOR_RESUME_KIND_OFFSET").count(),
        1
    );
    assert_eq!(
        loader
            .matches("for kind in GeneratorResumeKind::ALL")
            .count(),
        1
    );
    assert_eq!(loader.matches("kind.word()").count(), 1);
    assert_eq!(loader.matches("Instruction::Else").count(), 1);
    assert_eq!(loader.matches("Instruction::Unreachable").count(), 1);
    assert_eq!(
        loader
            .matches("LoadedGeneratorResumeKind(kind_word_local)")
            .count(),
        1
    );
    assert!(!loader.contains("_ =>"));

    let comparison = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_generator_resume_kind_equals(",
        "/// Initialize the exact resume-kind transport for fresh delegation.",
    );
    assert!(comparison.contains("loaded: &LoadedGeneratorResumeKind,"));
    assert!(comparison.contains("expected: GeneratorResumeKind,"));
    assert_eq!(comparison.matches("LocalGet(loaded.0)").count(), 1);
    assert_eq!(comparison.matches("expected.word()").count(), 1);

    let release = bounded(
        HEAP_SOURCE,
        "pub(crate) fn release_loaded_generator_resume_kind(",
        "/// Release the private local owned by a delegation resume-kind transport.",
    );
    assert!(release.contains("loaded: LoadedGeneratorResumeKind,"));
    assert!(!release.contains("&LoadedGeneratorResumeKind"));
    assert_eq!(release.matches("release_temp_local(loaded.0)").count(), 1);
}

#[test]
fn delegation_transport_accepts_only_typed_or_validated_resume_kinds() {
    let initialization = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_initialize_generator_resume_kind_transport(",
        "/// Copy a validated activation snapshot into the delegation transport.",
    );
    assert!(initialization.contains("kind: GeneratorResumeKind,"));
    assert!(initialization.contains(") -> GeneratorResumeKindTransport {"));
    assert_eq!(initialization.matches("kind.word()").count(), 1);
    assert_eq!(initialization.matches("reserve_temp_local()").count(), 1);
    assert!(!initialization.contains("HEAP_GENERATOR_RESUME_KIND_OFFSET"));

    let copy = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_copy_generator_resume_kind_to_transport(",
        "/// Compare one exact generator delegation resume-kind transport.",
    );
    assert!(copy.contains("loaded: &LoadedGeneratorResumeKind,"));
    assert!(copy.contains("transport: &GeneratorResumeKindTransport,"));
    assert_eq!(copy.matches("LocalGet(loaded.0)").count(), 1);
    assert_eq!(copy.matches("LocalSet(transport.0)").count(), 1);
    assert!(!copy.contains("HEAP_GENERATOR_RESUME_KIND_OFFSET"));

    let comparison = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_generator_resume_kind_transport_equals(",
        "/// Release the private local owned by a resume-kind snapshot.",
    );
    assert!(comparison.contains("transport: &GeneratorResumeKindTransport,"));
    assert!(comparison.contains("expected: GeneratorResumeKind,"));
    assert_eq!(comparison.matches("LocalGet(transport.0)").count(), 1);
    assert_eq!(comparison.matches("expected.word()").count(), 1);

    let release = bounded(
        HEAP_SOURCE,
        "pub(crate) fn release_generator_resume_kind_transport(",
        "/// Store one completion kind from the closed async-generator request",
    );
    assert!(release.contains("transport: GeneratorResumeKindTransport,"));
    assert!(!release.contains("&GeneratorResumeKindTransport"));
    assert_eq!(
        release.matches("release_temp_local(transport.0)").count(),
        1
    );
}

#[test]
fn resume_kind_has_two_typed_writers_and_two_strict_readers() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);

    let mut totals = [0; 9];
    for (path, source) in sources {
        let relative = path
            .strip_prefix(&source_root)
            .expect("collected source must remain below src")
            .to_string_lossy();
        let expected = match relative.as_ref() {
            "heap.rs" => (4, 1, 1, 1, 1, 1, 1, 1, 1),
            "functions.rs" => (0, 1, 0, 0, 0, 0, 0, 0, 0),
            "builtins/standard.rs" => (0, 1, 0, 0, 0, 0, 0, 0, 0),
            "control_flow.rs" => (0, 0, 1, 2, 1, 0, 0, 0, 0),
            "generator_delegation.rs" => (0, 0, 1, 0, 1, 1, 1, 3, 1),
            _ => (0, 0, 0, 0, 0, 0, 0, 0, 0),
        };
        let actual = (
            source.matches("HEAP_GENERATOR_RESUME_KIND_OFFSET").count(),
            source.matches("emit_store_generator_resume_kind(").count(),
            source
                .matches("emit_load_generator_resume_kind_strict(")
                .count(),
            source.matches("emit_generator_resume_kind_equals(").count(),
            source
                .matches("release_loaded_generator_resume_kind(")
                .count(),
            source
                .matches("emit_initialize_generator_resume_kind_transport(")
                .count(),
            source
                .matches("emit_copy_generator_resume_kind_to_transport(")
                .count(),
            source
                .matches("emit_generator_resume_kind_transport_equals(")
                .count(),
            source
                .matches("release_generator_resume_kind_transport(")
                .count(),
        );
        assert_eq!(
            actual, expected,
            "unexpected resume-kind owner in {relative}"
        );
        totals[0] += actual.0;
        totals[1] += actual.1;
        totals[2] += actual.2;
        totals[3] += actual.3;
        totals[4] += actual.4;
        totals[5] += actual.5;
        totals[6] += actual.6;
        totals[7] += actual.7;
        totals[8] += actual.8;
    }
    assert_eq!(totals, [4, 3, 3, 3, 3, 2, 2, 4, 2]);
}

#[test]
fn writers_select_typed_kinds_before_generator_resumption() {
    let allocation = normalized_code(generator_allocation_owner());
    let resume_tag = unique_position(
        &allocation,
        "HEAP_GENERATOR_RESUME_TAG_OFFSET",
        "allocation resume tag",
    );
    let kind = unique_position(
        &allocation,
        "self.emit_store_generator_resume_kind(payload_local,GeneratorResumeKind::Normal,function);",
        "allocation typed Normal resume kind",
    );
    let pending_head = unique_position(
        &allocation,
        "HEAP_GENERATOR_PENDING_COMPLETION_HEAD_OFFSET",
        "allocation pending-completion head",
    );
    assert!(resume_tag < kind && kind < pending_head);

    let dispatch = normalized_code(generator_dispatch_owner());
    assert_eq!(
        dispatch
            .matches("emit_store_generator_resume_kind(")
            .count(),
        1
    );
    for kind in ["Normal", "Return", "Throw"] {
        let selection = format!("GeneratorResumeKind::{kind}");
        assert_eq!(dispatch.matches(selection.as_str()).count(), 1);
    }
    let payloads = positions(&dispatch, "HEAP_GENERATOR_RESUME_PAYLOAD_OFFSET");
    let tags = positions(&dispatch, "HEAP_GENERATOR_RESUME_TAG_OFFSET");
    assert_eq!(payloads.len(), 2);
    assert_eq!(tags.len(), 2);
    let kind = unique_position(
        &dispatch,
        "self.emit_store_generator_resume_kind(this_payload_local,matchbuiltin{StandardBuiltinId::GeneratorPrototypeNext=>GeneratorResumeKind::Normal,StandardBuiltinId::GeneratorPrototypeReturn=>GeneratorResumeKind::Return,StandardBuiltinId::GeneratorPrototypeThrow=>GeneratorResumeKind::Throw,_=>unreachable!(),},function);",
        "prototype typed resume-kind selection",
    );
    let resume = unique_position(
        &dispatch,
        "self.emit_generator_resume_call(this_payload_local,function)?;",
        "suspended-yield resume call",
    );
    assert!(payloads[0] < tags[0] && tags[0] < kind && kind < resume);
}

#[test]
fn readers_validate_one_snapshot_and_preserve_fresh_delegation_normal() {
    let plain_yield = normalized_code(plain_generator_yield_owner());
    let load = unique_position(
        &plain_yield,
        "self.emit_load_generator_resume_kind_strict(activation_local,function)",
        "plain-yield strict load",
    );
    let returned = unique_position(
        &plain_yield,
        "self.emit_generator_resume_kind_equals(&resume_kind,GeneratorResumeKind::Return,function);",
        "plain-yield Return route",
    );
    let thrown = unique_position(
        &plain_yield,
        "self.emit_generator_resume_kind_equals(&resume_kind,GeneratorResumeKind::Throw,function);",
        "plain-yield Throw route",
    );
    let release = unique_position(
        &plain_yield,
        "self.release_loaded_generator_resume_kind(resume_kind);",
        "plain-yield token release",
    );
    let dispatch = unique_position(
        &plain_yield,
        "self.emit_dispatch_current_completion(function)?;",
        "plain-yield completion dispatch",
    );
    assert!(load < returned && returned < thrown && thrown < release && release < dispatch);

    let delegation = normalized_code(generator_delegation_owner());
    assert!(!delegation.contains("resume_kind_local"));
    assert!(!delegation.contains("HEAP_GENERATOR_RESUME_KIND_OFFSET"));
    assert!(!delegation.contains("emit_generator_resume_kind_equals("));
    assert_eq!(
        delegation
            .matches("emit_generator_resume_kind_transport_equals(")
            .count(),
        3
    );

    let initialization = unique_position(
        &delegation,
        "self.emit_initialize_generator_resume_kind_transport(GeneratorResumeKind::Normal,function)",
        "fresh delegation Normal initialization",
    );
    let load = unique_position(
        &delegation,
        "self.emit_load_generator_resume_kind_strict(activation_local,function)",
        "resumed delegation strict load",
    );
    let copy = unique_position(
        &delegation,
        "self.emit_copy_generator_resume_kind_to_transport(&loaded_resume_kind,&resume_kind,function);",
        "validated delegation transport copy",
    );
    let loaded_release = unique_position(
        &delegation,
        "self.release_loaded_generator_resume_kind(loaded_resume_kind);",
        "delegation activation-token release",
    );
    let first_route = unique_position(
        &delegation,
        "self.emit_generator_resume_kind_transport_equals(&resume_kind,GeneratorResumeKind::Throw,function);",
        "post-join delegation Throw route",
    );
    let transport_release = unique_position(
        &delegation,
        "self.release_generator_resume_kind_transport(resume_kind);",
        "delegation transport release",
    );
    let branch_split = &delegation[initialization..load];
    assert_eq!(branch_split.matches("Instruction::Else").count(), 1);
    let branch_join = &delegation[loaded_release..first_route];
    assert_eq!(branch_join.matches("Instruction::End").count(), 1);
    assert!(initialization < load);
    assert!(load < copy && copy < loaded_release && loaded_release < first_route);
    assert!(first_route < transport_release);
}

#[test]
fn contract_records_scope_census_and_pending_verification() {
    let contract = normalized(CONTRACT);
    assert!(contract.contains("three-valuesynchronousgeneratorresume-kinddomain"));
    assert!(contract.contains("twosemanticwriters"));
    assert!(contract.contains("twostrictreaders"));
    assert!(contract.contains("freshdelegationpath"));
    assert!(contract.contains("Cargoverificationpending"));
}
