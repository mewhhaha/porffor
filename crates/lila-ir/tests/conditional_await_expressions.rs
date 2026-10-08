use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, FunctionIr, StatementIr, TypedExpr};

#[path = "common/statement_awaits.rs"]
mod statement_awaits;

fn function(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("conditional source parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .expect("script")
        .functions
        .into_iter()
        .find(|function| function.name == "choose")
        .expect("async function")
}

fn flatten<'a>(statements: &'a [StatementIr], result: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        match statement {
            StatementIr::EmptyStatementCompletion(item) => {
                flatten(std::slice::from_ref(item.statement()), result)
            }
            StatementIr::LexicalBlock(statements) => flatten(statements, result),
            StatementIr::Block(block) => flatten(&block.statements, result),
            statement => result.push(statement),
        }
    }
}

fn final_write(branch: &StatementIr) -> &str {
    let StatementIr::LexicalBlock(statements) = branch else {
        panic!("branch prefix");
    };
    let Some(StatementIr::Expression(TypedExpr {
        expr: ExprIr::AssignIdentifier { name, .. },
        ..
    })) = statements.last()
    else {
        panic!("branch commits its result");
    };
    name
}

#[test]
fn selected_arms_and_result_cell_share_the_owned_join_before_the_next_await() {
    let function = function("async function choose(flag) { const result = flag ? await 1 : await 2; await 3; return result; }");
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let (name, plan) = statements
        .iter()
        .find_map(|statement| {
            let StatementIr::AsyncFunctionIf {
                then_branch,
                else_branch: Some(else_branch),
                plan,
                ..
            } = statement
            else {
                return None;
            };
            let name = final_write(then_branch);
            assert_eq!(final_write(else_branch), name);
            Some((name, *plan))
        })
        .expect("conditional owns its branch states");
    assert!(name.starts_with("$async.conditional.result."));
    assert_eq!(
        [
            plan.entry_state(),
            plan.then_entry_state(),
            plan.else_entry_state(),
            plan.exit_state()
        ],
        [0, 1, 3, 5]
    );
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == name));
    assert!(statements.iter().any(|statement| matches!(statement,
        StatementIr::Lexical { name: declared, .. } if declared == name)));
    assert!(statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 5,
            resume_state: 6,
            ..
        }
    )));
    assert!(statements.iter().any(|statement| matches!(statement,
        StatementIr::Lexical { name: declared, init: TypedExpr { expr: ExprIr::Identifier(read), .. }, .. }
            if declared == "result" && read == name)));
}

#[test]
fn nested_and_single_awaiting_arms_compose_in_invocation_and_outer_await_targets() {
    for source in [
        "async function choose(flag, other, f) { return f(flag ? (other ? await 1 : 2) : await 3); }",
        "async function choose(flag, C) { return new C(flag ? await 1 : 2); }",
        "async function choose(flag, tag) { return tag`${flag ? 1 : await 2}`; }",
        "async function choose(flag) { var result = await (flag ? await 1 : 2); return result; }",
        "async function choose(flag) { return await (flag ? await 1 : 2); }",
        "async function choose(flag) { if (flag ? await 1 : 0) return 3; return 4; }",
        "async function choose(flag) { throw (flag ? await 1 : 2); }",
    ] {
        function(source);
    }
}

#[test]
fn import_specifier_get_value_precedes_the_conditional_options_prefix() {
    let function = function("async function choose(flag, specifier, options) { return import(specifier, flag ? await options : undefined); }");
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let (specifier, options) = statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Return(TypedExpr {
                expr:
                    ExprIr::DynamicImport {
                        specifier,
                        options: Some(options),
                        ..
                    },
                ..
            }) => Some((specifier, options)),
            _ => None,
        })
        .expect("both evaluated import operands survive the branch owner");
    let ExprIr::Identifier(specifier_slot) = &specifier.expr else {
        panic!("raw specifier must be retained before options suspend");
    };
    let ExprIr::Identifier(options_slot) = &options.expr else {
        panic!("options must read the completed conditional result");
    };
    assert_ne!(specifier_slot, options_slot);
    let specifier_binding = function
        .owned_env_bindings
        .iter()
        .filter(|binding| &binding.name == specifier_slot)
        .collect::<Vec<_>>();
    let options_binding = function
        .owned_env_bindings
        .iter()
        .filter(|binding| &binding.name == options_slot)
        .collect::<Vec<_>>();
    assert_eq!(specifier_binding.len(), 1);
    assert_eq!(options_binding.len(), 1);
    assert_ne!(specifier_binding[0].slot, options_binding[0].slot);
    let capture = statements.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(original), .. }, .. }
            if name == specifier_slot && original == "specifier"))
        .expect("specifier GetValue capture");
    let branch = statements
        .iter()
        .position(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. }))
        .expect("options branch owner");
    assert!(capture < branch);
    let StatementIr::AsyncFunctionIf {
        then_branch,
        else_branch: Some(else_branch),
        ..
    } = statements[branch]
    else {
        unreachable!();
    };
    let result_slot = final_write(then_branch);
    assert_eq!(final_write(else_branch), result_slot);
    // Dynamic import retains the completed options GetValue in its own cell.
    // Follow that publication instead of assuming it reuses the branch cell.
    let options_capture = statements
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(read), .. }, .. }
            if name == options_slot && read == result_slot)
        })
        .expect("options retain the completed selected result");
    assert!(branch < options_capture);
}

#[test]
fn compound_optional_constructor_and_import_options_keep_unsupported_boundaries() {
    for (source, awaits) in [
        (
            "async function choose(flag, f) { f(missing &&= await 1); }",
            1,
        ),
        (
            "async function choose(flag, C) { new C(flag[await 0] ||= await 1); }",
            2,
        ),
        ("async function choose(flag) { missing &&= await 1; }", 1),
        (
            "async function choose(flag, other, f) { f(flag ? (other[await 0] &&= await 1) : 2); }",
            2,
        ),
        (
            "async function choose(flag) { import('leaf.js', missing &&= await 1); }",
            1,
        ),
        (
            "async function choose(flag) { while (flag) { flag ? await 1 : 2; } }",
            1,
        ),
        ("async function* choose(flag) { flag ? await 1 : 2; }", 1),
    ] {
        statement_awaits::assert_count(&function(source), awaits);
    }
}

#[test]
fn nested_function_suspension_does_not_turn_a_current_condition_into_a_branch_owner() {
    let function = function("async function choose(flag) { const nested = flag ? async () => await 1 : async () => await 2; await 3; return nested; }");
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    assert!(!statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. })));
    assert!(statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 0,
            resume_state: 1,
            ..
        }
    )));
}
