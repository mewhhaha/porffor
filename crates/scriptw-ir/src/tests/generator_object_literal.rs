#[test]
fn generator_object_properties_normalize_keys_before_values_and_publish_in_source_order() {
    let program = lower_script(
        r#"
function* build(short, proto, symbol) {
  return { __proto__: proto, short, [yield 'key']: yield 'value',
    [symbol]: function () {}, get [yield 'getter']() { return 1; },
    set [yield 'setter'](value) { short = value; },
    [yield 'method']() { return super.name; },
    ...yield 'spread', final: yield 'last' };
}
"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .unwrap()
        .functions
        .iter()
        .find(|function| function.name == "build")
        .unwrap();
    assert_eq!(
        function
            .generator_plan
            .as_ref()
            .unwrap()
            .suspension_points
            .len(),
        7
    );
    let mut statements = Vec::new();
    staged_operand_statements(&function.body.statements, &mut statements);
    let yields: Vec<_> = statements
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| {
            matches!(statement, StatementIr::GeneratorYield { .. }).then_some(index)
        })
        .collect();
    let keys: Vec<_> = statements
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| match statement {
            StatementIr::Lexical { init, .. }
                if matches!(
                    init.expr,
                    ExprIr::SpecOperation {
                        operation: SpecOperationIr::ToPropertyKey,
                        ..
                    }
                ) =>
            {
                Some(index)
            }
            _ => None,
        })
        .collect();
    let definitions: Vec<_> = statements
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| match statement {
            StatementIr::Expression(value) => match &value.expr {
                ExprIr::ObjectPropertyDefinition(definition) => Some((index, definition.as_ref())),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert_eq!(keys.len(), 5);
    assert_eq!(definitions.len(), 9);
    assert!(yields[0] < keys[0] && keys[0] < yields[1]);
    assert!(yields[1] < definitions[2].0 && definitions[2].0 < yields[2]);
    assert!(
        definitions[3].0 < yields[2],
        "anonymous function naming precedes the next property"
    );
    assert!(matches!(
        definitions[0].1.property(),
        ObjectPropertyIr::PrototypeSetter { .. }
    ));
    assert!(matches!(
        definitions[1].1.property(),
        ObjectPropertyIr::Data {
            is_shorthand: true,
            ..
        }
    ));
    assert!(matches!(
        definitions[3].1.property(),
        ObjectPropertyIr::ComputedData {
            name_inference: ComputedPropertyNameInferenceIr::Function,
            ..
        }
    ));
    assert!(matches!(
        definitions[4].1.property(),
        ObjectPropertyIr::ComputedGetter { .. }
    ));
    assert!(matches!(
        definitions[5].1.property(),
        ObjectPropertyIr::ComputedSetter { .. }
    ));
    assert!(matches!(
        definitions[6].1.property(),
        ObjectPropertyIr::ComputedMethod { .. }
    ));
    assert!(matches!(
        definitions[7].1.property(),
        ObjectPropertyIr::Spread { .. }
    ));
    for (_, definition) in definitions {
        assert_eq!(
            definition.target().possible_kinds,
            KindSet::from_kind(ValueKind::Object)
        );
        assert!(
            definition.target().heap_shape.is_none(),
            "caller mutation cannot leave a construction shape proof"
        );
    }
}

#[test]
fn generator_computed_anonymous_class_consumes_the_captured_key_before_its_own_suspension() {
    let program = lower_script(
        r#"
function* build() {
  return { [yield 'name']: class { [yield 'member']() {} static label = this.name; },
    later: yield 'last' };
}
"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .unwrap()
        .functions
        .iter()
        .find(|function| function.name == "build")
        .unwrap();
    assert_eq!(
        function
            .generator_plan
            .as_ref()
            .unwrap()
            .suspension_points
            .len(),
        3
    );
    let mut statements = Vec::new();
    staged_operand_statements(&function.body.statements, &mut statements);
    let (key_index, key_name) = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::Lexical { name, init, .. }
                if matches!(
                    init.expr,
                    ExprIr::SpecOperation {
                        operation: SpecOperationIr::ToPropertyKey,
                        ..
                    }
                ) =>
            {
                Some((index, name))
            }
            _ => None,
        })
        .unwrap();
    let (class_index, class) = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::ResumableClassDefinition(class) => Some((index, class)),
            _ => None,
        })
        .unwrap();
    assert!(key_index < class_index);
    assert!(matches!(&class.class().name_inference,
        ClassNameInferenceIr::PropertyKeyBinding(name) if name == key_name));
    let definition = statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Expression(value) => match &value.expr {
                ExprIr::ObjectPropertyDefinition(definition) => Some(definition),
                _ => None,
            },
            _ => None,
        })
        .unwrap();
    assert!(
        matches!(
            definition.property(),
            ObjectPropertyIr::ComputedData {
                name_inference: ComputedPropertyNameInferenceIr::None,
                ..
            }
        ),
        "the retained prepared class must not run NamedEvaluation again"
    );
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| &binding.name == key_name));
}

#[test]
fn generator_var_initializer_with_yield_in_computed_keys_is_supported() {
    let program = lower_script(
        r#"
function* g() {
    var a = { [yield 1]: 2, [yield 2]: 3};
    return a;
}
"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
}

#[test]
fn generator_var_initializer_with_yield_survives_script_level_direct_eval() {
    let program = lower_test262_script(
        r#"
var o = eval('1');
function* g() { var a = { [yield 1]: 2, [yield 2]: 3 }; return a; }
"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
}
