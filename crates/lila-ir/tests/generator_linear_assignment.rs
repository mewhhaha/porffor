use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, EnvironmentIdentifierOperationIr, ExprIr, GeneratorResumeModeIr,
    IdentifierReferenceCaptureAccess, IdentifierReferenceCaptureDisposition,
    IdentifierReferenceFallbackDisposition, ResumableRegionProtocolIr, StatementIr, TypedExpr,
};

fn flatten<'a>(source: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for statement in source {
        match statement {
            StatementIr::LexicalBlock(body) => flatten(body, output),
            StatementIr::Block(body) => flatten(&body.statements, output),
            StatementIr::EmptyStatementCompletion(item) => {
                flatten(std::slice::from_ref(item.statement()), output)
            }
            statement => output.push(statement),
        }
    }
}

#[test]
fn iterator_owned_assignment_preserves_its_two_received_operands_before_writing() {
    let source = "function* values(source, combine) { let value = 0; for (const item of source) { value = combine(yield 'left', yield 'right'); } return value; }";
    let parsed =
        parse(source, ParseOptions::script()).expect("retained iterator assignment parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .expect("script")
        .functions
        .into_iter()
        .find(|function| function.name == "values")
        .expect("actual generator");
    let mut owner_statements = Vec::new();
    flatten(&function.body.statements, &mut owner_statements);
    let result_name = owner_statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Return(TypedExpr {
                expr: ExprIr::Identifier(name),
                ..
            }) => Some(name),
            _ => None,
        })
        .expect("the final result reads the original static source binding");
    let plan = owner_statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncGeneratorForOf(plan) => Some(plan),
            _ => None,
        })
        .expect("existing iterator-owned continuation");
    assert_eq!(plan.execution(), ResumableRegionProtocolIr::Generator);
    assert!(plan.initialization().end_state() < plan.body().entry_state());
    let mut statements = Vec::new();
    flatten(&plan.body().block().statements, &mut statements);
    let received = statements
        .iter()
        .enumerate()
        .filter_map(|(index, statement)| match statement {
            StatementIr::GeneratorYield {
                resume_mode: GeneratorResumeModeIr::AssignIdentifier(name),
                ..
            } => Some((index, name)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        received.len(),
        2,
        "both call operands have real resume storage"
    );
    assert_ne!(received[0].1, received[1].1);
    let (capture_index, capture) = statements
        .iter()
        .enumerate()
        .find_map(|(index, statement)| match statement {
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::EnvironmentIdentifier(identifier),
                ..
            }) => match &identifier.operation {
                EnvironmentIdentifierOperationIr::CaptureAssignmentReference { capture } => {
                    Some((index, capture))
                }
                _ => None,
            },
            _ => None,
        })
        .expect("the original target Reference is selected before both RHS operands");
    assert_eq!(
        capture.access(),
        IdentifierReferenceCaptureAccess::WriteOnly
    );
    assert!(
        matches!(capture.disposition(), IdentifierReferenceCaptureDisposition::Located(
        IdentifierReferenceFallbackDisposition::Declarative { binding }
    ) if matches!(&binding.expr, ExprIr::Identifier(name) if name == result_name))
    );
    let assignment_index = statements.iter().position(|statement| matches!(statement,
        StatementIr::Expression(TypedExpr { expr: ExprIr::EnvironmentIdentifier(identifier), .. })
            if matches!(&identifier.operation, EnvironmentIdentifierOperationIr::PutCapturedReference { reference, .. }
                if reference == capture.reference())))
        .expect("existing local target write after the complete RHS");
    assert!(capture_index < received[0].0);
    assert!(
        assignment_index > received[1].0,
        "target write cannot replace an argument resume"
    );
    for (_, received_name) in received {
        assert!(
            function
                .owned_env_bindings
                .iter()
                .any(|binding| &binding.name == received_name),
            "{received_name} must survive the existing iterator continuation"
        );
    }
}
