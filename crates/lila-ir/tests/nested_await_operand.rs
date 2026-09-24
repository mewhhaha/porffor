//! An Await whose operand itself suspends (`await f(await p)`) stages the
//! inner suspension first, in every statement shape that lowers an Await
//! directly: an expression statement, an assignment, a lexical initializer
//! and a `return`.

use lila_front::{parse, ParseOptions};
use lila_ir::{lower, FunctionIr, StatementIr};

fn lower_function(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("async function parses");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program
        .script
        .expect("script IR")
        .functions
        .into_iter()
        .find(|function| function.name == "run")
        .expect("async function")
}

/// `(suspend, resume)` of every `StatementIr::AsyncAwait`, in body order,
/// wherever the statement shape nests it.
fn await_states(statements: &[StatementIr]) -> Vec<(u32, u32)> {
    let text = format!("{statements:?}");
    let number = |rest: &str, field: &str| -> (u32, usize) {
        let start = rest.find(field).expect("await state field") + field.len();
        let digits: String = rest[start..]
            .chars()
            .take_while(char::is_ascii_digit)
            .collect();
        (digits.parse().expect("state number"), start + digits.len())
    };
    let mut states = Vec::new();
    let mut rest = text.as_str();
    while let Some(position) = rest.find("AsyncAwait {") {
        rest = &rest[position..];
        let (suspend, after_suspend) = number(rest, "suspend_state: ");
        let (resume, after_resume) = number(&rest[after_suspend..], "resume_state: ");
        states.push((suspend, resume));
        rest = &rest[after_suspend + after_resume..];
    }
    states
}

#[test]
fn every_direct_await_shape_stages_an_inner_await_before_the_outer_one() {
    for body in [
        "await f(await p);",
        "x = await f(await p);",
        "const y = await f(await p); x = y;",
        "return await f(await p);",
    ] {
        let source = format!(
            "var x; async function f(v) {{ return v; }} async function run(p) {{ {body} }}"
        );
        let function = lower_function(&source);
        let states = await_states(&function.body.statements);
        // Two suspensions in source-evaluation order: the operand's Await
        // resumes before the outer Await suspends.
        assert_eq!(states.len(), 2, "{body}: {states:?}");
        assert_eq!(states[0].1, states[1].0, "{body}: {states:?}");
        assert!(states[0].0 < states[1].0, "{body}: {states:?}");
    }
}
