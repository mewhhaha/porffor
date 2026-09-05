use std::fs;
use std::path::Path;

const DELEGATION_SOURCE: &str = include_str!("../src/generator_delegation.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/async-generator-delegation-kind.md");

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
fn delegation_kind_is_the_exact_non_capability_two_row_domain() {
    let declaration = normalized(bounded(
        DELEGATION_SOURCE,
        "use super::*;\n\n",
        "enum GeneratorDelegateProperty {",
    ));
    assert_eq!(
        declaration, "pub(crate)enumAsyncGeneratorDelegationKind{YieldStar,ForAwaitYield,}",
        "the delegation kind must have exactly the two established rows"
    );
    assert!(DELEGATION_SOURCE
        .starts_with("use super::*;\n\npub(crate) enum AsyncGeneratorDelegationKind {"));
    for forbidden in [
        "impl Clone for AsyncGeneratorDelegationKind",
        "impl Copy for AsyncGeneratorDelegationKind",
        "impl Default for AsyncGeneratorDelegationKind",
        "impl PartialEq for AsyncGeneratorDelegationKind",
        "impl Eq for AsyncGeneratorDelegationKind",
        "for AsyncGeneratorDelegationKind",
    ] {
        assert!(
            !DELEGATION_SOURCE.contains(forbidden),
            "found `{forbidden}`"
        );
    }
}

#[test]
fn control_flow_has_exactly_one_producer_for_each_delegation_kind() {
    let control_flow = normalized(CONTROL_FLOW_SOURCE);
    assert!(control_flow.contains(
        "YieldForm::Delegate(_)=>{returnself.compile_async_generator_delegation(value,suspend_state,resume_state,resume_mode,AsyncGeneratorDelegationKind::YieldStar,function,);}"));
    assert!(control_flow.contains(
        "async_generator_for_await_is_transparent_yield(&binding.name,body){self.compile_async_generator_delegation(iterable,async_plan.entry_state,async_plan.exit_state,&GeneratorResumeModeIr::Ignore,AsyncGeneratorDelegationKind::ForAwaitYield,function,)?;returnOk(());}"));
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("AsyncGeneratorDelegationKind::YieldStar")
            .count(),
        1
    );
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("AsyncGeneratorDelegationKind::ForAwaitYield")
            .count(),
        1
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "AsyncGeneratorDelegationKind"),
        21,
        "the declaration, import, parameter, two producers and sixteen exhaustive arms must own every delegation-kind mention"
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "compile_async_generator_delegation("),
        3,
        "the definition and two control-flow producers must be the full call census"
    );
    for variant in ["YieldStar", "ForAwaitYield"] {
        assert_eq!(
            count_in_rust_sources(
                &source_root,
                &format!("AsyncGeneratorDelegationKind::{variant}")
            ),
            9,
            "one producer and eight exhaustive arms must name `{variant}`"
        );
    }
}

#[test]
fn delegation_emitter_has_eight_exhaustive_semantic_projections_in_order() {
    let emitter = bounded(
        DELEGATION_SOURCE,
        "    pub(crate) fn compile_async_generator_delegation(",
        "    fn emit_generator_delegate_property_read(",
    );
    assert!(emitter.contains("delegation_kind: AsyncGeneratorDelegationKind,"));
    assert_eq!(emitter.matches("match &delegation_kind {").count(), 8);
    assert!(!emitter.contains("match delegation_kind {"));
    assert_eq!(
        emitter
            .matches("AsyncGeneratorDelegationKind::YieldStar =>")
            .count(),
        8
    );
    assert_eq!(
        emitter
            .matches("AsyncGeneratorDelegationKind::ForAwaitYield =>")
            .count(),
        8
    );
    for forbidden in [
        "delegation_kind ==",
        "delegation_kind !=",
        "matches!(delegation_kind",
        "_ =>",
    ] {
        assert!(!emitter.contains(forbidden), "found `{forbidden}`");
    }

    let emitter = normalized(emitter);
    let starts = emitter
        .match_indices("match&delegation_kind{")
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    assert_eq!(starts.len(), 8);
    let blocks = starts
        .iter()
        .enumerate()
        .map(|(index, start)| {
            let end = starts.get(index + 1).copied().unwrap_or(emitter.len());
            &emitter[*start..end]
        })
        .collect::<Vec<_>>();

    let first_projection = blocks[0]
        .split_once("function.instruction(&Instruction::LocalGet(awaiting_sync_value_local));")
        .expect("the first projection must precede the awaiting-sync-value branch")
        .0;
    assert_eq!(
        first_projection,
        concat!(
            "match&delegation_kind{",
            "AsyncGeneratorDelegationKind::YieldStar=>{}",
            "AsyncGeneratorDelegationKind::ForAwaitYield=>{",
            "self.emit_async_generator_delegate_pending_kind_equals_resume_kind(",
            "pending_kind_local,AsyncGeneratorResumeKind::Throw,function,);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "self.load_i64_to_local_from_offset(record_local,",
            "HEAP_GENERATOR_DELEGATE_PENDING_PAYLOAD_OFFSET,self.result_local,function,);",
            "self.load_i64_to_local_from_offset(record_local,",
            "HEAP_GENERATOR_DELEGATE_PENDING_TAG_OFFSET,self.result_tag_local,function,);",
            "self.store_i64_const_at_offset(activation_local,",
            "HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET,0,function,);",
            "self.set_completion_kind(CompletionKind::Throw,function);",
            "self.emit_dispatch_async_generator_completion(function);",
            "function.instruction(&Instruction::End);}}"
        )
    );

    let second_projection = blocks[1]
        .split_once(
            "self.emit_generator_delegate_property_read(argument_payload_local,argument_tag_local,",
        )
        .expect("the second projection must precede the delegate done read")
        .0;
    assert_eq!(
        second_projection,
        concat!(
            "match&delegation_kind{",
            "AsyncGeneratorDelegationKind::YieldStar=>{}",
            "AsyncGeneratorDelegationKind::ForAwaitYield=>{",
            "self.emit_async_generator_delegate_pending_kind_equals_resume_kind(",
            "pending_kind_local,AsyncGeneratorResumeKind::Return,function,);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "self.load_i64_to_local_from_offset(record_local,",
            "HEAP_GENERATOR_DELEGATE_PENDING_PAYLOAD_OFFSET,self.result_local,function,);",
            "self.load_i64_to_local_from_offset(record_local,",
            "HEAP_GENERATOR_DELEGATE_PENDING_TAG_OFFSET,self.result_tag_local,function,);",
            "self.store_i64_const_at_offset(activation_local,",
            "HEAP_ASYNC_GENERATOR_DELEGATE_RECORD_OFFSET,0,function,);",
            "self.set_completion_kind(CompletionKind::Return,function);",
            "self.emit_dispatch_async_generator_completion(function);",
            "function.instruction(&Instruction::End);}}"
        )
    );

    let third_projection = blocks[2]
        .split_once("function.instruction(&Instruction::LocalGet(argument_payload_local));")
        .expect("the third projection must precede rejected delegate-value forwarding")
        .0;
    assert_eq!(
        third_projection,
        concat!(
            "match&delegation_kind{",
            "AsyncGeneratorDelegationKind::YieldStar=>{}",
            "AsyncGeneratorDelegationKind::ForAwaitYield=>{",
            "self.load_i64_to_local_from_offset(record_local,",
            "HEAP_GENERATOR_DELEGATE_PENDING_KIND_OFFSET,pending_kind_local,function,);",
            "self.emit_async_generator_delegate_pending_kind_equals_resume_kind(",
            "pending_kind_local,AsyncGeneratorResumeKind::Throw,function,);",
            "function.instruction(&Instruction::If(BlockType::Empty));",
            "self.load_i64_to_local_from_offset(record_local,",
            "HEAP_GENERATOR_DELEGATE_PENDING_PAYLOAD_OFFSET,self.result_local,function,);",
            "self.load_i64_to_local_from_offset(record_local,",
            "HEAP_GENERATOR_DELEGATE_PENDING_TAG_OFFSET,self.result_tag_local,function,);",
            "self.set_completion_kind(CompletionKind::Throw,function);",
            "self.emit_dispatch_async_generator_completion(function);",
            "function.instruction(&Instruction::End);}}"
        )
    );

    assert!(blocks[3].starts_with(
        "match&delegation_kind{AsyncGeneratorDelegationKind::YieldStar=>{self.emit_async_generator_delegate_pending_kind_equals_resume_kind(next_pending_kind_local,AsyncGeneratorResumeKind::Throw,function,);}AsyncGeneratorDelegationKind::ForAwaitYield=>{function.instruction(&Instruction::I32Const(0));}}"
    ));

    assert!(blocks[4].starts_with(
        "match&delegation_kind{AsyncGeneratorDelegationKind::YieldStar=>{}AsyncGeneratorDelegationKind::ForAwaitYield=>{self.emit_async_generator_delegate_pending_kind_equals_resume_kind(next_pending_kind_local,AsyncGeneratorResumeKind::Throw,function,);function.instruction(&Instruction::I32Or);}}"
    ));

    assert!(blocks[5].starts_with(
        "match&delegation_kind{AsyncGeneratorDelegationKind::YieldStar=>{self.set_completion_kind(CompletionKind::Return,function);}AsyncGeneratorDelegationKind::ForAwaitYield=>{self.emit_async_generator_delegate_pending_kind_equals_resume_kind(next_pending_kind_local,AsyncGeneratorResumeKind::Throw,function,);function.instruction(&Instruction::If(BlockType::Empty));self.set_completion_kind(CompletionKind::Throw,function);function.instruction(&Instruction::Else);self.set_completion_kind(CompletionKind::Return,function);function.instruction(&Instruction::End);}}"
    ));

    assert!(blocks[6].starts_with(
        "match&delegation_kind{AsyncGeneratorDelegationKind::YieldStar=>{&[(argument_payload_local,argument_tag_local)][..]}AsyncGeneratorDelegationKind::ForAwaitYield=>&[][..],};"
    ));

    assert!(blocks[7].starts_with(
        "match&delegation_kind{AsyncGeneratorDelegationKind::YieldStar=>{}AsyncGeneratorDelegationKind::ForAwaitYield=>{self.emit_undefined_payload(function);function.instruction(&Instruction::LocalSet(argument_payload_local));function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag()asi64));function.instruction(&Instruction::LocalSet(argument_tag_local));}}self.emit_generator_delegate_call(next_payload_local,next_tag_local,iterator_payload_local,iterator_tag_local,&[(argument_payload_local,argument_tag_local)]"
    ));
}

#[test]
fn contract_records_the_invariant_and_behavior_preserving_scope() {
    let contract = normalized(CONTRACT);
    for claim in [
        "exactlytwoproducers",
        "eightexhaustivematches",
        "derivesnocloningorcopyingcapability",
        "emittedWasmisexpectedtoremainbyte-identical",
        "Nogeneratororiteratorbehaviorchangeisclaimed",
    ] {
        assert!(contract.contains(claim), "missing contract claim `{claim}`");
    }
    assert!(CONTRACT
        .contains("cargo test -p lila-aot-wasm --test async_generator_delegation_kind_structure"));
}
