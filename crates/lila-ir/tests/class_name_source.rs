use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ClassDefinitionIr, ClassElementDefinitionIr, ExprIr, FunctionIr,
    ResumableClassDefinitionIr, ScriptIr, StatementIr,
};

fn lower_script(source: &str) -> ScriptIr {
    let parsed = parse(source, ParseOptions::script()).expect("class names parse");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program.script.expect("script IR")
}

fn collect_classes<'a>(
    statements: &'a [StatementIr],
    classes: &mut Vec<(&'a str, &'a ClassDefinitionIr)>,
) {
    for statement in statements {
        match statement {
            StatementIr::Lexical { name, init, .. } => {
                if let ExprIr::ClassDefinition(class) = &init.expr {
                    classes.push((name, class));
                }
            }
            StatementIr::Var(declarations) => {
                for declaration in declarations {
                    if let Some(init) = &declaration.init {
                        if let ExprIr::ClassDefinition(class) = &init.expr {
                            classes.push((&declaration.name, class));
                        }
                    }
                }
            }
            StatementIr::LexicalBlock(statements) => collect_classes(statements, classes),
            StatementIr::Block(block) => collect_classes(&block.statements, classes),
            StatementIr::ModuleUnitOnce { block, .. } => {
                collect_classes(&block.statements, classes)
            }
            _ => {}
        }
    }
}

fn collect_plans<'a>(
    statements: &'a [StatementIr],
    plans: &mut Vec<&'a ResumableClassDefinitionIr>,
) {
    for statement in statements {
        match statement {
            StatementIr::ResumableClassDefinition(plan) => {
                plans.push(plan);
                for prefix in plan.prefixes() {
                    collect_plans(prefix.statements(), plans);
                }
            }
            StatementIr::LexicalBlock(statements) => collect_plans(statements, plans),
            StatementIr::Block(block) => collect_plans(&block.statements, plans),
            _ => {}
        }
    }
}

fn read_method<'a>(class: &ClassDefinitionIr, functions: &'a [FunctionIr]) -> &'a FunctionIr {
    let method_id = class
        .element_plan
        .definitions
        .iter()
        .find_map(|definition| match definition {
            ClassElementDefinitionIr::PublicMethod(method) => Some(&method.function_id),
            _ => None,
        })
        .expect("one actual class method");
    functions
        .iter()
        .find(|function| &function.id == method_id)
        .expect("method function")
}

#[test]
fn inferred_class_labels_capture_outer_cells_and_explicit_names_capture_inner_cells() {
    let script = lower_script(
        "function classes() { let Inferred = class { static read() { return Inferred; } }; let Explicit = class Inner { static read() { return Inner; } }; class Declared { static read() { return Declared; } } return [Inferred, Explicit, Declared]; }",
    );
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "classes")
        .expect("class creation function");
    let mut classes = Vec::new();
    collect_classes(&function.body.statements, &mut classes);
    assert_eq!(classes.len(), 3);
    let (outer_storage, inferred) = classes[0];
    assert_eq!(inferred.name.as_deref(), Some("Inferred"));
    assert!(inferred.name_binding.is_none());
    assert!(read_method(inferred, &script.functions)
        .captured_bindings
        .iter()
        .any(|binding| binding.source_name == "Inferred" && binding.name == outer_storage));
    for (_, class) in &classes[1..] {
        let inner = class.name_binding.as_ref().expect("source class name cell");
        assert!(read_method(class, &script.functions)
            .captured_bindings
            .iter()
            .any(|binding| binding.name == inner.storage_name));
    }
    assert_eq!(classes[1].1.name.as_deref(), Some("Inner"));
    assert_eq!(classes[2].1.name.as_deref(), Some("Declared"));
}

#[test]
fn yielded_heritage_retains_only_actual_source_class_name_environments() {
    let script = lower_script(
        "function* classes() { let Inferred = class extends (yield Object) { static read() { return Inferred; } }; let Explicit = class Inner extends (yield Object) { static read() { return Inner; } }; return [Inferred, Explicit]; }",
    );
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "classes")
        .expect("class generator");
    let mut plans = Vec::new();
    collect_plans(&function.body.statements, &mut plans);
    assert_eq!(plans.len(), 2);
    assert_eq!(plans[0].class().name.as_deref(), Some("Inferred"));
    assert!(plans[0].class().name_binding.is_none());
    assert!(plans[0].name_environment_binding().is_none());
    assert_eq!(plans[1].class().name.as_deref(), Some("Inner"));
    assert!(plans[1].class().name_binding.is_some());
    let retained = plans[1]
        .name_environment_binding()
        .expect("explicit name environment survives heritage suspension");
    assert_eq!(
        function
            .owned_env_bindings
            .iter()
            .filter(|binding| binding.name == retained)
            .count(),
        1
    );
    assert_eq!(plans[0].exit_state(), plans[1].entry_state());
}

#[test]
fn anonymous_default_export_preserves_its_display_label_without_a_class_name_cell() {
    let parsed = parse(
        "export default class { static label = this.name; }",
        ParseOptions::module(),
    )
    .expect("anonymous default class parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.expect("module script IR");
    let mut classes = Vec::new();
    collect_classes(&script.body.statements, &mut classes);
    for function in &script.functions {
        collect_classes(&function.body.statements, &mut classes);
    }
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].1.name.as_deref(), Some("default"));
    assert!(classes[0].1.name_binding.is_none());
}
