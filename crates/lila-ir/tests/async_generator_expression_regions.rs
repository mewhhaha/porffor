use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, FunctionIr, ResumableSuspensionKindIr, StatementIr, TypedExpr};

fn values(source: &str) -> FunctionIr {
    let program = lower(&parse(source, ParseOptions::script()).unwrap());
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program
        .script
        .unwrap()
        .functions
        .into_iter()
        .find(|function| function.name == "values" || function.name.ends_with(".values"))
        .unwrap()
}

fn statements<'a>(items: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for item in items {
        output.push(item);
        match item {
            StatementIr::Block(block) => statements(&block.statements, output),
            StatementIr::LexicalBlock(items) => statements(items, output),
            StatementIr::EmptyStatementCompletion(item) => {
                statements(std::slice::from_ref(item.statement()), output)
            }
            StatementIr::AsyncGeneratorIf(plan) => {
                statements(&plan.condition().region().block().statements, output);
                statements(&plan.then_branch().block().statements, output);
                statements(&plan.else_branch().block().statements, output);
            }
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            } => {
                statements(std::slice::from_ref(then_branch.as_ref()), output);
                if let Some(branch) = else_branch {
                    statements(std::slice::from_ref(branch.as_ref()), output);
                }
            }
            StatementIr::ResumableClassDefinition(plan) => {
                for prefix in plan.prefixes() {
                    statements(prefix.statements(), output);
                }
            }
            _ => {}
        }
    }
}

fn expressions<'a>(value: &'a TypedExpr, output: &mut Vec<&'a TypedExpr>) {
    output.push(value);
    match &value.expr {
        ExprIr::AssignIdentifier { value, .. } => expressions(value, output),
        ExprIr::Comma { lhs, rhs } => {
            expressions(lhs, output);
            expressions(rhs, output);
        }
        ExprIr::MaterializeBinding { value, body, .. } => {
            expressions(value, output);
            expressions(body, output);
        }
        _ => {}
    }
}

fn projected<'a>(items: &[&'a StatementIr]) -> Vec<&'a TypedExpr> {
    let mut output = Vec::new();
    for item in items {
        match item {
            StatementIr::Lexical { init, .. } => expressions(init, &mut output),
            StatementIr::Expression(value)
            | StatementIr::DeclarationEvaluation(value)
            | StatementIr::Return(value) => expressions(value, &mut output),
            _ => {}
        }
    }
    output
}

#[test]
fn mixed_literals_commit_original_accumulation_before_each_later_protocol() {
    let function = values("async function* values(source,p){const array=[...(yield source),await p,,yield 2];const object={[await p]:yield array,method(){return 1;}};yield object;}");
    let tape = &function.resumable_plan.as_ref().unwrap().suspension_points;
    assert_eq!(
        tape.iter().map(|point| point.kind).collect::<Vec<_>>(),
        [
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Yield,
        ]
    );
    assert!(tape
        .windows(2)
        .all(|pair| pair[0].resume_state == pair[1].suspend_state));
    let mut items = Vec::new();
    statements(&function.body.statements, &mut items);
    let expressions = projected(&items);
    let arrays = expressions
        .iter()
        .filter_map(|value| match &value.expr {
            ExprIr::ArrayAccumulation(plan) => Some(plan),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        arrays.len() >= 2,
        "completed literal prefixes flush before later suspension"
    );
    assert!(arrays
        .windows(2)
        .all(|pair| pair[0].target() == pair[1].target()));
    assert_eq!(
        expressions
            .iter()
            .filter(|value| matches!(value.expr, ExprIr::ObjectPropertyDefinition(_)))
            .count(),
        2
    );
    let mut slots = std::collections::BTreeSet::new();
    assert!(function
        .owned_env_bindings
        .iter()
        .all(|binding| slots.insert(binding.slot)));
}

#[test]
fn mixed_class_retains_original_name_environment_and_contiguous_evaluation_prefixes() {
    let function = values("async function* values(Base,key){const C=class Named extends(await(yield Base)){[(yield 'key',await key)](){return Named;}};yield C;}");
    let mut items = Vec::new();
    statements(&function.body.statements, &mut items);
    let plans = items
        .iter()
        .filter_map(|item| match item {
            StatementIr::ResumableClassDefinition(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(plans.len(), 1);
    let plan = plans[0];
    assert_eq!((plan.entry_state(), plan.exit_state()), (0, 4));
    assert_eq!(plan.class().name.as_deref(), Some("Named"));
    assert!(plan.class().name_binding.is_some());
    let name = plan.name_environment_binding().unwrap();
    assert_eq!(
        function
            .owned_env_bindings
            .iter()
            .filter(|binding| binding.name == name)
            .count(),
        1
    );
    let prefixes = plan.prefixes().collect::<Vec<_>>();
    assert!(prefixes
        .windows(2)
        .all(|pair| pair[0].exit_state() == pair[1].entry_state()));
    assert_eq!(
        function
            .resumable_plan
            .as_ref()
            .unwrap()
            .suspension_points
            .iter()
            .map(|point| point.kind)
            .collect::<Vec<_>>(),
        [
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
        ]
    );
}

#[test]
fn mixed_optional_operands_have_empty_skipped_arms_and_exact_selected_protocol_tape() {
    let function = values("async function* values(object,p){const value=(yield object)?.[await p]?.method(yield 1,await p);yield value;}");
    let mut items = Vec::new();
    statements(&function.body.statements, &mut items);
    let branches = items
        .iter()
        .filter_map(|item| match item {
            StatementIr::AsyncGeneratorIf(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(branches.len(), 3);
    for branch in branches {
        assert!(branch.condition().region().block().statements.is_empty());
        assert!(branch.then_branch().block().statements.is_empty());
        assert_eq!(
            branch.then_branch().entry_state(),
            branch.condition().region().end_state() + 1
        );
        assert_eq!(
            branch.else_branch().entry_state(),
            branch.then_branch().end_state() + 1
        );
        assert_eq!(branch.exit_state(), branch.else_branch().end_state() + 1);
        assert!(branch
            .else_branch()
            .block()
            .statements
            .iter()
            .any(|item| matches!(
                item,
                StatementIr::AsyncAwait { .. } | StatementIr::GeneratorYield { .. }
            )));
    }
    let plan = function.resumable_plan.as_ref().unwrap();
    assert_eq!(plan.state_count, 15);
    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|point| (point.kind, point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        [
            (ResumableSuspensionKindIr::Yield, 0, 1),
            (ResumableSuspensionKindIr::Await, 3, 4),
            (ResumableSuspensionKindIr::Yield, 7, 8),
            (ResumableSuspensionKindIr::Await, 11, 12),
            (ResumableSuspensionKindIr::Yield, 13, 14),
        ]
    );
}

#[test]
fn mixed_optional_delete_consumes_terminal_reference_and_grouped_call_keeps_receiver() {
    let deleted = values("async function* values(object,key){const deleted=delete((yield object)?.[await key]);yield deleted;}");
    let mut items = Vec::new();
    statements(&deleted.body.statements, &mut items);
    let exprs = projected(&items);
    assert_eq!(
        exprs
            .iter()
            .filter(|value| matches!(value.expr, ExprIr::DeleteProperty { .. }))
            .count(),
        1
    );
    assert!(!exprs.iter().any(|value| matches!(
        value.expr,
        ExprIr::OptionalPropertyChain { .. } | ExprIr::CaptureOptionalCallReference(_)
    )));
    let called = values("async function* values(object,key){const value=((yield object)?.[await key])(yield 2);yield value;}");
    let mut items = Vec::new();
    statements(&called.body.statements, &mut items);
    let exprs = projected(&items);
    assert_eq!(
        exprs
            .iter()
            .filter(|value| matches!(value.expr, ExprIr::CaptureOptionalCallReference(_)))
            .count(),
        1
    );
    assert!(exprs.iter().any(|value| matches!(
        &value.expr,
        ExprIr::CallIndirect {
            this_arg: Some(_),
            ..
        }
    )));
}

#[test]
fn mixed_private_in_super_get_and_update_consume_original_reference_operations() {
    let function = values("class Base{} class C extends Base{#value=1;async* values(p){const has=#value in(await(yield this));const read=super[(yield 'key',await p)];const old=(yield this)[await(yield 'update')]++;yield [has,read,old];}}");
    let mut items = Vec::new();
    statements(&function.body.statements, &mut items);
    let exprs = projected(&items);
    assert_eq!(
        exprs
            .iter()
            .filter(|value| matches!(value.expr, ExprIr::PrivateIn { .. }))
            .count(),
        1
    );
    assert_eq!(
        exprs
            .iter()
            .filter(|value| matches!(value.expr, ExprIr::SuperPropertyRead { .. }))
            .count(),
        1
    );
    let updates = exprs
        .iter()
        .filter_map(|value| match &value.expr {
            ExprIr::OrdinaryPropertyNumericUpdate(plan) => Some(plan),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(updates.len(), 1);
    let ExprIr::Identifier(base) = &updates[0].base_and_receiver().expr else {
        panic!("actual retained raw base")
    };
    assert_eq!(
        function
            .owned_env_bindings
            .iter()
            .filter(|binding| &binding.name == base)
            .count(),
        1
    );
    let lila_ir::PropertyKeyIr::StringExpr(key) = updates[0].referenced_name() else {
        panic!("actual computed raw key")
    };
    let ExprIr::Identifier(key) = &key.expr else {
        panic!("actual retained raw key")
    };
    assert_eq!(
        function
            .owned_env_bindings
            .iter()
            .filter(|binding| &binding.name == key)
            .count(),
        1
    );
    assert_ne!(base, key);
    assert_eq!(
        function
            .resumable_plan
            .as_ref()
            .unwrap()
            .suspension_points
            .iter()
            .map(|point| point.kind)
            .collect::<Vec<_>>(),
        [
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
        ]
    );
}

#[test]
fn mixed_annex_b_update_finishes_original_call_before_reference_error() {
    let function =
        values("async function* values(call){const result=call(await(yield 1))++;yield result;}");
    let mut items = Vec::new();
    statements(&function.body.statements, &mut items);
    let exprs = projected(&items);
    assert_eq!(
        exprs
            .iter()
            .filter(|value| matches!(value.expr, ExprIr::CallIndirect { .. }))
            .count(),
        1
    );
    assert_eq!(
        exprs
            .iter()
            .filter(|value| matches!(
                value.expr,
                ExprIr::RuntimeThrow {
                    name: lila_ir::NativeErrorKind::ReferenceError,
                    ..
                }
            ))
            .count(),
        1
    );
    assert_eq!(
        function
            .resumable_plan
            .as_ref()
            .unwrap()
            .suspension_points
            .iter()
            .map(|point| point.kind)
            .collect::<Vec<_>>(),
        [
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
        ]
    );
}

#[test]
fn mixed_private_optional_call_and_super_delete_keep_original_reference_roles() {
    let called=values("class C{#method(){} async*values(){const result=((yield this)?.#method)(await(yield 1));yield result;}}");
    let mut items = Vec::new();
    statements(&called.body.statements, &mut items);
    let exprs = projected(&items);
    let captures = exprs
        .iter()
        .filter_map(|value| match &value.expr {
            ExprIr::CaptureOptionalCallReference(capture) => Some(capture),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(captures.len(), 1);
    let ExprIr::OptionalPropertyChain { chain, .. } = &captures[0].chain_expression().expr else {
        panic!("original optional private Get carrier")
    };
    assert!(matches!(
        chain.as_slice(),
        [lila_ir::OptionalChainOperationIr::PrivateProperty { shorted: false, .. }]
    ));
    assert!(exprs.iter().any(|value| matches!(
        &value.expr,
        ExprIr::CallIndirect {
            this_arg: Some(_),
            ..
        }
    )));
    let deleted = values("class C{async*values(key){delete super[await(yield key)];}}");
    let mut items = Vec::new();
    statements(&deleted.body.statements, &mut items);
    let exprs = projected(&items);
    assert_eq!(
        exprs
            .iter()
            .filter(|value| matches!(
                value.expr,
                ExprIr::RuntimeThrow {
                    name: lila_ir::NativeErrorKind::ReferenceError,
                    ..
                }
            ))
            .count(),
        1
    );
    assert!(!exprs
        .iter()
        .any(|value| matches!(value.expr, ExprIr::SuperPropertyRead { .. })));
}

#[test]
fn mixed_import_stages_both_raw_operands_before_original_dynamic_import() {
    let function=values("async function* values(specifier,options){const result=import(await(yield specifier),await(yield options));yield result;}");
    let mut items = Vec::new();
    statements(&function.body.statements, &mut items);
    let exprs = projected(&items);
    let imports = exprs
        .iter()
        .filter_map(|value| match &value.expr {
            ExprIr::DynamicImport {
                specifier, options, ..
            } => Some((specifier, options)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(imports.len(), 1);
    let (specifier, options) = imports[0];
    let ExprIr::Identifier(specifier) = &specifier.expr else {
        panic!("retained raw import specifier")
    };
    let ExprIr::Identifier(options) = &options.as_ref().unwrap().expr else {
        panic!("retained raw import options")
    };
    assert_ne!(specifier, options);
    for name in [specifier, options] {
        assert_eq!(
            function
                .owned_env_bindings
                .iter()
                .filter(|binding| &binding.name == name)
                .count(),
            1
        );
    }
    assert_eq!(
        function
            .resumable_plan
            .as_ref()
            .unwrap()
            .suspension_points
            .iter()
            .map(|point| point.kind)
            .collect::<Vec<_>>(),
        [
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
        ]
    );
}
