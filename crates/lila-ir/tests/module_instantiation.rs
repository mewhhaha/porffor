use lila_front::{parse, ParseGoal, ParseOptions, ParsedSource};
use lila_ir::{
    lower_module_graph, lower_script_graph, ExprIr, FunctionExecutionKind, FunctionFlavor,
    FunctionProtocolIr, ModuleEvaluationModeIr, ModuleGraphSources, ModuleKey, ModuleSourceIr,
    ProgramIr, ScriptIr, StatementIr, SynchronousModuleGraphIr,
};

fn sources(files: &[(&str, &str)], goal: ParseGoal) -> ModuleGraphSources {
    let modules: Vec<_> = files
        .iter()
        .enumerate()
        .map(|(index, (name, source))| {
            let key = ModuleKey::from_host(*name);
            let url = format!("file:///{}", key.as_str());
            if index == 0 && goal == ParseGoal::Script {
                let ParsedSource::Script(parsed) =
                    parse(*source, ParseOptions::script()).expect("Script fixture parses")
                else {
                    panic!("Script parse goal");
                };
                ModuleSourceIr::from_parsed_script(key, url, parsed)
            } else {
                ModuleSourceIr::new(key, (*source).into(), url)
            }
        })
        .collect();
    let mut resolutions = Vec::new();
    for (referrer, module) in modules.iter().enumerate() {
        for request in module.module_requests().unwrap_or_default() {
            let target = files
                .iter()
                .position(|(name, _)| *name == request.specifier().trim_start_matches("./"))
                .expect("fixture target exists");
            resolutions.push((referrer as u32, request, target as u32));
        }
    }
    ModuleGraphSources {
        modules,
        entry: 0,
        resolutions,
    }
}

fn module_program(files: &[(&str, &str)]) -> ProgramIr {
    let program = lower_module_graph(&sources(files, ParseGoal::Module));
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
}

fn activation_graph(script: &ScriptIr) -> Option<&SynchronousModuleGraphIr> {
    script.body.statements.iter().find_map(|statement| {
        let StatementIr::Expression(expression) = statement else {
            return None;
        };
        let ExprIr::SynchronousModuleGraph(graph) = &expression.expr else {
            return None;
        };
        Some(graph.as_ref())
    })
}

#[test]
fn nested_module_import_captures_do_not_infer_source_placeholder_values() {
    module_program(&[
        (
            "entry.js",
            "import defer * as namespace from './value.js'; import { value } from './value.js'; for (const read of [() => namespace.value, () => value.property]) print(read());",
        ),
        ("value.js", "export let value = { property: 3 };"),
    ]);
}

#[test]
fn synchronous_deferred_modules_have_private_owners_and_canonical_environment_slots() {
    let program = module_program(&[
        ("entry.js", "import defer * as first from './first.js'; print(first.value);"),
        ("first.js", "import { value as next } from './last.js'; export let value = next; export function read() { return next; }"),
        ("last.js", "export const { value } = { value: 3 };"),
    ]);
    let modules = program.modules.as_ref().expect("linked graph");
    assert_eq!(modules.evaluation_mode(0), ModuleEvaluationModeIr::Eager);
    assert_eq!(modules.evaluation_mode(1), ModuleEvaluationModeIr::Deferred);
    assert_eq!(modules.evaluation_mode(2), ModuleEvaluationModeIr::Deferred);
    let script = program.script.as_ref().expect("linked Script IR");
    let graph = activation_graph(script).expect("private graph operation");
    assert_eq!(graph.record_count, 3);
    assert_eq!(graph.activations.len(), 3);
    assert!(
        !script
            .global_bindings
            .lexical_names()
            .any(|name| name.contains("namespace")),
        "private namespace cells must not become Script bindings"
    );
    for activation in &graph.activations {
        let owner = script
            .functions
            .iter()
            .find(|function| function.id == activation.function)
            .expect("private owner remains reachable");
        assert_eq!(owner.protocol, FunctionProtocolIr::ModuleActivation);
        assert_eq!(owner.protocol.flavor(), FunctionFlavor::Arrow);
        assert_eq!(
            owner.protocol.execution_kind(),
            FunctionExecutionKind::Generator
        );
        assert!(!owner.protocol.is_constructable());
        let suspension = owner
            .generator_plan
            .as_ref()
            .expect("private suspension plan");
        assert_eq!(suspension.state_count, 2);
        assert_eq!(suspension.suspension_points.len(), 1);
        assert!(owner.strict);
        assert!(!owner.captures_lexical_arguments);
        assert!(script
            .functions
            .iter()
            .any(|function| function.id == activation.evaluator));
    }
    let last = script
        .functions
        .iter()
        .find(|function| function.id == graph.activations[2].function)
        .unwrap();
    assert!(last
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == "value"));
}

#[test]
fn private_source_spans_survive_ecmascript_line_terminators_and_ordinary_arrays() {
    for terminator in ["\r", "\r\n", "\n", "\u{2028}", "\u{2029}"] {
        let source = format!(
            "const ordinary = [async () => {{ return 1; }}, () => 0];{terminator}\
             export default function () {{ return ordinary; }}"
        );
        let program = module_program(&[
            (
                "entry.js",
                "import defer * as ns from './value.js'; print(ns.default.name);",
            ),
            ("value.js", &source),
        ]);
        let script = program.script.as_ref().unwrap();
        assert_eq!(activation_graph(script).unwrap().activations.len(), 2);
        assert_eq!(
            script
                .functions
                .iter()
                .filter(|function| function.protocol == FunctionProtocolIr::ModuleActivation)
                .count(),
            2
        );
        assert!(script
            .functions
            .iter()
            .any(|function| function.protocol == FunctionProtocolIr::AsyncArrow));
    }
}

#[test]
fn tla_script_entries_and_source_phase_keep_their_existing_driver() {
    let asynchronous = module_program(&[
        (
            "entry.js",
            "import defer * as ns from './value.js'; await 0; print(ns.value);",
        ),
        ("value.js", "export const value = 2;"),
    ]);
    assert!(activation_graph(asynchronous.script.as_ref().unwrap()).is_none());
    let script = lower_script_graph(&sources(
        &[
            (
                "entry.js",
                "import.defer('./value.js').then(ns => print(ns.value));",
            ),
            ("value.js", "export const value = 2;"),
        ],
        ParseGoal::Script,
    ));
    assert!(script.is_wasm_supported(), "{:?}", script.diagnostics);
    assert!(activation_graph(script.script.as_ref().unwrap()).is_none());
    let source_phase = module_program(&[
        ("entry.js", "import source source from './source.js'; import defer * as ns from './value.js'; source; print(ns.value);"),
        ("source.js", "export const value = 1;"),
        ("value.js", "export const value = 2;"),
    ]);
    assert!(activation_graph(source_phase.script.as_ref().unwrap()).is_none());
}

#[test]
fn original_module_parse_rejects_syntax_an_async_wrapper_could_otherwise_admit() {
    for source in ["return 1;", "new.target;", "yield 1;", "with ({}) {}"] {
        let graph = sources(
            &[
                ("entry.js", "import defer * as ns from './invalid.js'; ns;"),
                ("invalid.js", source),
            ],
            ParseGoal::Module,
        );
        let program = lower_module_graph(&graph);
        assert!(!program.is_wasm_supported(), "{source}");
        assert!(
            program.script.is_none(),
            "the rejected Module must not reach generated Script parsing"
        );
        assert!(!program.diagnostics.is_empty());
    }
}

#[test]
fn global_script_prelude_is_separate_from_module_and_dynamic_eval_candidates() {
    let ParsedSource::Script(prelude) = parse(
        "function helper() { return this; } let globalLexical = 4;",
        ParseOptions::script(),
    )
    .expect("global Script parses") else {
        panic!("Script parse goal")
    };
    let program = lila_ir::lower_module_graph_with_prelude(
        &sources(
            &[
                (
                    "entry.js",
                    "import defer * as ns from './dependency.js'; print(helper(), ns.value);",
                ),
                ("dependency.js", "export const value = globalLexical;"),
            ],
            ParseGoal::Module,
        ),
        &prelude,
        lila_ir::HostSurfacePolicy::Test262,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let prelude = script
        .module_prelude
        .as_ref()
        .expect("separate Script thunk");
    assert_eq!(prelude.kind, lila_ir::PreparedScriptKind::RealmScript);
    assert!(!prelude.strict);
    assert!(prelude
        .global_bindings
        .iter()
        .any(|binding| binding.name == "helper"));
    assert!(prelude
        .global_bindings
        .lexical_names()
        .any(|name| name == "globalLexical"));
    assert!(
        script.prepared_scripts.is_empty(),
        "prelude is not an eval candidate"
    );
    assert!(activation_graph(&script).is_some());
    let helper = script
        .functions
        .iter()
        .find(|function| function.name == "helper")
        .unwrap();
    assert!(
        !helper.strict,
        "Module strictness must not affect a Script function"
    );
    let ids: std::collections::BTreeSet<_> = script.functions.iter().map(|f| &f.id).collect();
    assert_eq!(
        ids.len(),
        script.functions.len(),
        "source units share allocation IDs"
    );
}

#[test]
fn retained_module_drivers_have_private_lexical_owners_outside_global_script() {
    let ParsedSource::Script(prelude) = parse(
        "const shared = 'global'; function helperRead() { return shared; }",
        ParseOptions::script(),
    )
    .expect("global Script parses") else {
        panic!("Script parse goal")
    };
    for (files, protocol) in [
        (
            vec![("entry.js", "await 0; const shared = 'entry'; var localVar; function localFunction() {} helperRead();")],
            FunctionProtocolIr::AsyncArrow,
        ),
        (
            vec![
                ("entry.js", "import source source from './source.js'; const shared = 'entry'; var localVar; function localFunction() {} helperRead();"),
                ("source.js", "export const unused = 1;"),
            ],
            FunctionProtocolIr::Arrow,
        ),
    ] {
        let program = lila_ir::lower_module_graph_with_prelude(
            &sources(&files, ParseGoal::Module),
            &prelude,
            lila_ir::HostSurfacePolicy::Test262,
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("Module Script IR");
        assert!(activation_graph(script).is_none(), "retained driver control");
        assert!(script.global_bindings.lexical_names().next().is_none());
        assert!(script.global_bindings.iter().all(|binding| {
            binding.declarations == lila_ir::GlobalDeclarationSetIr::None
        }), "the private owner and its declarations must not create global bindings");
        assert!(script.prepared_scripts.is_empty(), "the private owner must not enter eval candidate tables");
        let calls = script.body.statements.iter().filter_map(|statement| {
            let StatementIr::Expression(expression) = statement else {
                return None;
            };
            let ExprIr::ModuleEntryEvaluation(entry) = &expression.expr else {
                return None;
            };
            let ExprIr::CallIndirect { callee, args, .. } = &entry.evaluation().expr else {
                return None;
            };
            let ExprIr::FunctionValue(target) = &callee.expr else {
                return None;
            };
            assert!(args.is_empty(), "the private driver has no parameters");
            Some(target)
        });
        let calls = calls.collect::<Vec<_>>();
        assert_eq!(calls.len(), 1, "one top-level retained-driver invocation");
        let owner = script
            .functions
            .iter()
            .find(|function| &function.id == calls[0])
            .expect("the invoked driver has a lowered owner");
        assert_eq!(owner.protocol, protocol);
        assert!(!owner.is_nested);
        assert!(owner.strict);
        assert!(!owner.protocol.is_constructable());
        assert!(!owner.captures_lexical_arguments);
        let prelude = script.module_prelude.as_ref().unwrap();
        assert!(prelude.global_bindings.lexical_names().any(|name| name == "shared"));
    }
}

#[test]
fn ordinary_modules_with_colliding_names_use_distinct_activation_owners() {
    let program = module_program(&[
        ("entry.js", "import { read as imported } from './dependency.js'; const shared = 'entry'; var localVar; function localFunction() {} print(imported(), shared);"),
        ("dependency.js", "const shared = 'dependency'; var localVar; function localFunction() {} export function read() { return shared; }"),
    ]);
    let script = program.script.as_ref().unwrap();
    let graph = activation_graph(script).expect("ordinary Module graph uses canonical owners");
    assert_eq!(graph.activations.len(), 2);
    assert_ne!(graph.activations[0].function, graph.activations[1].function);
    assert!(script.global_bindings.lexical_names().next().is_none());
    assert!(script
        .global_bindings
        .iter()
        .all(|binding| { binding.declarations == lila_ir::GlobalDeclarationSetIr::None }));
    assert!(script.prepared_scripts.is_empty());
    for activation in &graph.activations {
        let owner = script
            .functions
            .iter()
            .find(|function| function.id == activation.function)
            .unwrap();
        assert_eq!(owner.protocol, FunctionProtocolIr::ModuleActivation);
        assert!(owner.strict);
        assert!(!owner.captures_lexical_arguments);
        assert!(owner
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == "shared"));
    }
}

#[test]
fn synchronous_cycle_plans_keep_ordered_dependencies_and_complete_components() {
    let program = module_program(&[
        (
            "entry.js",
            "import './left.js'; import './external.js'; import './right.js'; print('entry');",
        ),
        ("left.js", "import './entry.js'; print('left');"),
        ("external.js", "print('external');"),
        ("right.js", "import './entry.js'; print('right');"),
    ]);
    let script = program.script.as_ref().unwrap();
    let graph = activation_graph(script).expect("evaluation cycles use canonical owners");
    assert_eq!(graph.activations.len(), 4);
    let expected_dependencies = [vec![1, 2, 3], vec![0], vec![], vec![0]];
    for activation in &graph.activations {
        let evaluator = script
            .functions
            .iter()
            .find(|function| function.id == activation.evaluator)
            .unwrap();
        let plan = evaluator
            .body
            .statements
            .iter()
            .find_map(|statement| {
                let StatementIr::Return(expression) = statement else {
                    return None;
                };
                let ExprIr::ModuleEvaluate(plan) = &expression.expr else {
                    return None;
                };
                Some(plan)
            })
            .expect("the registered evaluator consumes a typed evaluation plan");
        assert_eq!(plan.module(), activation.module);
        assert_eq!(
            plan.dependencies(),
            expected_dependencies[activation.module as usize]
        );
        let mut members = plan.component_members().to_vec();
        members.sort_unstable();
        assert_eq!(
            members,
            if activation.module == 2 {
                vec![2]
            } else {
                vec![0, 1, 3]
            }
        );
    }
    let initial = script
        .body
        .statements
        .iter()
        .filter_map(|statement| {
            let StatementIr::Expression(expression) = statement else {
                return None;
            };
            let ExprIr::ModuleEntryEvaluation(entry) = &expression.expr else {
                return None;
            };
            let ExprIr::ModuleEvaluate(plan) = &entry.evaluation().expr else {
                panic!("the synchronous entry owns its actual evaluation operation");
            };
            Some(plan.module())
        })
        .collect::<Vec<_>>();
    assert_eq!(
        initial,
        [0],
        "static dependencies must be visited through entry DFS"
    );
}

#[test]
fn deferred_evaluation_components_remain_deferred_and_self_cycles_are_admitted() {
    let program = module_program(&[
        (
            "entry.js",
            "import defer * as cycle from './left.js'; print(cycle.value);",
        ),
        ("left.js", "import './right.js'; export const value = 1;"),
        ("right.js", "import './left.js';"),
    ]);
    let modules = program.modules.as_ref().unwrap();
    assert_eq!(modules.evaluation_mode(1), ModuleEvaluationModeIr::Deferred);
    assert_eq!(modules.evaluation_mode(2), ModuleEvaluationModeIr::Deferred);
    assert_eq!(
        activation_graph(program.script.as_ref().unwrap())
            .unwrap()
            .activations
            .len(),
        3
    );
    let self_cycle = module_program(&[(
        "entry.js",
        "import * as self from './entry.js'; export const value = 1; print(self.value);",
    )]);
    assert_eq!(
        activation_graph(self_cycle.script.as_ref().unwrap())
            .unwrap()
            .activations
            .len(),
        1
    );
}

#[test]
fn synchronous_module_try_scopes_do_not_allocate_source_resume_states() {
    fn flatten_scopes<'a>(statements: &'a [StatementIr], flattened: &mut Vec<&'a StatementIr>) {
        for statement in statements {
            match statement {
                StatementIr::Block(block) => flatten_scopes(&block.statements, flattened),
                StatementIr::LexicalBlock(statements) => flatten_scopes(statements, flattened),
                statement => flattened.push(statement),
            }
        }
    }

    for prefix in ["", "import defer * as namespace from './dependency.js';"] {
        let source = format!(
            r#"{prefix}
for (let index = 0; index < 2; index++) {{
  try {{ throw index; }} catch (error) {{ print(error); }}
  try {{ print(index); }} finally {{ print('finally'); }}
  try {{ throw index; }} catch (error) {{ print(error); }} finally {{ print('both'); }}
}}
print('after loop');
"#
        );
        let program = module_program(&[
            ("entry.js", &source),
            ("dependency.js", "export const value = 1;"),
        ]);
        let script = program.script.as_ref().unwrap();
        let activation = activation_graph(script)
            .unwrap()
            .activations
            .iter()
            .find(|activation| activation.module == 0)
            .unwrap();
        let owner = script
            .functions
            .iter()
            .find(|function| function.id == activation.function)
            .unwrap();
        let plan = owner.generator_plan.as_ref().unwrap();
        assert_eq!(plan.state_count, 2);
        assert_eq!(plan.suspension_points.len(), 1);
        assert_eq!(plan.suspension_points[0].suspend_state, 0);
        assert_eq!(plan.suspension_points[0].resume_state, 1);
        let mut statements = Vec::new();
        flatten_scopes(&owner.body.statements, &mut statements);
        assert_eq!(
            statements
                .iter()
                .filter(|statement| matches!(statement, StatementIr::GeneratorYield { .. }))
                .count(),
            1
        );
        let loop_index = statements
            .iter()
            .position(|statement| matches!(statement, StatementIr::For { .. }))
            .expect("source loop retains ordinary control flow");
        assert!(statements[loop_index + 1..]
            .iter()
            .any(|statement| matches!(statement, StatementIr::Expression(_))));
        let StatementIr::For { body, .. } = statements[loop_index] else {
            unreachable!();
        };
        let StatementIr::Block(block) = body.as_ref() else {
            panic!("loop fixture owns a block");
        };
        let mut loop_statements = Vec::new();
        flatten_scopes(&block.statements, &mut loop_statements);
        let mut forms = Vec::new();
        for statement in loop_statements {
            let (form, generator_plan, async_plan) = match statement {
                StatementIr::TryCatch {
                    generator_plan,
                    async_plan,
                    ..
                } => ("catch", generator_plan, async_plan),
                StatementIr::TryFinally {
                    generator_plan,
                    async_plan,
                    ..
                } => ("finally", generator_plan, async_plan),
                StatementIr::TryCatchFinally {
                    generator_plan,
                    async_plan,
                    ..
                } => ("catch finally", generator_plan, async_plan),
                _ => continue,
            };
            assert!(
                generator_plan.is_none(),
                "module {form} cannot change the private boundary state"
            );
            assert!(
                async_plan.is_none(),
                "a synchronous module has no async try plan"
            );
            forms.push(form);
        }
        assert_eq!(forms, ["catch", "finally", "catch finally"]);
    }
}

#[test]
fn synchronous_module_resources_have_no_suspension_lifetime() {
    let program = module_program(&[(
        "entry.js",
        "using resource = { [Symbol.dispose]() {} }; try { throw 1; } catch (error) { print(error); } print('after catch');",
    )]);
    let script = program.script.as_ref().unwrap();
    let activation = activation_graph(script)
        .unwrap()
        .activations
        .first()
        .unwrap();
    let owner = script
        .functions
        .iter()
        .find(|function| function.id == activation.function)
        .unwrap();
    fn resource_scope(
        statements: &[StatementIr],
    ) -> Option<&lila_ir::SyncDisposableScopeExecutionIr> {
        statements.iter().find_map(|statement| match statement {
            StatementIr::SyncDisposableScope { execution, .. } => Some(execution),
            StatementIr::Block(block) => resource_scope(&block.statements),
            StatementIr::LexicalBlock(statements) => resource_scope(statements),
            _ => None,
        })
    }
    assert!(matches!(
        resource_scope(&owner.body.statements),
        Some(lila_ir::SyncDisposableScopeExecutionIr::Immediate)
    ));
}

#[test]
fn synchronous_module_resource_forms_use_the_canonical_owner_lifetime() {
    for source in [
        "using resource = null; print('after');",
        "{ using resource = null; } print('after');",
        "for (using resource = null; false;) {} print('after');",
        "for (using resource of [null]) {} print('after');",
        "function nested() { using resource = null; return 1; } print(nested());",
    ] {
        let program = module_program(&[("entry.js", source)]);
        let script = program.script.as_ref().unwrap();
        let activation = activation_graph(script)
            .unwrap()
            .activations
            .first()
            .unwrap();
        let owner = script
            .functions
            .iter()
            .find(|function| function.id == activation.function)
            .unwrap();
        assert_eq!(owner.protocol, FunctionProtocolIr::ModuleActivation);
        let plan = owner.generator_plan.as_ref().unwrap();
        assert_eq!(plan.state_count, 2, "{source}");
        assert_eq!(plan.suspension_points.len(), 1, "{source}");
    }
}

#[test]
fn retained_module_drivers_do_not_gain_resource_admission() {
    for resource in [
        "using resource = null;",
        "for (using resource = null; false;) {}",
        "for (using resource of [null]) {}",
    ] {
        for prefix in ["await 0;", "import source source from './source.js';"] {
            let entry = format!("{prefix} {resource}");
            let program = lower_module_graph(&sources(
                &[
                    ("entry.js", &entry),
                    ("source.js", "export const value = 1;"),
                ],
                ParseGoal::Module,
            ));
            assert!(!program.is_wasm_supported(), "{entry}");
            assert!(
                program.diagnostics.iter().any(|diagnostic| diagnostic
                    .message
                    .contains("using declaration in a module without a canonical execution owner")),
                "{entry}: {:?}",
                program.diagnostics
            );
            if let Some(script) = program.script.as_ref() {
                assert!(activation_graph(script).is_none());
            }
        }
    }
}
