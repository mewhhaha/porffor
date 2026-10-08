use std::collections::BTreeSet;

use lila_front::{parse, ParseOptions};

#[path = "common/statement_awaits.rs"]
mod statement_awaits;
use lila_ir::{
    lower, ExprIr, FunctionIr, KindSet, OptionalCallReferenceCaptureIr, OptionalChainOperationIr,
    PropertyKeyIr, StatementIr, TypedExpr,
};

fn choose(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("grouped Reference source parses");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program
        .script
        .expect("script")
        .functions
        .into_iter()
        .find(|function| function.name == "choose" || function.name.ends_with(".choose"))
        .expect("plain async function")
}

fn flatten<'a>(source: &'a [StatementIr], statements: &mut Vec<&'a StatementIr>) {
    for statement in source {
        match statement {
            StatementIr::EmptyStatementCompletion(item) => {
                flatten(std::slice::from_ref(item.statement()), statements)
            }
            StatementIr::LexicalBlock(source) => flatten(source, statements),
            StatementIr::Block(source) => flatten(&source.statements, statements),
            statement => statements.push(statement),
        }
    }
}

fn scoped(branch: &StatementIr) -> Vec<&StatementIr> {
    let StatementIr::LexicalBlock(source) = branch else {
        panic!("private completed arm scope");
    };
    let mut statements = Vec::new();
    flatten(source, &mut statements);
    statements
}

fn all_statements<'a>(statement: &'a StatementIr, result: &mut Vec<&'a StatementIr>) {
    match statement {
        StatementIr::EmptyStatementCompletion(item) => all_statements(item.statement(), result),
        StatementIr::LexicalBlock(source) => {
            for statement in source {
                all_statements(statement, result);
            }
        }
        StatementIr::Block(source) => {
            for statement in &source.statements {
                all_statements(statement, result);
            }
        }
        StatementIr::AsyncFunctionIf {
            then_branch,
            else_branch,
            ..
        }
        | StatementIr::If {
            then_branch,
            else_branch,
            ..
        } => {
            result.push(statement);
            all_statements(then_branch, result);
            if let Some(branch) = else_branch {
                all_statements(branch, result);
            }
        }
        statement => result.push(statement),
    }
}

fn value(statement: &StatementIr) -> Option<&TypedExpr> {
    match statement {
        StatementIr::Lexical { init, .. } => Some(init),
        StatementIr::Expression(value)
        | StatementIr::DeclarationEvaluation(value)
        | StatementIr::Return(value) => Some(value),
        _ => None,
    }
}

fn captures<'a>(value: &'a TypedExpr, result: &mut Vec<&'a OptionalCallReferenceCaptureIr>) {
    match &value.expr {
        ExprIr::CaptureOptionalCallReference(capture) => result.push(capture),
        ExprIr::AssignIdentifier { value, .. } => captures(value, result),
        ExprIr::Comma { lhs, rhs } => {
            captures(lhs, result);
            captures(rhs, result);
        }
        ExprIr::MaterializeBinding { value, body, .. } => {
            captures(value, result);
            captures(body, result);
        }
        _ => {}
    }
}

fn statement_captures<'a>(
    statements: &[&'a StatementIr],
) -> Vec<&'a OptionalCallReferenceCaptureIr> {
    let mut result = Vec::new();
    for statement in statements {
        if let Some(value) = value(statement) {
            captures(value, &mut result);
        }
    }
    result
}

fn identifier(value: &TypedExpr) -> &str {
    let ExprIr::Identifier(name) = &value.expr else {
        panic!("retained activation cell");
    };
    name
}

fn owned_slot(function: &FunctionIr, name: &str) -> u32 {
    let bindings = function
        .owned_env_bindings
        .iter()
        .filter(|binding| binding.name == name)
        .collect::<Vec<_>>();
    assert_eq!(
        bindings.len(),
        1,
        "{name}: {:?}",
        function.owned_env_bindings
    );
    bindings[0].slot
}

fn write(statement: &StatementIr) -> (&str, &TypedExpr) {
    let StatementIr::Expression(TypedExpr {
        expr: ExprIr::AssignIdentifier { name, value },
        ..
    }) = statement
    else {
        panic!("Normal-only arm result publication");
    };
    (name, value)
}

fn invocation<'a>(
    statements: &[&'a StatementIr],
) -> (&'a TypedExpr, &'a TypedExpr, &'a [TypedExpr]) {
    statements
        .iter()
        .filter_map(|statement| value(statement))
        .find_map(|value| {
            let ExprIr::CallIndirect {
                callee,
                this_arg: Some(receiver),
                args,
                ..
            } = &value.expr
            else {
                return None;
            };
            Some((callee.as_ref(), receiver.as_ref(), args.as_slice()))
        })
        .expect("outer invocation consumes the captured Reference")
}

fn value_invocation<'a>(statements: &[&'a StatementIr]) -> (&'a TypedExpr, &'a [TypedExpr]) {
    statements
        .iter()
        .filter_map(|statement| value(statement))
        .find_map(|value| {
            let ExprIr::CallIndirect {
                callee,
                this_arg: None,
                args,
                direct_eval,
                ..
            } = &value.expr
            else {
                return None;
            };
            assert!(direct_eval.is_none());
            Some((callee.as_ref(), args.as_slice()))
        })
        .expect("outer invocation consumes a completed Call Value")
}

#[test]
fn terminal_get_and_receiver_are_captured_once_before_unconditional_outer_arguments() {
    let function = choose(
        "async function choose(target, key, argument) {
        const result = (target?.[await key])(await argument);
        await 0;
        return result;
    }",
    );
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let (branch_index, skipped, selected, plan) = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::AsyncFunctionIf {
                then_branch,
                else_branch: Some(selected),
                plan,
                ..
            } => Some((index, then_branch.as_ref(), selected.as_ref(), plan)),
            _ => None,
        })
        .expect("the optional key owns its selected states");
    assert_eq!(
        [
            plan.entry_state(),
            plan.then_entry_state(),
            plan.else_entry_state(),
            plan.exit_state()
        ],
        [0, 1, 2, 4]
    );
    let skipped = scoped(skipped);
    assert!(statement_captures(&skipped).is_empty());
    assert!(!skipped
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncAwait { .. })));
    let (result_cell, skipped_value) = write(skipped.last().expect("skipped callee publication"));
    assert!(matches!(skipped_value.expr, ExprIr::Undefined));
    let selected = scoped(selected);
    let terminal = statement_captures(&selected);
    assert_eq!(
        terminal.len(),
        1,
        "the terminal Get cannot be replayed for receiver capture"
    );
    let terminal = terminal[0];
    let (selected_result, selected_value) =
        write(selected.last().expect("selected callee publication"));
    assert_eq!(selected_result, result_cell);
    let ExprIr::CaptureOptionalCallReference(published) = &selected_value.expr else {
        panic!("the actual terminal Get supplies the joined callee");
    };
    assert_eq!(published, terminal);
    assert!(
        !selected
            .iter()
            .filter_map(|statement| value(statement))
            .any(|value| matches!(
                value.expr,
                ExprIr::OptionalPropertyChain { .. } | ExprIr::PropertyRead { .. }
            )),
        "terminal Reference capture must not follow a separately materialized terminal Get"
    );
    let terminal_index = selected
        .iter()
        .position(|statement| statement_captures(&[*statement]).len() == 1)
        .expect("terminal Reference completion");
    let key_await = selected
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::AsyncAwait {
                    suspend_state: 2,
                    resume_state: 3,
                    ..
                }
            )
        })
        .expect("selected key Await");
    assert!(key_await < terminal_index);
    let ExprIr::OptionalPropertyChain { target, chain } = &terminal.chain_expression().expr else {
        panic!("existing checked optional Reference IR");
    };
    let saved_base = identifier(target);
    let [OptionalChainOperationIr::Property {
        key: PropertyKeyIr::StringExpr(key),
        shorted: false,
    }] = chain.as_slice()
    else {
        panic!("one selected terminal raw key and Get");
    };
    assert_eq!(
        key.possible_kinds,
        KindSet::all_runtime_tags(),
        "emitter owns ToPropertyKey"
    );
    let bases = statements
        .iter()
        .filter(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(source), .. }, .. }
            if name == saved_base && source == "target")
        })
        .count();
    assert_eq!(
        bases, 1,
        "the original base GetValue precedes the guarded key once"
    );
    let (callee, receiver, args) = invocation(&statements);
    let receiver = identifier(receiver);
    assert_eq!(receiver, terminal.receiver().storage_name());
    assert!(receiver.starts_with("$call.optional.base"));
    let callee = identifier(callee);
    let callee_capture = statements
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(source), .. }, .. }
            if name == callee && source == result_cell)
        })
        .expect("callee retained from joined result");
    let outer_await = statements
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::AsyncAwait {
                    suspend_state: 4,
                    resume_state: 5,
                    ..
                }
            )
        })
        .expect("unconditional outer argument");
    assert!(branch_index < callee_capture && callee_capture < outer_await);
    assert_eq!(args.len(), 1);
    let names = [
        saved_base,
        result_cell,
        receiver,
        callee,
        identifier(&args[0]),
    ];
    assert_eq!(
        names
            .map(|name| owned_slot(&function, name))
            .into_iter()
            .collect::<BTreeSet<_>>()
            .len(),
        names.len()
    );
    assert!(statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 5,
            resume_state: 6,
            ..
        }
    )));
}

#[test]
fn later_optional_guards_skip_the_whole_suffix_and_terminal_receiver_uses_its_last_base() {
    let function = choose(
        "async function choose(target, first, second, argument) {
        const result = (target?.[await first]?.[await second].method)(await argument);
        await 0;
        return result;
    }",
    );
    let mut top = Vec::new();
    flatten(&function.body.statements, &mut top);
    let mut all = Vec::new();
    for statement in &function.body.statements {
        all_statements(statement, &mut all);
    }
    let guards = all
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::AsyncFunctionIf {
                then_branch, plan, ..
            } => {
                let skipped = scoped(then_branch);
                assert!(statement_captures(&skipped).is_empty());
                assert!(!skipped
                    .iter()
                    .any(|statement| matches!(statement, StatementIr::AsyncAwait { .. })));
                Some((plan.entry_state(), plan.exit_state()))
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(guards, [(0, 8), (3, 7)]);
    let awaits = all
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => Some((*suspend_state, *resume_state)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(awaits, [(2, 3), (5, 6), (8, 9), (9, 10)]);
    let terminal = statement_captures(&all);
    assert_eq!(terminal.len(), 1);
    let terminal = terminal[0];
    let ExprIr::OptionalPropertyChain { target, chain } = &terminal.chain_expression().expr else {
        panic!("terminal eager suffix capture");
    };
    assert!(
        matches!(chain.last(), Some(OptionalChainOperationIr::Property {
        key: PropertyKeyIr::StaticString(name), shorted: false
    }) if name == "method")
    );
    let last_base = identifier(target);
    assert!(all.iter().any(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::OptionalPropertyChain { .. }, .. }, .. }
            if name == last_base)), "the last receiver comes from the completed preceding Get");
    let (_, receiver, _) = invocation(&top);
    assert_eq!(identifier(receiver), terminal.receiver().storage_name());
    owned_slot(&function, last_base);
    owned_slot(&function, identifier(receiver));
}

#[test]
fn grouped_tag_completes_reference_then_template_object_and_outer_substitutions() {
    let function = choose(
        "async function choose(target, key, first, second) {
        const result = (target?.[await key])`head${first}${await second}tail`;
        await 0;
        return result;
    }",
    );
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let branch = statements
        .iter()
        .position(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. }))
        .expect("tag Reference guard");
    let template = statements
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::Lexical {
                    init: TypedExpr {
                        expr: ExprIr::TemplateObject(_),
                        ..
                    },
                    ..
                }
            )
        })
        .expect("GetTemplateObject retained unconditionally");
    let first = statements.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { init: TypedExpr { expr: ExprIr::Identifier(name), .. }, .. } if name == "first"))
        .expect("first substitution retained unconditionally");
    let second = statements
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::AsyncAwait {
                    suspend_state: 4,
                    resume_state: 5,
                    ..
                }
            )
        })
        .expect("outer substitution Await");
    assert!(branch < template && template < first && first < second);
    let mut all = Vec::new();
    for statement in &function.body.statements {
        all_statements(statement, &mut all);
    }
    let captures = statement_captures(&all);
    assert_eq!(captures.len(), 1);
    let (_, receiver, args) = invocation(&statements);
    assert_eq!(identifier(receiver), captures[0].receiver().storage_name());
    assert_eq!(args.len(), 3);
    assert!(
        matches!(statements[template], StatementIr::Lexical { name, .. }
        if name == identifier(&args[0]))
    );
    assert!(statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 5,
            resume_state: 6,
            ..
        }
    )));
}

#[test]
fn erased_key_await_keeps_reference_capture_without_claiming_resumption_states() {
    let function = choose(
        "async function choose(target, key, argument) {
        return (target?.[(undefined)?.[await key]])(await argument);
    }",
    );
    let mut top = Vec::new();
    flatten(&function.body.statements, &mut top);
    assert!(top
        .iter()
        .any(|statement| matches!(statement, StatementIr::If { .. })));
    assert!(!top
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. })));
    let mut all = Vec::new();
    for statement in &function.body.statements {
        all_statements(statement, &mut all);
    }
    let awaits = all
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => Some((*suspend_state, *resume_state)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        awaits,
        [(0, 1)],
        "only the unconditional outer argument suspends"
    );
    let captures = statement_captures(&all);
    assert_eq!(
        captures.len(),
        1,
        "the completed eager arm still owns its terminal Reference"
    );
    let (_, receiver, _) = invocation(&top);
    assert_eq!(identifier(receiver), captures[0].receiver().storage_name());
    owned_slot(&function, identifier(receiver));
}

#[test]
fn terminal_call_publishes_a_value_before_unconditional_outer_arguments() {
    let function = choose(
        "async function choose(target, inner, first, second) {
        const result = (target?.make(await inner))(first, await second);
        await 0; return result;
    }",
    );
    let mut top = Vec::new();
    flatten(&function.body.statements, &mut top);
    let (branch_index, then_branch, else_branch, plan) = top
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::AsyncFunctionIf {
                then_branch,
                else_branch: Some(selected),
                plan,
                ..
            } => Some((index, then_branch.as_ref(), selected.as_ref(), plan)),
            _ => None,
        })
        .expect("the inner chain owns selected argument states");
    assert_eq!(
        [
            plan.entry_state(),
            plan.then_entry_state(),
            plan.else_entry_state(),
            plan.exit_state()
        ],
        [0, 1, 2, 4]
    );
    let skipped = scoped(then_branch);
    let selected = scoped(else_branch);
    assert_eq!(skipped.len(), 1);
    let (result, skipped_value) = write(skipped[0]);
    assert!(matches!(skipped_value.expr, ExprIr::Undefined));
    let (published, completed_call) = write(selected.last().unwrap());
    assert_eq!(published, result);
    let ExprIr::CallIndirect {
        callee: inner_callee,
        this_arg: Some(inner_receiver),
        args: inner_args,
        direct_eval,
        ..
    } = &completed_call.expr
    else {
        panic!("Normal publication consumes the actual completed terminal Call")
    };
    assert!(direct_eval.is_none());
    let capture = statement_captures(&selected);
    assert_eq!(capture.len(), 1);
    assert_eq!(
        identifier(inner_receiver),
        capture[0].receiver().storage_name()
    );
    let inner_await = selected
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::AsyncAwait {
                    suspend_state: 2,
                    resume_state: 3,
                    ..
                }
            )
        })
        .unwrap();
    assert!(!selected[..inner_await].iter().any(|statement| matches!(statement,
        StatementIr::Expression(TypedExpr { expr: ExprIr::AssignIdentifier { name, .. }, .. }) if name == result)),
        "the selected value cannot publish before argument resumption and Call");
    let (callee, args) = value_invocation(&top);
    let callee = identifier(callee);
    assert_eq!(args.len(), 2);
    let pin = top
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(saved), .. }, .. }
            if name == callee && saved == result)
        })
        .expect("outer callee pins the joined Call Value");
    let first = top
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(source), .. }, .. }
            if name == identifier(&args[0]) && source == "first")
        })
        .unwrap();
    let outer_await = top
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::AsyncAwait {
                    suspend_state: 4,
                    resume_state: 5,
                    ..
                }
            )
        })
        .unwrap();
    assert!(branch_index < pin && pin < first && first < outer_await);
    let mut all = Vec::new();
    for statement in &function.body.statements {
        all_statements(statement, &mut all);
    }
    assert_eq!(
        statement_captures(&all).len(),
        1,
        "no outer Reference capture replays an inner Get"
    );
    let actual_awaits = all
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => Some((*suspend_state, *resume_state)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(actual_awaits, [(2, 3), (4, 5), (5, 6)]);
    let names = [
        result,
        identifier(inner_callee),
        identifier(inner_receiver),
        identifier(&inner_args[0]),
        callee,
        identifier(&args[0]),
        identifier(&args[1]),
    ];
    assert_eq!(
        names
            .map(|name| owned_slot(&function, name))
            .into_iter()
            .collect::<BTreeSet<_>>()
            .len(),
        names.len()
    );
}

#[test]
fn terminal_call_tag_pins_value_before_the_owned_template_and_unconditional_substitutions() {
    let function = choose(
        "async function choose(target, inner, first, second) {
        const result = (target?.make(await inner))`head${first}${await second}tail`;
        await 0; return result;
    }",
    );
    let mut top = Vec::new();
    flatten(&function.body.statements, &mut top);
    let (branch_index, selected) = top
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::AsyncFunctionIf {
                else_branch: Some(selected),
                ..
            } => Some((index, scoped(selected))),
            _ => None,
        })
        .unwrap();
    let (joined, terminal) = write(selected.last().unwrap());
    assert!(matches!(
        terminal.expr,
        ExprIr::CallIndirect {
            this_arg: Some(_),
            ..
        }
    ));
    let (callee, args) = value_invocation(&top);
    assert_eq!(args.len(), 3);
    let callee = identifier(callee);
    let pin = top
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(saved), .. }, .. }
            if name == callee && saved == joined)
        })
        .unwrap();
    let (template_index, template) = top
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::Lexical {
                name,
                init:
                    TypedExpr {
                        expr: ExprIr::TemplateObject(template),
                        ..
                    },
                ..
            } if name == identifier(&args[0]) => Some((index, template)),
            _ => None,
        })
        .expect("the original template site supplies the cached object operand");
    let owner = function
        .template_source
        .expect("actual parsed template owner");
    assert_eq!(template.site_id.source(), owner.id());
    assert!(template.site_id.cache_slot() < owner.site_count());
    assert_eq!(template.raw, ["head", "", "tail"]);
    assert_eq!(
        template.cooked,
        [
            Some(String::from("head")),
            Some(String::new()),
            Some(String::from("tail"))
        ]
    );
    let first = top
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Identifier(source), .. }, .. }
            if name == identifier(&args[1]) && source == "first")
        })
        .unwrap();
    let awaited = top
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::AsyncAwait {
                    suspend_state: 4,
                    resume_state: 5,
                    ..
                }
            )
        })
        .unwrap();
    assert!(
        branch_index < pin && pin < template_index && template_index < first && first < awaited
    );
    let mut all = Vec::new();
    for statement in &function.body.statements {
        all_statements(statement, &mut all);
    }
    assert_eq!(
        statement_captures(&all).len(),
        1,
        "only the inner method retains its receiver"
    );
    let names = [
        joined,
        callee,
        identifier(&args[0]),
        identifier(&args[1]),
        identifier(&args[2]),
    ];
    assert_eq!(
        names
            .map(|name| owned_slot(&function, name))
            .into_iter()
            .collect::<BTreeSet<_>>()
            .len(),
        names.len()
    );
}

#[test]
fn target_only_await_retains_the_completed_base_and_reference_before_outer_spread_and_await() {
    let function = choose("async function choose(target, key, spread, argument) { const result = ((await target)?.[key].method)(...spread, await argument); await 0; return result; }");
    let mut top = Vec::new();
    flatten(&function.body.statements, &mut top);
    assert!(
        !top.iter()
            .any(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. })),
        "synchronous Property tail does not invent branch resume states"
    );
    let references = statement_captures(&top);
    assert_eq!(
        references.len(),
        1,
        "one complete synchronous chain owns the terminal Reference"
    );
    let reference = references[0];
    let ExprIr::OptionalPropertyChain { target, chain } = &reference.chain_expression().expr else {
        panic!("actual retained tail")
    };
    let base = identifier(target);
    assert!(matches!(chain.as_slice(), [
        OptionalChainOperationIr::Property { key: PropertyKeyIr::StringExpr(_), shorted: true },
        OptionalChainOperationIr::Property { key: PropertyKeyIr::StaticString(name), shorted: false }
    ] if name == "method"));
    let (callee, receiver, args) = invocation(&top);
    let callee = identifier(callee);
    let receiver = identifier(receiver);
    assert_eq!(receiver, reference.receiver().storage_name());
    let receiver_index = top.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Undefined, .. }, .. } if name == receiver)).unwrap();
    let base_index = top
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, .. } if name == base)
        })
        .unwrap();
    let capture_index = top.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::CaptureOptionalCallReference(_), .. }, .. } if name == callee)).unwrap();
    let awaits = top
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| match statement {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => Some((index, *suspend_state, *resume_state)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        awaits
            .iter()
            .map(|(_, entry, exit)| (*entry, *exit))
            .collect::<Vec<_>>(),
        [(0, 1), (1, 2), (2, 3)]
    );
    assert_eq!(args.len(), 2);
    let ExprIr::CapturedArgumentList(list) = &args[0].expr else {
        panic!("outer spread snapshot")
    };
    let spread = identifier(list.binding());
    let spread_index = top.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::CaptureArgumentList(_), .. }, .. } if name == spread)).unwrap();
    assert!(receiver_index < awaits[0].0 && awaits[0].0 < base_index && base_index < capture_index);
    assert!(
        capture_index < spread_index && spread_index < awaits[1].0,
        "completed callee and raw receiver precede all outer operands"
    );
    let names = [base, callee, receiver, spread, identifier(&args[1])];
    assert_eq!(
        names
            .map(|name| owned_slot(&function, name))
            .into_iter()
            .collect::<BTreeSet<_>>()
            .len(),
        names.len()
    );
}

#[test]
fn target_only_property_tag_uses_the_original_template_after_get_and_before_substitution_await() {
    let function = choose("async function choose(target, argument) { const result = ((await target)?.tag)`head${await argument}tail`; await 0; return result; }");
    let mut top = Vec::new();
    flatten(&function.body.statements, &mut top);
    let references = statement_captures(&top);
    assert_eq!(references.len(), 1);
    let (callee, receiver, args) = invocation(&top);
    let callee = identifier(callee);
    let receiver = identifier(receiver);
    assert_eq!(receiver, references[0].receiver().storage_name());
    assert_eq!(args.len(), 2);
    let capture_index=top.iter().position(|statement| matches!(statement,
        StatementIr::Lexical { name,init:TypedExpr {expr:ExprIr::CaptureOptionalCallReference(_),..},.. } if name==callee)).unwrap();
    let template_name = identifier(&args[0]);
    let (template_index, template) = top
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::Lexical {
                name,
                init:
                    TypedExpr {
                        expr: ExprIr::TemplateObject(template),
                        ..
                    },
                ..
            } if name == template_name => Some((index, template)),
            _ => None,
        })
        .unwrap();
    let owner = function.template_source.as_ref().unwrap();
    assert_eq!(template.site_id.source(), owner.id());
    assert!(template.site_id.cache_slot() < owner.site_count());
    assert_eq!(template.raw, ["head", "tail"]);
    let outer_await = top
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::AsyncAwait {
                    suspend_state: 1,
                    resume_state: 2,
                    ..
                }
            )
        })
        .unwrap();
    assert!(capture_index < template_index && template_index < outer_await);
    for name in [callee, receiver, template_name, identifier(&args[1])] {
        owned_slot(&function, name);
    }
}

#[test]
fn admitted_grouped_routes_and_existing_synchronous_and_target_only_values_compose() {
    for source in [
        "async function choose(target, key) { return (target?.[await key])(); }",
        "async function choose(target, key) { return await (target?.[await key])(); }",
        "async function choose(target, key) { return ((target?.[await key]))`${1}`; }",
        "async function choose(target, key, argument) { var result = (target?.[await key])(await argument); return result; }",
        "async function choose(target, key, argument) { (target?.[await key])(await argument); }",
        "async function choose(target, key, flag) { return (target?.[flag ? await key : 'method'])(1); }",
        "async function choose(target, key, argument) { return (target?.[(undefined)?.[await key]])(await argument); }",
        "async function choose(target, argument) { return (target?.method)(await argument); }",
        "async function choose(target, argument) { return (target?.method)`head${await argument}tail`; }",
        "async function choose(target) { return (await target)?.method; }",
        "async function choose(target) { return (await target)?.(); }",
        "async function choose(target) { return ((await target)?.method)(1); }",
        "async function choose(target) { return ((await target)?.method)`${1}`; }",
        "async function choose(target, argument) { return ((await target)?.make().method)(await argument); }",
    ] { choose(source); }
}

#[test]
fn unowned_references_and_nonordinary_owners_are_refused_before_consumption() {
    for source in [
        "async function* choose(target) { return ((await target)?.method)`${yield 1}`; }",
        "async function* choose(target, key) { return (target?.[await key])(); }",
        "async function* choose(target, key) { return (target?.[await key])`${yield 1}`; }",
    ] {
        assert!(choose(source).resumable_plan.is_some());
    }
    for source in [
        "async function choose(target, key) { return delete target?.[await key]; }",
        "class C { #value = 1; async choose(target, key) { return (target?.[await key].#value)(); } }",
        "class C extends Object { async choose(key) { return (super.value?.[await key])(); } }",
    ] { choose(source); }
    for source in [
        "async function choose(target) { while (true) { return ((await target)?.method)(1); } }",
        "async function choose(target, key) { while ((target?.[await key])()) { break; } }",
        "async function choose(target, key, flag) { while (flag) { (target?.[await key])(); } }",
        "async function choose(target, key, flag) { for (; flag; ) { (target?.[await key])`${1}`; } }",
        "async function choose(target, key, flag) { do { (target?.[await key])(); } while (flag); }",
    ] {
        statement_awaits::assert_count(&choose(source), 1);
    }
}
