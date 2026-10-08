#[test]
fn registers_zero_suspension_generator_declarations() {
    let program = lower_script(
        "function* empty() {}
             function* returns() { return 1; }
             function* throws() { throw 2; }",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let generators = script
        .functions
        .iter()
        .filter(|function| function.protocol.execution_kind() == FunctionExecutionKind::Generator)
        .collect::<Vec<_>>();
    assert_eq!(generators.len(), 3);
    assert_eq!(
        generators
            .iter()
            .map(|function| function.name.as_str())
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["empty", "returns", "throws"])
    );
    for function in generators {
        assert_zero_suspension_generator(function);
    }
}

#[test]
fn lowers_zero_suspension_generator_expressions_as_function_values() {
    let program = lower_script(
        "let returns = function* named() { return 1; };
             let throws = function* () { throw 2; };",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let generator_ids = script
        .functions
        .iter()
        .filter(|function| function.protocol.execution_kind() == FunctionExecutionKind::Generator)
        .map(|function| {
            assert_zero_suspension_generator(function);
            function.id.clone()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(generator_ids.len(), 2);
    let lexical_targets = script
        .body
        .statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::Lexical { init, .. } => match &init.expr {
                ExprIr::FunctionValue(function_id) => Some(function_id.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(lexical_targets, generator_ids);
}

#[test]
fn records_explicit_inferred_and_anonymous_generator_expression_names() {
    let program = lower_script(
        "let inferred = function* () {};
             let values = [function* explicit() {}, function* () {}];",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let names = script
        .functions
        .iter()
        .filter(|function| function.protocol.execution_kind() == FunctionExecutionKind::Generator)
        .map(|function| function.name.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(names, BTreeSet::from(["", "explicit", "inferred"]));
}

#[test]
fn records_empty_names_and_exact_sources_for_unnamed_function_expressions() {
    for (syntax, protocol) in [
        ("function", FunctionProtocolIr::OrdinaryCallAndConstruct),
        ("function*", FunctionProtocolIr::Generator),
        ("async function", FunctionProtocolIr::Async),
        ("async function*", FunctionProtocolIr::AsyncGenerator),
    ] {
        let anonymous_source = format!("{syntax} () {{}}");
        let explicit_source = format!("{syntax} explicit() {{}}");
        let program = lower_script(&format!(
                "let inferred = {anonymous_source}; let values = [{explicit_source}, (0, {anonymous_source})];"
            ));
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.expect("script IR");
        let functions = script
            .functions
            .iter()
            .filter(|function| function.protocol == protocol)
            .collect::<Vec<_>>();
        assert_eq!(functions.len(), 3, "{syntax}");
        assert_eq!(
            functions
                .iter()
                .map(|function| function.name.as_str())
                .collect::<BTreeSet<_>>(),
            BTreeSet::from(["", "explicit", "inferred"]),
            "{syntax}"
        );
        for function in functions {
            let expected_source = if function.name == "explicit" {
                &explicit_source
            } else {
                &anonymous_source
            };
            assert_eq!(
                function.to_string_representation,
                CallableToStringRepresentation::ExactSource(expected_source.clone())
            );
            assert_eq!(function.is_named_expression, function.name == "explicit");
        }
    }
}

#[test]
fn lowers_arrow_function_from_generator_object_parameter_default() {
    let program = lower_script("let f = function* ({ arrow = () => 1 }) {};");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    assert!(script
        .functions
        .iter()
        .any(
            |function| function.protocol.flavor() == FunctionFlavor::Arrow
                && function.name == "arrow"
        ));
}

#[test]
fn lowers_function_expression_from_nested_generator_object_parameter_default() {
    let program = lower_script(
        "let f = function* ({ nested: { ordinary = function () { return 1; } } }) {};",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    assert!(script.functions.iter().any(|function| {
        function.protocol.execution_kind() == FunctionExecutionKind::Ordinary
            && function.protocol.flavor() == FunctionFlavor::Ordinary
            && function.is_expression
            && function.name == "ordinary"
    }));
}

#[test]
fn lowers_generator_function_instanceof_without_requiring_constructability() {
    let program =
        lower_script("let generator = function* () {}; generator() instanceof generator;");
    assert!(program.is_wasm_supported());
    assert!(program.ir_summary().contains("instanceofs=1"));
}

#[test]
fn records_generator_default_parameter_tdz_and_prior_binding_reads() {
    let program = lower_script(
        "let selfRead = function* (value = value) {};
             let laterRead = function* (value = later, later) {};
             let priorRead = function* (value = 1, later = value) {};",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");

    for function_name in ["selfRead", "laterRead"] {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == function_name)
            .unwrap_or_else(|| panic!("missing `{function_name}`"));
        assert!(matches!(
            function.params[0]
                .default_init
                .as_ref()
                .map(|init| &init.expr),
            Some(ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                ..
            })
        ));
    }

    let prior_read = script
        .functions
        .iter()
        .find(|function| function.name == "priorRead")
        .expect("missing `priorRead`");
    assert!(matches!(
        prior_read.params[1]
            .default_init
            .as_ref()
            .map(|init| &init.expr),
        Some(ExprIr::Identifier(name)) if name == "value"
    ));
}

#[test]
fn records_zero_suspension_object_and_class_generator_methods() {
    let program = lower_script(
        "const object = {
                 *returns() { return 1; },
                 *throws() { throw 2; }
             };
             class Example {
                 *empty() {}
                 static *returns() { return 1; }
             }",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let generators = script
        .functions
        .iter()
        .filter(|function| function.protocol.execution_kind() == FunctionExecutionKind::Generator)
        .collect::<Vec<_>>();
    assert_eq!(generators.len(), 4);
    for function in generators {
        assert_zero_suspension_generator(function);
    }
}

#[test]
fn records_linear_generator_suspension_edges() {
    let program = lower_script("function* sequence() { yield 1; yield 2; return yield 3; }");
    assert!(program.is_wasm_supported());
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "sequence")
        .expect("generator should be registered");
    assert_eq!(
        function.protocol.execution_kind(),
        FunctionExecutionKind::Generator
    );
    assert_eq!(
        function.generator_plan,
        Some(GeneratorPlanIr {
            entry_state: 0,
            state_count: 4,
            suspension_points: vec![
                GeneratorSuspensionPointIr {
                    suspend_state: 0,
                    resume_state: 1,
                },
                GeneratorSuspensionPointIr {
                    suspend_state: 1,
                    resume_state: 2,
                },
                GeneratorSuspensionPointIr {
                    suspend_state: 2,
                    resume_state: 3,
                },
            ],
        })
    );
    assert_eq!(
        function
            .body
            .statements
            .iter()
            .filter(|statement| matches!(statement, StatementIr::GeneratorYield { .. }))
            .count(),
        3
    );
}

#[test]
fn admits_generator_loops_with_owned_break_and_continue_control() {
    for source in [
        "function* sequence() { for (let i = 0; i < 1; i++) { yield i; break; } }",
        "function* sequence() { while (true) { yield 1; continue; } }",
    ] {
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "loop control must enter the checked classic plan: {source}: {:?}",
            program.diagnostics
        );
    }
}

#[test]
fn admits_generator_loops_with_captured_per_iteration_bindings() {
    let uncaptured =
        lower_script("function* sequence() { for (let i = 0; i < 2; i++) { yield i; } }");
    assert!(
        uncaptured.is_wasm_supported(),
        "{:?}",
        uncaptured.diagnostics
    );

    let program = lower_script(
        "function* sequence() {
                 for (let i = 0; i < 2; i++) {
                     yield function () { return i; };
                 }
             }",
    );

    assert!(
        program.is_wasm_supported(),
        "captured for-let bindings retain a resumable per-iteration environment: {:?}",
        program.diagnostics
    );
}

#[test]
fn records_nested_yield_as_two_linear_suspensions() {
    let program = lower_script("function* sequence() { yield yield 1; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "sequence")
        .expect("generator should be registered");
    assert_eq!(
        function.generator_plan,
        Some(GeneratorPlanIr {
            entry_state: 0,
            state_count: 3,
            suspension_points: vec![
                GeneratorSuspensionPointIr {
                    suspend_state: 0,
                    resume_state: 1,
                },
                GeneratorSuspensionPointIr {
                    suspend_state: 1,
                    resume_state: 2,
                },
            ],
        })
    );
    assert!(matches!(
        function.body.statements.as_slice(),
        [StatementIr::LexicalBlock(statements)]
            if matches!(
                statements.as_slice(),
                [
                    StatementIr::Lexical { .. },
                    StatementIr::GeneratorYield { .. },
                    StatementIr::GeneratorYield { .. }
                ]
            )
    ));
}

#[test]
fn records_regexp_yield_assignment_resume_mode() {
    let program = lower_script(
        "let received;
             function* sequence() { received = yield/abc/i; }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "sequence")
        .expect("generator should be registered");
    let [StatementIr::LexicalBlock(statements)] = function.body.statements.as_slice() else {
        panic!("expected the captured assignment continuation");
    };
    let [StatementIr::Lexical {
        name: reference_slot,
        ..
    }, StatementIr::Expression(capture), StatementIr::Lexical {
        name: resumed_value,
        ..
    }, StatementIr::GeneratorYield {
        value, resume_mode, ..
    }, StatementIr::Expression(write)] = statements.as_slice()
    else {
        panic!("expected Reference capture before RegExp Yield and PutValue after resume");
    };
    let ExprIr::EnvironmentIdentifier(identifier) = &capture.expr else {
        panic!("expected actual Identifier Reference capture");
    };
    assert_eq!(identifier.name, "received");
    assert_eq!(identifier.strictness, Strictness::Sloppy);
    let EnvironmentIdentifierOperationIr::CaptureAssignmentReference { capture } =
        &identifier.operation
    else {
        panic!("expected write-only capture before the RHS");
    };
    assert_eq!(
        capture.access(),
        IdentifierReferenceCaptureAccess::WriteOnly
    );
    assert_eq!(capture.reference().storage_name(), reference_slot);
    assert!(matches!(
        &value.expr,
        ExprIr::RegExpLiteral { source, flags, .. } if source == "abc" && flags == "i"
    ));
    assert!(matches!(resume_mode,
        GeneratorResumeModeIr::AssignIdentifier(name) if name == resumed_value));
    let ExprIr::EnvironmentIdentifier(identifier) = &write.expr else {
        panic!("expected PutValue through the retained Reference");
    };
    assert_eq!(identifier.name, "received");
    assert_eq!(identifier.strictness, Strictness::Sloppy);
    let EnvironmentIdentifierOperationIr::PutCapturedReference { reference, value } =
        &identifier.operation
    else {
        panic!("expected the captured assignment's PutValue");
    };
    assert_eq!(reference, capture.reference());
    assert!(matches!(&value.expr, ExprIr::Identifier(name) if name == resumed_value));
}

#[test]
fn generator_property_resume_carries_one_ordinary_reference_and_its_strictness() {
    let program = lower_script(
        "function* sloppy(object, key) { object[key] = yield 1; }
             function* strict(object, key) { 'use strict'; object[key] = yield 1; }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");

    for (name, expected_strictness) in [
        ("sloppy", Strictness::Sloppy),
        ("strict", Strictness::Strict),
    ] {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == name)
            .unwrap_or_else(|| panic!("generator `{name}` should be registered"));
        let resume_mode = function
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::GeneratorYield { resume_mode, .. } => Some(resume_mode),
                StatementIr::Empty
                | StatementIr::ParameterInitialization { .. }
                | StatementIr::Expression(_) => None,
                other => panic!("unexpected generator setup statement: {other:?}"),
            })
            .expect("expected one generator yield after parameter setup");
        let GeneratorResumeModeIr::AssignProperty(reference) = resume_mode else {
            panic!("expected a suspended property Reference");
        };
        match reference.use_view() {
            SuspendedPropertyReferenceUse::Ordinary {
                base_and_receiver,
                key,
                strictness,
            } => {
                assert!(matches!(&base_and_receiver.expr, ExprIr::Identifier(_)));
                assert!(matches!(key, PropertyKeyIr::StringExpr(_)));
                assert_eq!(strictness, expected_strictness);
            }
        }
    }
}

#[test]
fn generator_nullish_property_resume_retains_raw_reference_and_rhs() {
    let program = lower_script(
        "function* computedNull() { null[{ toString() { return 'value'; } }] = yield 1; }
             function* namedUndefined() { undefined.value = yield 2; }
             async function* computedUndefined() {
               undefined[{ toString() { return 'value'; } }] = yield 3;
             }
             async function* namedNull() { 'use strict'; null.value = yield 4; }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    for (name, base_kind, computed, expected_rhs, expected_strictness) in [
        (
            "computedNull",
            ValueKind::Null,
            true,
            1.0_f64,
            Strictness::Sloppy,
        ),
        (
            "namedUndefined",
            ValueKind::Undefined,
            false,
            2.0,
            Strictness::Sloppy,
        ),
        (
            "computedUndefined",
            ValueKind::Undefined,
            true,
            3.0,
            Strictness::Sloppy,
        ),
        ("namedNull", ValueKind::Null, false, 4.0, Strictness::Strict),
    ] {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == name)
            .unwrap_or_else(|| panic!("generator `{name}` should be registered"));
        let mut statements = Vec::new();
        staged_operand_statements(&function.body.statements, &mut statements);
        let (yield_position, value, resume_mode) = statements
            .iter().enumerate()
            .find_map(|(position, statement)| match statement {
                StatementIr::GeneratorYield { value, resume_mode, .. } => Some((position, value, resume_mode)),
                _ => None,
            })
            .expect("nullish Reference must retain its suspended RHS");
        assert!(matches!(value.expr, ExprIr::Number(number) if number == expected_rhs.to_bits()));
        let (base_and_receiver, key, strictness) = match resume_mode {
            GeneratorResumeModeIr::AssignProperty(reference) => match reference.use_view() {
                SuspendedPropertyReferenceUse::Ordinary { base_and_receiver, key, strictness } => (base_and_receiver, key, strictness),
            },
            GeneratorResumeModeIr::AssignIdentifier(resumed) => {
                let assignment = statements[yield_position + 1..].iter().find_map(|statement| match statement {
                    StatementIr::Expression(TypedExpr { expr: ExprIr::OrdinaryPropertyAssignment(assignment), .. }) => Some(assignment),
                    _ => None,
                }).expect("mixed continuation performs the original raw Reference Put after resume");
                assert!(matches!(&assignment.rhs().expr, ExprIr::Identifier(name) if name == resumed));
                (assignment.base_and_receiver(), assignment.referenced_name(), assignment.strictness())
            }
            _ => panic!("assignment must consume its resumed whole value"),
        };
        assert_eq!(base_and_receiver.kind, base_kind);
        assert_eq!(strictness, expected_strictness);
        if computed {
            assert!(matches!(key, PropertyKeyIr::StringExpr(raw) if raw.kind == ValueKind::Object));
        } else {
            assert!(matches!(key, PropertyKeyIr::StaticString(name) if name == "value"));
        }
    }
}

#[test]
fn records_discarded_generator_expression_suspensions() {
    let program = lower_script(
        "function* grouping() { (yield 1); }
             function* array() { [yield 1]; }
             function* block() { { yield 1; } }
             function* comma() { yield 1, yield 2; }
             function* add() { (yield 1) + (yield 2); }
             function* conditional() { (yield 1) ? yield 2 : yield 3; }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    for (name, state_count, suspension_count) in [
        ("grouping", 2, 1),
        ("array", 2, 1),
        ("block", 2, 1),
        ("comma", 3, 2),
        ("add", 3, 2),
        ("conditional", 7, 3),
    ] {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == name)
            .unwrap_or_else(|| panic!("generator `{name}` should be registered"));
        let plan = function
            .generator_plan
            .as_ref()
            .unwrap_or_else(|| panic!("generator `{name}` should have a suspension plan"));
        assert_eq!(plan.state_count, state_count, "generator `{name}`");
        assert_eq!(
            plan.suspension_points.len(),
            suspension_count,
            "generator `{name}`"
        );
    }
    let conditional = script
        .functions
        .iter()
        .find(|function| function.name == "conditional")
        .expect("conditional generator should be registered");
    assert!(matches!(
        conditional.body.statements.as_slice(),
        [StatementIr::LexicalBlock(statements)]
            if matches!(
                statements.as_slice(),
                [
                    StatementIr::Lexical { .. },
                    StatementIr::GeneratorYield { .. },
                    StatementIr::Lexical { .. },
                    StatementIr::Lexical { .. },
                    StatementIr::OrdinaryGeneratorIf(_),
                    StatementIr::Expression(_)
                ]
            )
    ));
    let add = script
        .functions
        .iter()
        .find(|function| function.name == "add")
        .expect("add generator should be registered");
    assert!(matches!(
        add.body.statements.as_slice(),
        [StatementIr::LexicalBlock(statements)]
            if matches!(
                statements.as_slice(),
                [
                    StatementIr::Lexical { .. },
                    StatementIr::GeneratorYield { .. },
                    StatementIr::Lexical { .. },
                    StatementIr::Lexical { .. },
                    StatementIr::GeneratorYield { .. },
                    StatementIr::Lexical { .. },
                    StatementIr::Expression(TypedExpr {
                        expr: ExprIr::CoerciveAdd { .. },
                        ..
                    })
                ]
            )
    ));
}

#[test]
fn records_generator_template_interpolation_suspensions() {
    let program = lower_script(
        "let output;
             function before() { return 2; }
             function* sequence() {
                 output = `1${before()}3${yield 4}5${yield 6}7`;
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "sequence")
        .expect("generator should be registered");
    assert_eq!(
        function.generator_plan,
        Some(GeneratorPlanIr {
            entry_state: 0,
            state_count: 3,
            suspension_points: vec![
                GeneratorSuspensionPointIr {
                    suspend_state: 0,
                    resume_state: 1,
                },
                GeneratorSuspensionPointIr {
                    suspend_state: 1,
                    resume_state: 2,
                },
            ],
        })
    );
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name.starts_with("$generator.template.value.")));
    assert!(matches!(
        function.body.statements.as_slice(),
        [StatementIr::LexicalBlock(statements)]
            if statements.iter().filter(|statement| matches!(statement, StatementIr::GeneratorYield { .. })).count() == 2
    ));
}

#[test]
fn records_generator_suspensions_inside_with() {
    let program = lower_script(
        "function* sequence() {
                 let x = 1;
                 yield x;
                 with ({ x: 2 }) {
                     yield x;
                     x = 3;
                     yield x;
                 }
                 yield x;
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .expect("script ir should exist")
        .functions
        .iter()
        .find(|function| function.name == "sequence")
        .expect("generator should be registered");
    let plan = function
        .generator_plan
        .as_ref()
        .expect("generator should have a suspension plan");
    assert_eq!(plan.state_count, 7);
    assert_eq!(plan.suspension_points.iter().map(|point| (point.suspend_state, point.resume_state)).collect::<Vec<_>>(),
        [(0, 1), (2, 3), (3, 4), (5, 6)]);
    let with = function.body.statements.iter().find_map(|statement| match statement {
        StatementIr::OrdinaryGeneratorWith(plan) => Some(plan),
        _ => None,
    }).expect("one original With environment lifetime");
    assert_eq!((with.entry_state(), with.body().entry_state(), with.body().end_state(), with.exit_state()), (1, 2, 4, 5));
    assert!(function.owned_env_bindings.contains(with.head_binding()));
    assert!(with.lexical_environment().bindings.contains(with.object_binding()));
    assert_ne!(with.head_binding().name, with.object_binding().name);
}
