use lila_front::{parse, ParseOptions};
use lila_ir::{lower, AsyncGeneratorForInIr, BindingMode, ExprIr, FunctionIr, StatementIr};

fn functions(source: &str) -> Vec<FunctionIr> {
    let program = lower(&parse(source, ParseOptions::script()).expect("ForIn source parses"));
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program.script.unwrap().functions
}

fn walk<'a>(statements: &'a [StatementIr], rows: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        rows.push(statement);
        match statement {
            StatementIr::Block(block) => walk(&block.statements, rows),
            StatementIr::LexicalBlock(statements) => walk(statements, rows),
            StatementIr::EmptyStatementCompletion(item) => {
                walk(std::slice::from_ref(item.statement()), rows)
            }
            StatementIr::Labelled { statement, .. } => {
                walk(std::slice::from_ref(statement.as_ref()), rows)
            }
            StatementIr::AsyncGeneratorForIn(plan) => {
                walk(&plan.head().region().block().statements, rows);
                walk(&plan.initialization().statements, rows);
                walk(&plan.body().block().statements, rows);
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                walk(&plan.head().region().block().statements, rows);
                walk(&plan.initialization().block().statements, rows);
                walk(&plan.body().block().statements, rows);
            }
            StatementIr::OrdinaryGeneratorWith(plan) => {
                walk(&plan.head().region().block().statements, rows);
                walk(&plan.body().block().statements, rows);
            }
            StatementIr::OrdinaryGeneratorLoop(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, rows);
                }
            }
            StatementIr::OrdinaryGeneratorSwitch(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, rows);
                }
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                walk(&plan.then_branch().block().statements, rows);
                walk(&plan.else_branch().block().statements, rows);
            }
            StatementIr::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                walk(&try_block.statements, rows);
                walk(&catch_block.statements, rows);
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => {
                walk(&try_block.statements, rows);
                walk(&finally_block.statements, rows);
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                walk(&try_block.statements, rows);
                walk(&catch_block.statements, rows);
                walk(&finally_block.statements, rows);
            }
            _ => {}
        }
    }
}

fn plans(function: &FunctionIr) -> Vec<&AsyncGeneratorForInIr> {
    let mut rows = Vec::new();
    walk(&function.body.statements, &mut rows);
    rows.into_iter()
        .filter_map(|statement| match statement {
            StatementIr::AsyncGeneratorForIn(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .collect()
}

#[test]
fn complete_for_in_target_and_body_retain_distinct_invocation_cells_and_source_ranges() {
    let source = "function* values(view){for(let key in choose(yield 'head-a',yield 'head-b')){let received=yield key;try{yield received;}finally{yield 'finally';}}}";
    let compiled = functions(source);
    let function = compiled
        .iter()
        .find(|function| function.name == "values")
        .unwrap();
    let plan = plans(function)[0];
    assert_eq!(plan.head_mode(), BindingMode::Let);
    assert_eq!(plan.advance_state(), plan.head().region().end_state() + 1);
    assert_eq!(
        plan.initialization_region().entry_state(),
        plan.advance_state() + 1
    );
    assert_eq!(
        plan.body().entry_state(),
        plan.initialization_region().end_state() + 1
    );
    assert_eq!(plan.continue_state(), plan.advance_state());
    assert_eq!(plan.exit_state(), plan.body().end_state() + 1);
    let bindings = [
        plan.head_binding(),
        plan.enumerator_binding(),
        plan.key_binding(),
        plan.value_binding(),
    ];
    assert_eq!(
        bindings
            .iter()
            .map(|binding| binding.slot)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    assert_eq!(
        bindings
            .iter()
            .map(|binding| binding.name.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    assert!(bindings
        .iter()
        .all(|binding| function.owned_env_bindings.contains(binding)));
    assert!(plan.head().region().block().lexical_environment.is_none());
    assert!(plan.initialization().lexical_environment.is_none());
    assert!(
        matches!(&plan.head().value().expr, ExprIr::Identifier(name) if name == &plan.head_binding().name)
    );
    let points = &function.generator_plan.as_ref().unwrap().suspension_points;
    assert_eq!(
        points.len(),
        5,
        "both target operands and all original body/finalizer Yields are consumed"
    );
    assert_eq!(
        points
            .iter()
            .filter(|point| point.suspend_state <= plan.head().region().end_state())
            .count(),
        2
    );
    assert!(points
        .iter()
        .skip(2)
        .all(|point| point.suspend_state >= plan.body().entry_state()));
}

#[test]
fn captured_const_head_keeps_the_original_iteration_environment_and_empty_declarations() {
    let compiled = functions("function* values(view){for(const key in view){const local=yield key;yield ()=>[key,local];}} ");
    let function = compiled
        .iter()
        .find(|function| function.name == "values")
        .unwrap();
    let plan = plans(function)[0];
    assert_eq!(plan.head_mode(), BindingMode::Const);
    let iteration = plan
        .lexical_environment()
        .unwrap()
        .iteration_environment
        .as_ref()
        .unwrap();
    let captured = iteration
        .bindings
        .iter()
        .find(|binding| {
            compiled.iter().any(|function| {
                function
                    .captured_bindings
                    .iter()
                    .any(|capture| capture.name == binding.name && capture.slot == binding.slot)
            })
        })
        .unwrap();
    assert_ne!(captured.name, plan.key_binding().name);
    assert!(!function
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == captured.name));
    let mut body = Vec::new();
    walk(&plan.body().block().statements, &mut body);
    assert!(
        body.iter()
            .any(|statement| matches!(statement, StatementIr::EmptyStatementCompletion(_))),
        "the actual yielded declaration has a recursive Empty completion proof"
    );
    assert!(plan.initialization().statements.iter().any(|statement| matches!(statement, StatementIr::Lexical {mode:BindingMode::Const,name,..} if name == &captured.name)));
}

#[test]
fn eager_enclosing_control_counts_for_in_phases_and_foreign_iterator_bodies_keep_their_owner() {
    let compiled = functions("function* values(view,choice){if(choice){for(let key in view){key;}}else{switch(choice){case 0:for(var key in view){key;}break;default:break;}}}");
    let function = compiled
        .iter()
        .find(|function| function.name == "values")
        .unwrap();
    assert_eq!(plans(function).len(), 2);
    assert!(function
        .generator_plan
        .as_ref()
        .unwrap()
        .suspension_points
        .is_empty());
    let iterator = functions(
        "function* values(view){for(let item of [1]){for(let key in view){key;}yield item;}}",
    );
    let function = iterator
        .iter()
        .find(|function| function.name == "values")
        .unwrap();
    let inner = plans(function);
    assert_eq!(
        inner.len(),
        1,
        "the complete ForOf body retains its actual nested ForIn phases"
    );
    let mut ordered = Vec::new();
    walk(&function.body.statements, &mut ordered);
    let iterator = ordered
        .into_iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncGeneratorForOf(plan) => Some(plan),
            _ => None,
        })
        .expect("the complete enclosing iterator owner");
    assert!(inner[0].entry_state() >= iterator.body().entry_state());
    assert!(inner[0].exit_state() <= iterator.body().end_state());
    let nested = functions("function* values(view){outer:for(let key in yield view){with(view){for(const inner in view){if(yield inner)continue outer;}}}}");
    assert_eq!(
        plans(
            nested
                .iter()
                .find(|function| function.name == "values")
                .unwrap()
        )
        .len(),
        2
    );
    let foreign = functions("function* values(view){for(let key in view){for(let item of [1]){if(item){key;}}yield key;}}");
    assert_eq!(
        plans(
            foreign
                .iter()
                .find(|function| function.name == "values")
                .unwrap()
        )
        .len(),
        1,
        "the nested complete iterator uses its own regions while the outer enumeration stays singular"
    );
}

#[test]
fn for_in_eager_head_semantics_are_shared_and_suspended_head_references_use_owned_regions() {
    for source in [
        "function* values(view){for(var key in view){yield key;}}",
        "function* values(view){let key;for(key in view){yield key;}}",
        "function* values(view,target){for(target.slot in view){yield target.slot;}}",
        "function* values(view){for(let [first,...rest] in view){yield first;}}",
        "function* values(view){for(const {length:size} in view){yield size;}}",
        "function* values(view,target){for([target.slot] in view){yield target.slot;}}",
        "class Holder{#value;*values(view){for(this.#value in view){yield this.#value;}}}",
    ] {
        let compiled = functions(source);
        assert_eq!(
            compiled
                .iter()
                .map(|function| plans(function).len())
                .sum::<usize>(),
            1,
            "{source}"
        );
    }
    for source in [
        "function* values(view,target){for(target[yield 'key'] in view){yield 1;}}",
        "function* values(view){for(let {missing=yield 1} in view){yield 2;}}",
        "function* values(view){for(const item of [1]){for(let key in view){yield key;}}}",
        "async function values(view){for(let key in view){await key;}}",
    ] {
        let program =
            lower(&parse(source, ParseOptions::script()).expect("refused ForIn source parses"));
        assert!(
            program.is_wasm_supported(),
            "complete per-key source owner: {source}: {:?}",
            program.diagnostics
        );
    }
}

#[test]
fn ordinary_for_in_owns_annex_b_prefix_and_per_key_default_in_source_order() {
    let compiled=functions("function* values(input){for(var key=yield 'prefix' in yield 'head'){yield key;}for(let [first,second,missing=yield 'default'] in input){yield missing;}}");
    let function = compiled
        .iter()
        .find(|function| function.name == "values")
        .unwrap();
    let found = plans(function);
    assert_eq!(found.len(), 2);
    assert!(found
        .iter()
        .all(|plan| plan.execution() == lila_ir::ResumableRegionProtocolIr::Generator));
    assert_eq!(
        found[0].head().region().end_state() - found[0].entry_state(),
        2
    );
    assert!(
        found[1].initialization_region().end_state()
            > found[1].initialization_region().entry_state()
    );
    let points = &function.generator_plan.as_ref().unwrap().suspension_points;
    assert_eq!(points.len(), 5);
    assert_eq!(
        points
            .iter()
            .filter(
                |point| point.suspend_state >= found[1].initialization_region().entry_state()
                    && point.suspend_state < found[1].body().entry_state()
            )
            .count(),
        1
    );
}
