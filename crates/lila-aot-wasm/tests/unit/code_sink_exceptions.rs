use super::*;
use std::borrow::Cow;
use std::panic::{catch_unwind, AssertUnwindSafe};
use wasm_encoder::{
    BlockType, CodeSection, ExportKind, ExportSection, FunctionSection, Module, RefType,
    TagKind, TagSection, TagType, TypeSection,
};

#[path = "../fixtures/exception_control.rs"]
mod fixtures;

fn empty_body() -> Function {
    Function::new_with_locals_types(std::iter::empty())
}

fn table(ty: BlockType, catches: Vec<Catch>) -> Instruction<'static> {
    Instruction::TryTable(ty, Cow::Owned(catches))
}

fn rejected_without_mutation(function: &mut Function, instruction: Instruction<'_>) {
    let before = function.clone();
    let result = catch_unwind(AssertUnwindSafe(|| {
        function.instruction(&instruction);
    }));
    assert!(result.is_err(), "accepted invalid instruction: {instruction:?}");
    assert_eq!(*function, before, "rejection changed the body or live frames");
}

#[test]
fn every_table_catch_checks_the_enclosing_stack_before_opening_a_frame() {
    for label in [1, u32::MAX] {
        for catch in [
            Catch::One { tag: 0, label },
            Catch::OneRef { tag: 0, label },
            Catch::All { label },
            Catch::AllRef { label },
        ] {
            rejected_without_mutation(&mut empty_body(), table(BlockType::Empty, vec![catch]));
        }
    }
    for catch in [
        Catch::One { tag: 0, label: 0 },
        Catch::OneRef { tag: 0, label: 0 },
        Catch::All { label: 0 },
        Catch::AllRef { label: 0 },
    ] {
        let mut function = empty_body();
        function.instruction(&table(BlockType::Empty, vec![catch]));
        assert_eq!(function.depth(), 2);
    }
}

#[test]
fn a_late_invalid_clause_does_not_partially_publish_the_table() {
    rejected_without_mutation(
        &mut empty_body(),
        table(
            BlockType::Empty,
            vec![Catch::All { label: 0 }, Catch::One { tag: 0, label: 1 }],
        ),
    );
}

#[test]
fn a_table_body_counts_its_own_label_for_ordinary_branches() {
    let mut function = empty_body();
    let root = function.label_depth();
    function.instruction(&table(BlockType::Empty, vec![Catch::All { label: 0 }]));
    let table_label = function.label_depth();
    function.instruction(&Instruction::Block(BlockType::Empty));
    assert_eq!(function.branch_depth_to(table_label), BranchDepth(1));
    assert_eq!(function.branch_depth_to(root), BranchDepth(2));
    function.branch_to_label(table_label);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
    let _ = function.into_body();
}

#[test]
fn nested_tables_resolve_handlers_against_their_immediate_enclosing_stack() {
    let mut function = empty_body();
    function.instruction(&table(BlockType::Empty, vec![]));
    rejected_without_mutation(
        &mut function,
        table(BlockType::Empty, vec![Catch::All { label: 2 }]),
    );
    function.instruction(&table(BlockType::Empty, vec![Catch::All { label: 1 }]));
    assert_eq!(function.depth(), 3);
}

#[test]
fn a_closed_table_handle_cannot_target_a_sibling() {
    let mut function = empty_body();
    function.instruction(&table(BlockType::Empty, vec![]));
    let stale = function.label_depth();
    function.instruction(&Instruction::End);
    function.instruction(&table(BlockType::Empty, vec![]));
    assert!(catch_unwind(|| function.branch_depth_to(stale)).is_err());
}

#[test]
fn tagged_handlers_and_catch_all_retain_the_try_label() {
    let mut function = empty_body();
    function.instruction(&Instruction::Try(BlockType::Empty));
    let label = function.label_depth();
    for handler in [Instruction::Catch(0), Instruction::Catch(1), Instruction::CatchAll] {
        function.instruction(&handler);
        assert_eq!(function.label_depth(), label);
        assert_eq!(function.depth(), 2);
        function.instruction(&Instruction::Rethrow(0));
    }
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
    let _ = function.into_body();
}

#[test]
fn handler_transitions_cannot_cross_non_try_frames() {
    let openers = [
        Instruction::Block(BlockType::Empty),
        Instruction::Loop(BlockType::Empty),
        Instruction::If(BlockType::Empty),
        table(BlockType::Empty, vec![]),
    ];
    for opener in openers {
        let mut function = empty_body();
        function.instruction(&Instruction::Try(BlockType::Empty));
        function.instruction(&opener);
        rejected_without_mutation(&mut function, Instruction::Catch(0));
        rejected_without_mutation(&mut function, Instruction::CatchAll);
        rejected_without_mutation(&mut function, Instruction::Delegate(0));
    }
    rejected_without_mutation(&mut empty_body(), Instruction::Catch(0));
    rejected_without_mutation(&mut empty_body(), Instruction::CatchAll);
    rejected_without_mutation(&mut empty_body(), Instruction::Delegate(0));
}

#[test]
fn catch_all_is_the_last_legacy_handler() {
    let mut function = empty_body();
    function.instruction(&Instruction::Try(BlockType::Empty));
    function.instruction(&Instruction::CatchAll);
    rejected_without_mutation(&mut function, Instruction::Catch(0));
    rejected_without_mutation(&mut function, Instruction::CatchAll);
    rejected_without_mutation(&mut function, Instruction::Delegate(0));
}

#[test]
fn delegate_is_not_legal_after_a_tagged_handler() {
    let mut function = empty_body();
    function.instruction(&Instruction::Try(BlockType::Empty));
    function.instruction(&Instruction::Catch(0));
    rejected_without_mutation(&mut function, Instruction::Delegate(0));
}

#[test]
fn delegate_closes_exactly_its_try_and_can_target_the_function() {
    let mut function = empty_body();
    let root = function.label_depth();
    function.instruction(&Instruction::Try(BlockType::Empty));
    let delegated = function.label_depth();
    rejected_without_mutation(&mut function, Instruction::Delegate(1));
    rejected_without_mutation(&mut function, Instruction::Delegate(u32::MAX));
    function.instruction(&Instruction::Delegate(0));
    assert_eq!(function.label_depth(), root);
    assert_eq!(function.depth(), 1);
    assert!(catch_unwind(|| function.branch_depth_to(delegated)).is_err());
    function.instruction(&Instruction::End);
    let _ = function.into_body();
}

#[test]
fn delegate_counts_ordinary_enclosing_frames() {
    let mut function = empty_body();
    function.instruction(&Instruction::Block(BlockType::Empty));
    let outer = function.label_depth();
    for label in [0, 1] {
        function.instruction(&Instruction::Try(BlockType::Empty));
        function.instruction(&Instruction::Delegate(label));
        assert_eq!(function.label_depth(), outer);
    }
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::End);
    let _ = function.into_body();
}

#[test]
fn rethrow_requires_a_catch_at_the_exact_relative_depth() {
    let mut function = empty_body();
    rejected_without_mutation(&mut function, Instruction::Rethrow(0));
    function.instruction(&Instruction::Try(BlockType::Empty));
    rejected_without_mutation(&mut function, Instruction::Rethrow(0));
    function.instruction(&Instruction::Catch(0));
    function.instruction(&Instruction::Block(BlockType::Empty));
    rejected_without_mutation(&mut function, Instruction::Rethrow(0));
    rejected_without_mutation(&mut function, Instruction::Rethrow(2));
    rejected_without_mutation(&mut function, Instruction::Rethrow(3));
    rejected_without_mutation(&mut function, Instruction::Rethrow(u32::MAX));
    function.instruction(&Instruction::Rethrow(1));
    function.instruction(&Instruction::Try(BlockType::Empty));
    rejected_without_mutation(&mut function, Instruction::Rethrow(0));
    function.instruction(&Instruction::Rethrow(2));
    function.instruction(&Instruction::CatchAll);
    function.instruction(&Instruction::Rethrow(0));
    function.instruction(&Instruction::Rethrow(2));
}

#[test]
fn modern_table_frames_are_not_legacy_catch_handlers() {
    let mut function = empty_body();
    function.instruction(&table(BlockType::Empty, vec![Catch::All { label: 0 }]));
    rejected_without_mutation(&mut function, Instruction::Rethrow(0));
    rejected_without_mutation(&mut function, Instruction::Else);
}

#[test]
fn throwing_instructions_do_not_close_structural_frames() {
    let mut function = empty_body();
    function.instruction(&table(BlockType::Empty, vec![]));
    let label = function.label_depth();
    function.instruction(&Instruction::Throw(0));
    function.instruction(&Instruction::ThrowRef);
    assert_eq!(function.label_depth(), label);
    assert_eq!(function.depth(), 2);
}

#[test]
fn exception_instructions_cannot_reopen_a_finished_body() {
    let mut function = empty_body();
    function.instruction(&Instruction::End);
    for instruction in [
        table(BlockType::Empty, vec![]),
        Instruction::Try(BlockType::Empty),
        Instruction::Catch(0),
        Instruction::CatchAll,
        Instruction::Delegate(0),
        Instruction::Rethrow(0),
        Instruction::Throw(0),
        Instruction::ThrowRef,
    ] {
        rejected_without_mutation(&mut function, instruction);
    }
}

#[test]
fn cloning_and_rewriting_locals_preserve_handler_state_and_identity() {
    let mut function = Function::new_with_locals_types([ValType::I64; 4]);
    function.instruction(&Instruction::Try(BlockType::Empty));
    function.instruction(&Instruction::CatchAll);
    let handler = function.label_depth();
    let mut cloned = function.clone().rewrite_local_declaration(4, 2);
    assert_eq!(cloned.label_depth(), handler);
    cloned.instruction(&Instruction::Rethrow(0));
    rejected_without_mutation(&mut cloned, Instruction::Catch(0));
    rejected_without_mutation(&mut cloned, Instruction::Delegate(0));
    cloned.instruction(&Instruction::End);
    cloned.instruction(&Instruction::End);
    let _ = cloned.into_body();
}

fn encoded_module(instructions: &[Instruction<'_>], legacy: bool) -> Vec<u8> {
    let mut function = empty_body();
    let mut encoder = wasm_encoder::Function::new([]);
    for instruction in instructions {
        function.instruction(instruction);
        encoder.instruction(instruction);
    }
    let body = function.into_body();
    assert_eq!(body, encoder, "the checked sink must not change encoder bytes");

    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I32]);
    types.ty().function([ValType::I32], []);
    types.ty().function([], [ValType::I32, ValType::Ref(RefType::EXNREF)]);
    let mut functions = FunctionSection::new();
    functions.function(0);
    let mut tags = TagSection::new();
    for _ in 0..2 {
        tags.tag(TagType { kind: TagKind::Exception, func_type_idx: 1 });
    }
    let mut exports = ExportSection::new();
    exports.export("run", ExportKind::Func, 0);
    let mut code = CodeSection::new();
    code.function(&body);
    let mut module = Module::new();
    module.section(&types).section(&functions).section(&tags).section(&exports).section(&code);
    let bytes = module.finish();
    let mut features = wasmparser::WasmFeatures::default();
    features.set(wasmparser::WasmFeatures::EXCEPTIONS, true);
    features.set(wasmparser::WasmFeatures::LEGACY_EXCEPTIONS, legacy);
    wasmparser::Validator::new_with_features(features)
        .validate_all(&bytes)
        .expect("exception-control module must validate");
    bytes
}

#[test]
fn modern_exception_modules_match_the_executed_fixtures_byte_for_byte() {
    use Instruction::*;
    let result = BlockType::Result(ValType::I32);
    let reference = BlockType::Result(ValType::Ref(RefType::EXNREF));
    let payload_reference = BlockType::FunctionType(2);
    let tagged = || vec![Catch::One { tag: 0, label: 0 }];
    let bodies = [
        vec![Block(result), table(result, tagged()), I32Const(42), Throw(0), End, End, End],
        vec![
            Block(result), table(result, tagged()), Block(payload_reference),
            table(payload_reference, vec![Catch::OneRef { tag: 0, label: 0 }]),
            I32Const(73), Throw(0), End, End, ThrowRef, End, End, End,
        ],
        vec![
            Block(BlockType::Empty), table(BlockType::Empty, vec![Catch::All { label: 0 }]),
            I32Const(7), Throw(0), End, Unreachable, End, I32Const(11), End,
        ],
        vec![
            Block(result), table(result, tagged()), Block(reference),
            table(reference, vec![Catch::AllRef { label: 0 }]),
            I32Const(84), Throw(0), End, End, ThrowRef, End, End, End,
        ],
        vec![
            Block(result), table(result, tagged()), Block(result),
            table(result, vec![Catch::One { tag: 1, label: 0 }]),
            I32Const(25), Throw(0), End, End, End, End, End,
        ],
        vec![table(result, vec![]), I32Const(9), Br(0), Unreachable, End, End],
        vec![table(result, tagged()), I32Const(17), Throw(0), End, End],
        vec![
            Block(result), Block(result),
            table(result, vec![Catch::One { tag: 0, label: 0 }, Catch::One { tag: 0, label: 1 }]),
            I32Const(5), Throw(0), End, End, I32Const(10), I32Add, End, End,
        ],
        vec![table(result, tagged()), I32Const(29), End, End],
    ];
    assert_eq!(bodies.len(), fixtures::CASES.len());
    for ((name, bytes, _), instructions) in fixtures::CASES.iter().zip(bodies) {
        assert_eq!(encoded_module(&instructions, false), *bytes, "fixture {name}");
    }
    let trap = [
        Block(BlockType::Empty), table(BlockType::Empty, vec![Catch::All { label: 0 }]),
        Unreachable, End, End, I32Const(99), End,
    ];
    assert_eq!(encoded_module(&trap, false), fixtures::TRAP_NOT_CAUGHT);
}

#[test]
fn legacy_catch_delegate_and_rethrow_modules_keep_encoder_bytes_and_validate() {
    use Instruction::*;
    let result = BlockType::Result(ValType::I32);
    let bodies = [
        vec![Try(result), I32Const(42), Throw(0), Catch(0), End, End],
        vec![Try(result), I32Const(7), Throw(1), Catch(0), CatchAll, I32Const(8), End, End],
        vec![Try(result), Try(result), I32Const(5), Throw(0), Delegate(0), Catch(0), End, End],
        vec![
            Try(result), Block(result), Try(result), I32Const(6), Throw(0),
            Delegate(0), End, Catch(0), End, End,
        ],
        vec![
            Try(result), Try(result), I32Const(7), Throw(0), Catch(0), Drop,
            Block(BlockType::Empty), Rethrow(1), End, Unreachable, End, Catch(0), End, End,
        ],
        vec![Try(result), I32Const(8), Throw(0), Delegate(0), End],
    ];
    for instructions in bodies {
        let _ = encoded_module(&instructions, true);
    }
}
