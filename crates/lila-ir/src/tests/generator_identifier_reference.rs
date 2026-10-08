fn identifier_reference_function(source: &str) -> FunctionIr {
    let program = lower_script(source);
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
        .find(|f| f.name == "g")
        .unwrap()
}

fn identifier_reference_rows<'a>(statements: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        output.push(statement);
        match statement {
            StatementIr::Block(block) => identifier_reference_rows(&block.statements, output),
            StatementIr::LexicalBlock(statements) => identifier_reference_rows(statements, output),
            StatementIr::EmptyStatementCompletion(item) => identifier_reference_rows(std::slice::from_ref(item.statement()), output),
            StatementIr::OrdinaryGeneratorWith(plan) => {
                identifier_reference_rows(&plan.head().region().block().statements, output);
                identifier_reference_rows(&plan.body().block().statements, output);
            }
            StatementIr::AsyncFunctionWith(plan) => {
                identifier_reference_rows(&plan.head().statements, output);
                identifier_reference_rows(&plan.body().statements, output);
            }
            StatementIr::AsyncGeneratorWith(plan) => {
                identifier_reference_rows(&plan.head().region().block().statements, output);
                identifier_reference_rows(&plan.body().block().statements, output);
            }
            StatementIr::OrdinaryGeneratorLoop(plan) => {
                for region in plan.regions() { identifier_reference_rows(&region.block().statements, output); }
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                for region in plan.regions() { identifier_reference_rows(&region.block().statements, output); }
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                identifier_reference_rows(&plan.head().region().block().statements, output);
                identifier_reference_rows(&plan.initialization().block().statements, output);
                identifier_reference_rows(&plan.body().block().statements, output);
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                identifier_reference_rows(&plan.then_branch().block().statements, output);
                identifier_reference_rows(&plan.else_branch().block().statements, output);
            }
            _ => {}
        }
    }
}

fn captured_assignment(statements: &[StatementIr]) -> &IdentifierReferenceCaptureIr {
    let mut flattened = Vec::new();
    identifier_reference_rows(statements, &mut flattened);
    flattened
        .into_iter()
        .find_map(|statement| {
            let init = match statement {
                StatementIr::Lexical { init, .. } | StatementIr::Expression(init) => init,
                _ => return None,
            };
            let ExprIr::EnvironmentIdentifier(identifier) = &init.expr else {
                return None;
            };
            let EnvironmentIdentifierOperationIr::CaptureAssignmentReference { capture } =
                &identifier.operation
            else {
                return None;
            };
            Some(capture)
        })
        .expect("actual Reference capture precedes suspension")
}

#[test]
fn suspended_identifier_assignment_captures_global_with_and_runtime_records_once() {
    for (source, expected) in [
        ("function* g() { return missing += yield 1; }", "global"),
        (
            "var value = 1; function* g() { return value += yield 1; }",
            "global",
        ),
        (
            "function* g(object) { let value = 1; with (object) { value += yield 1; } }",
            "with",
        ),
        (
            "function* g() { let value = 1; eval(''); return value += yield 1; }",
            "runtime",
        ),
    ] {
        let function = identifier_reference_function(source);
        let capture = captured_assignment(&function.body.statements);
        assert_eq!(
            capture.access(),
            IdentifierReferenceCaptureAccess::ReadBeforeRhs
        );
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == capture.reference().storage_name()));
        match (expected, capture.disposition()) {
            (
                "global",
                IdentifierReferenceCaptureDisposition::Located(
                    IdentifierReferenceFallbackDisposition::Global,
                ),
            ) => {}
            (
                "with",
                IdentifierReferenceCaptureDisposition::WithObject {
                    selection,
                    fallback,
                },
            ) => {
                assert!(matches!(
                    fallback,
                    IdentifierReferenceFallbackDisposition::Declarative { .. }
                ));
                assert!(matches!(selection.expr, ExprIr::Conditional { .. }));
            }
            ("runtime", IdentifierReferenceCaptureDisposition::RuntimeEnvironment) => {}
            _ => panic!("wrong actual located capture: {source}"),
        }
    }
}

#[test]
fn generator_logical_assignments_own_the_complete_selected_rhs_and_release_skipped_references() {
    for operation in ["&&=", "||=", "??="] {
        let function = identifier_reference_function(&format!(
            "function* g() {{ let value = 1; return value {operation} f(yield 'a', yield 'b'); }}"
        ));
        let plan = function.generator_plan.as_ref().unwrap();
        assert_eq!(plan.suspension_points.len(), 2);
        let mut flattened = Vec::new();
        identifier_reference_rows(&function.body.statements, &mut flattened);
        let branch = flattened
            .into_iter()
            .find_map(|statement| match statement {
                StatementIr::OrdinaryGeneratorIf(branch) => Some(branch),
                _ => None,
            })
            .expect("complete selected RHS region");
        assert_eq!(branch.then_branch().entry_state(), branch.entry_state() + 1);
        assert!(branch.then_branch().end_state() > branch.then_branch().entry_state());
        assert_eq!(
            branch.else_branch().entry_state(),
            branch.else_branch().end_state()
        );
        assert!(plan
            .suspension_points
            .iter()
            .all(
                |point| point.suspend_state >= branch.then_branch().entry_state()
                    && point.resume_state <= branch.then_branch().end_state()
            ));
        let capture = captured_assignment(&function.body.statements);
        assert_eq!(
            capture.access(),
            IdentifierReferenceCaptureAccess::ReadBeforeRhs
        );
        let skipped = branch.else_branch().block().statements.last().unwrap();
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::EnvironmentIdentifier(identifier),
            ..
        }) = skipped
        else {
            panic!("one actual Reference release in skipped branch");
        };
        let EnvironmentIdentifierOperationIr::ReleaseCapturedReference { reference } =
            &identifier.operation
        else {
            panic!("skipped branch never writes");
        };
        assert_eq!(reference, capture.reference());
        assert!(
            crate::reference::carried_put_value_failure(&ExprIr::EnvironmentIdentifier(
                identifier.clone()
            ))
            .is_none()
        );
    }
}

#[test]
fn plain_generator_assignment_retains_reference_without_get_value_before_the_complete_rhs() {
    for (source, expected, yields) in [
        ("function* g() { missing = yield 1; }", "global", 1),
        ("function* g() { return missing = yield 1; }", "global", 1),
        ("function* g(object) { with (object) { value = yield 1; } }", "with-global", 1),
        ("function* g(object) { let value = 0; with (object) { return value = f(yield 1, yield 2); } }", "with-cell", 2),
        ("function* g() { value = yield 1; let value; }", "cell", 1),
        ("function* g() { const value = 0; return value = yield 1; }", "cell", 1),
        ("function* g() { let value; value = `A${yield 1}${yield 2}`; }", "cell", 2),
        ("function* g() { let value = 0; eval(''); return value = yield 1; }", "runtime", 1),
        ("function* g() { let value = 0; for (let i = 0; i < 1; i++) { value = yield 1; } }", "cell", 1),
    ] {
        let function = identifier_reference_function(source);
        let capture = captured_assignment(&function.body.statements);
        assert_eq!(capture.access(), IdentifierReferenceCaptureAccess::WriteOnly);
        assert!(function.owned_env_bindings.iter().any(|binding|
            binding.name == capture.reference().storage_name()));
        assert!(!function.owned_env_bindings.iter().any(|binding|
            binding.name.contains("generator.compound.old.")));
        assert_eq!(function.generator_plan.as_ref().unwrap().suspension_points.len(), yields);
        match (expected, capture.disposition()) {
            ("global", IdentifierReferenceCaptureDisposition::Located(
                IdentifierReferenceFallbackDisposition::Global)) => {},
            ("cell", IdentifierReferenceCaptureDisposition::Located(
                IdentifierReferenceFallbackDisposition::Declarative { .. })) => {},
            ("with-global", IdentifierReferenceCaptureDisposition::WithObject {
                fallback: IdentifierReferenceFallbackDisposition::Global, .. }) => {},
            ("with-cell", IdentifierReferenceCaptureDisposition::WithObject {
                fallback: IdentifierReferenceFallbackDisposition::Declarative { .. }, .. }) => {},
            ("runtime", IdentifierReferenceCaptureDisposition::RuntimeEnvironment) => {},
            _ => panic!("wrong actual write-only Reference: {source}"),
        }
    }
}

#[test]
fn plain_assignment_put_consumes_the_same_owned_reference_after_suspension() {
    for source in [
        "function* g() { return missing = yield 1; }",
        "function* g() { let value; return value = f(yield 1, yield 2); }",
        "function* g(object) { with (object) { return value = yield 1; } }",
    ] {
        let function = identifier_reference_function(source);
        let capture = captured_assignment(&function.body.statements);
        let mut statements = Vec::new();
        identifier_reference_rows(&function.body.statements, &mut statements);
        let capture_index = statements.iter().position(|statement| {
            matches!(statement, StatementIr::Expression(TypedExpr {
                expr: ExprIr::EnvironmentIdentifier(identifier), ..
            }) if matches!(&identifier.operation, EnvironmentIdentifierOperationIr::CaptureAssignmentReference { .. }))
        }).unwrap();
        let yield_index = statements
            .iter()
            .position(|statement| matches!(statement, StatementIr::GeneratorYield { .. }))
            .unwrap();
        let (put_index, reference) = statements
            .iter()
            .enumerate()
            .find_map(|(index, statement)| {
                let StatementIr::Return(TypedExpr {
                    expr: ExprIr::EnvironmentIdentifier(identifier),
                    ..
                }) = statement
                else {
                    return None;
                };
                let EnvironmentIdentifierOperationIr::PutCapturedReference { reference, .. } =
                    &identifier.operation
                else {
                    return None;
                };
                Some((index, reference))
            })
            .unwrap();
        assert!(capture_index < yield_index && yield_index < put_index);
        assert_eq!(reference, capture.reference());
    }
}
