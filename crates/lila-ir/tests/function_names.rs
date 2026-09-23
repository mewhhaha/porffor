use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ComputedPropertyNameInferenceIr, ExprIr, FunctionIr, ObjectPropertyIr,
    ResumableClassDefinitionIr, SpecOperationIr, StatementIr,
};

fn lower_factory(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("function name fixture parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .expect("script IR")
        .functions
        .into_iter()
        .find(|function| function.name == "factory")
        .expect("factory function")
}

#[test]
fn computed_names_distinguish_anonymous_functions_classes_and_ordinary_values() {
    let function = lower_factory(
        "function factory(key, existing) { return { [key]: function () {}, \
         ['class']: class {}, [key]: function named() {}, [key]: (0, function () {}), \
         [key]: existing }; }",
    );
    let value = function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Return(value) => Some(value),
            _ => None,
        })
        .expect("factory return");
    let ExprIr::ObjectLiteral(properties) = &value.expr else {
        panic!("object literal must retain its property definitions");
    };
    assert_eq!(properties.len(), 5);
    assert!(matches!(
        properties[0],
        ObjectPropertyIr::ComputedData {
            name_inference: ComputedPropertyNameInferenceIr::Function,
            ..
        }
    ));
    let ObjectPropertyIr::ComputedData {
        name_inference: ComputedPropertyNameInferenceIr::Class { key_binding },
        value,
        ..
    } = &properties[1]
    else {
        panic!("anonymous class must retain its property key binding");
    };
    let ExprIr::ClassDefinition(class) = &value.expr else {
        panic!("anonymous class remains a class evaluation");
    };
    assert_eq!(class.inferred_name_binding.as_ref(), Some(key_binding));
    assert!(class.name_binding.is_none());
    for property in &properties[2..] {
        assert!(matches!(
            property,
            ObjectPropertyIr::ComputedData {
                name_inference: ComputedPropertyNameInferenceIr::None,
                ..
            }
        ));
    }
}

fn class_plan<'a>(
    statements: &'a [StatementIr],
    initializers: &mut Vec<(&'a str, &'a ExprIr)>,
) -> Option<&'a ResumableClassDefinitionIr> {
    for statement in statements {
        match statement {
            StatementIr::Lexical { name, init, .. } => {
                initializers.push((name.as_str(), &init.expr));
            }
            StatementIr::ResumableClassDefinition(plan) => return Some(plan),
            StatementIr::LexicalBlock(statements) => {
                if let Some(plan) = class_plan(statements, initializers) {
                    return Some(plan);
                }
            }
            StatementIr::Block(block) => {
                if let Some(plan) = class_plan(&block.statements, initializers) {
                    return Some(plan);
                }
            }
            _ => {}
        }
    }
    None
}

#[test]
fn suspended_class_name_key_is_normalized_before_heritage_and_owned_by_activation() {
    let function = lower_factory(
        "async function factory(key, Base) { return { [key]: class extends (await Base) {} }; }",
    );
    let mut initializers = Vec::new();
    let plan = class_plan(&function.body.statements, &mut initializers)
        .expect("awaited heritage owns a class continuation");
    let name = plan
        .class()
        .inferred_name_binding
        .as_ref()
        .expect("class continuation retains its inferred-name binding");
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| &binding.name == name));
    assert!(initializers.iter().any(|(binding, expression)| {
        *binding == name.as_str()
            && matches!(
                expression,
                ExprIr::SpecOperation { operation: SpecOperationIr::ToPropertyKey, operands }
                    if operands.len() == 1
            )
    }));
    assert_eq!((plan.entry_state(), plan.exit_state()), (0, 1));
}
