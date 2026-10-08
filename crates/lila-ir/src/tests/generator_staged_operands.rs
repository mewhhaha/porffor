fn staged_operand_statements<'a>(statements: &'a [StatementIr], out: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        out.push(statement);
        match statement {
            StatementIr::LexicalBlock(statements) => staged_operand_statements(statements, out),
            StatementIr::Block(block) => staged_operand_statements(&block.statements, out),
            StatementIr::EmptyStatementCompletion(statement) => staged_operand_statements(std::slice::from_ref(statement.statement()), out),
            _ => {}
        }
    }
}

#[test]
fn generator_logical_selectors_consume_their_prefix_before_selected_arm_states() {
    for (expression, expected) in [
        ("(yield 'left') && (yield 'right')", 2),
        ("(yield 'left') || (yield 'right')", 2),
        ("(yield 'left') ?? (yield 'right')", 2),
        ("(yield 'left') && 3", 1),
        ("((yield 'a') && (yield 'b')) || (yield 'c')", 3),
    ] {
        let program = lower_script(&format!("function* g() {{ return {expression}; }}"));
        assert!(
            program.is_wasm_supported(),
            "{expression}: {:?}",
            program.diagnostics
        );
        let function = program
            .script
            .as_ref()
            .unwrap()
            .functions
            .iter()
            .find(|f| f.name == "g")
            .unwrap();
        let points = &function.generator_plan.as_ref().unwrap().suspension_points;
        assert_eq!(points.len(), expected);
        assert_eq!(points[0].suspend_state, 0);
        assert_eq!(points[0].resume_state, 1);
        assert_eq!(
            points
                .iter()
                .map(|p| p.resume_state)
                .collect::<BTreeSet<_>>()
                .len(),
            expected
        );
        let mut statements = Vec::new();
        staged_operand_statements(&function.body.statements, &mut statements);
        let first_yield = statements
            .iter()
            .position(|s| matches!(s, StatementIr::GeneratorYield { .. }))
            .unwrap();
        let saved = statements.iter().position(|s| matches!(s, StatementIr::Lexical { name, .. } if name.starts_with("$generator.branch.saved."))).unwrap();
        assert!(
            first_yield < saved,
            "the selector GetValue completes before its retained decision"
        );
        for statement in statements {
            if let StatementIr::OrdinaryGeneratorIf(branch) = statement {
                assert!(
                    branch.entry_state() >= points[0].resume_state,
                    "RHS selection follows the left continuation"
                );
            }
        }
    }
}

#[test]
fn generator_compound_identifier_retains_old_getvalue_and_the_original_declarative_write() {
    let program = lower_script("function* g() { let i = 1; return i += yield 'rhs'; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .unwrap()
        .functions
        .iter()
        .find(|f| f.name == "g")
        .unwrap();
    let mut statements = Vec::new();
    staged_operand_statements(&function.body.statements, &mut statements);
    let (old_index, old_name, original, captured) = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| {
            if let StatementIr::Lexical { name, init, .. } = statement {
                if name.starts_with("$generator.compound.old.") {
                    let ExprIr::EnvironmentIdentifier(identifier) = &init.expr else {
                        panic!("actual captured GetValue must be retained");
                    };
                    let EnvironmentIdentifierOperationIr::CaptureAssignmentReference { capture } =
                        &identifier.operation
                    else {
                        panic!("complete actual Reference capture");
                    };
                    let IdentifierReferenceCaptureDisposition::Located(
                        IdentifierReferenceFallbackDisposition::Declarative { binding },
                    ) = capture.disposition()
                    else {
                        panic!("actual original declarative cell");
                    };
                    let ExprIr::Identifier(original) = &binding.expr else {
                        panic!("cell binding");
                    };
                    return Some((index, name, original, capture.reference().storage_name()));
                }
            }
            None
        })
        .unwrap();
    let yield_index = statements
        .iter()
        .position(|s| matches!(s, StatementIr::GeneratorYield { .. }))
        .unwrap();
    assert!(old_index < yield_index);
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| &binding.name == old_name));
    let write = statements
        .iter()
        .find_map(|s| match s {
            StatementIr::Return(value) => Some(value),
            _ => None,
        })
        .unwrap();
    let ExprIr::EnvironmentIdentifier(identifier) = &write.expr else {
        panic!("same captured Reference PutValue");
    };
    let EnvironmentIdentifierOperationIr::PutCapturedReference { reference, value } =
        &identifier.operation
    else {
        panic!("one captured write");
    };
    assert_eq!(reference.storage_name(), captured);
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| &binding.name == original));
    let ExprIr::CoerciveAdd { lhs, .. } = &value.expr else {
        panic!("whole dynamic old value keeps ordered coercion");
    };
    assert!(matches!(&lhs.expr, ExprIr::Identifier(name) if name == old_name));
}

#[test]
fn classic_generator_heads_accept_staged_logical_tests_and_closed_compound_updates() {
    for operation in [
        "+=", "-=", "*=", "/=", "%=", "**=", "&=", "|=", "^=", "<<=", ">>=", ">>>=",
    ] {
        let program = lower_script(&format!("function* g() {{ for (let i = 1; (yield 'test') && i < 4; i {operation} yield 'step') {{ yield function () {{ return i; }}; }} }}"));
        assert!(
            program.is_wasm_supported(),
            "{operation}: {:?}",
            program.diagnostics
        );
        let function = program
            .script
            .as_ref()
            .unwrap()
            .functions
            .iter()
            .find(|f| f.name == "g")
            .unwrap();
        let plan = ordinary_classic_loop(function);
        assert!(!plan
            .lexical_environment()
            .unwrap()
            .per_iteration_slots
            .is_empty());
        assert_eq!(
            function
                .generator_plan
                .as_ref()
                .unwrap()
                .suspension_points
                .len(),
            3
        );
        assert!(plan.update().unwrap().region().block().statements.iter().any(|s| matches!(s, StatementIr::Lexical { name, .. } if name.starts_with("$generator.compound.old."))));
    }
}

#[test]
fn suspended_compound_identifiers_keep_dynamic_reference_ownership_explicit() {
    for source in ["async function* g() { let i = 1; return i += yield 1; }"] {
        let function = identifier_reference_function(source);
        let capture = captured_assignment(&function.body.statements);
        assert_eq!(capture.access(), IdentifierReferenceCaptureAccess::ReadBeforeRhs);
        assert_eq!(function.protocol, FunctionProtocolIr::AsyncGenerator);
        let points = &function.resumable_plan.as_ref().unwrap().suspension_points;
        // Async-generator Return awaits its result after the explicit Yield.
        assert_eq!(points.iter().map(|point| point.kind).collect::<Vec<_>>(),
            [ResumableSuspensionKindIr::Yield, ResumableSuspensionKindIr::Await]);
        assert_eq!(points[0].resume_state, points[1].suspend_state);
    }
}
