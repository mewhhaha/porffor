use lila_front::{parse, ParseOptions};
use lila_ir::{lower, BlockIr, ExprIr, KindSet, StatementIr, TypedExpr, ValueKind};

fn final_expression(block: &BlockIr) -> &TypedExpr {
    match block.statements.last().expect("catch body is nonempty") {
        StatementIr::Expression(expression) => expression,
        StatementIr::Block(block) => final_expression(block),
        other => panic!("expected catch binding read: {other:?}"),
    }
}

#[test]
fn constructor_body_throws_are_not_narrowed_by_a_later_explicit_throw() {
    for source in [
        "try { new Intl.NumberFormat(null); throw 'sentinel'; } catch (caught) { caught; }",
        "function C() { throw {answer:42}; } try { new C(); throw 'sentinel'; } catch (caught) { caught; }",
        "function C() { throw undefined; } try { new C(); throw {}; } catch (caught) { caught; }",
    ] {
        let parsed = parse(source, ParseOptions::script()).expect("constructor fixture parses");
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.expect("script IR");
        let catch = script.body.statements.iter().find_map(|statement| match statement {
            StatementIr::TryCatch { try_block, catch_block, .. } => {
                assert!(try_block.statements.iter().any(|statement| matches!(statement,
                    StatementIr::Expression(TypedExpr { expr: ExprIr::Construct { .. }, .. })
                )), "fixture must retain a generic Construct: {source}");
                Some(catch_block)
            },
            _ => None,
        }).expect("top-level catch");
        let read = final_expression(catch);
        assert_eq!(read.kind, ValueKind::Dynamic, "{source}");
        assert_eq!(read.possible_kinds, KindSet::all_runtime_tags(), "{source}");
        assert!(read.heap_shape.is_none(), "{source}");
    }
}
