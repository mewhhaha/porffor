use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ClassFieldNameIr, ClassInstanceElementIr, ClassNameInferenceIr, ClassStaticElementIr,
    ComputedPropertyNameInferenceIr, ExprIr, FunctionIr, ObjectPropertyIr,
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
    assert_eq!(
        class.name_inference,
        ClassNameInferenceIr::PropertyKeyBinding(key_binding.clone())
    );
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
    let ClassNameInferenceIr::PropertyKeyBinding(name) = &plan.class().name_inference else {
        panic!("class continuation retains its inferred-name binding");
    };
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

fn initializer_class<'a>(functions: &'a [FunctionIr], id: &str) -> &'a lila_ir::ClassDefinitionIr {
    let initializer = functions
        .iter()
        .find(|function| function.id == id)
        .expect("field initializer is a generated function");
    let [StatementIr::Return(value)] = initializer.body.statements.as_slice() else {
        panic!("field initializer has one return");
    };
    let ExprIr::ClassDefinition(class) = &value.expr else {
        panic!("anonymous initializer remains class creation");
    };
    class
}

#[test]
fn class_field_named_evaluation_owns_original_static_private_and_cached_keys() {
    let parsed = parse(
        r#"function factory(key) {
            return class Outer {
                literal = class {};
                128 = class {};
                128n = class {};
                [key] = class {};
                parenthesized = ((class {}));
                explicit = class Original {};
                #secret = class {};
                static literal = class {};
                static [key] = class {};
                static #secretStatic = class {};
            };
        }"#,
        ParseOptions::script(),
    )
    .expect("class field name fixture parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.expect("script IR");
    let factory = script
        .functions
        .iter()
        .find(|function| function.name == "factory")
        .expect("factory function");
    let outer = factory
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Return(value) => match &value.expr {
                ExprIr::ClassDefinition(class) => Some(class),
                _ => None,
            },
            _ => None,
        })
        .expect("returned class definition");
    let constructor = script
        .functions
        .iter()
        .find(|function| function.id == outer.constructor_function_id)
        .expect("outer constructor");
    let instance = constructor
        .class_instance_element_plan
        .as_ref()
        .expect("instance element plan");
    let expected = [
        ClassNameInferenceIr::FieldInitializer(ClassFieldNameIr::Static("literal".into())),
        ClassNameInferenceIr::FieldInitializer(ClassFieldNameIr::Computed(0)),
        ClassNameInferenceIr::FieldInitializer(ClassFieldNameIr::Computed(1)),
        ClassNameInferenceIr::FieldInitializer(ClassFieldNameIr::Computed(2)),
        ClassNameInferenceIr::FieldInitializer(ClassFieldNameIr::Static("parenthesized".into())),
        ClassNameInferenceIr::None,
        ClassNameInferenceIr::FieldInitializer(ClassFieldNameIr::Static("#secret".into())),
    ];
    assert_eq!(instance.elements.len(), expected.len());
    for (element, expected) in instance.elements.iter().zip(expected) {
        let ClassInstanceElementIr::Field(field) = element else {
            panic!("fixture has only fields");
        };
        let class = initializer_class(
            &script.functions,
            field
                .init_function_id
                .as_deref()
                .expect("field owns an initializer"),
        );
        assert_eq!(class.name_inference, expected);
    }
    let expected = [
        ClassNameInferenceIr::FieldInitializer(ClassFieldNameIr::Static("literal".into())),
        ClassNameInferenceIr::FieldInitializer(ClassFieldNameIr::Computed(3)),
        ClassNameInferenceIr::FieldInitializer(ClassFieldNameIr::Static("#secretStatic".into())),
    ];
    assert_eq!(outer.element_plan.static_elements.len(), expected.len());
    for (element, expected) in outer.element_plan.static_elements.iter().zip(expected) {
        let ClassStaticElementIr::Field(field) = element else {
            panic!("fixture has only static fields");
        };
        let class = initializer_class(
            &script.functions,
            field
                .init_function_id
                .as_deref()
                .expect("static field owns an initializer"),
        );
        assert_eq!(class.name_inference, expected);
    }
}
