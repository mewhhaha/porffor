use super::*;
use lila_front::{parse, ParseOptions};
use serde_json::json;

fn program(source: &str) -> ProgramIr {
    crate::lower(&parse(source, ParseOptions::script()).unwrap())
}

fn input(body: Value) -> IrRobustnessInput {
    IrRobustnessInput::from_bytes(
        &serde_json::to_vec(&json!({
            "schema_version": 1, "body": body,
        }))
        .unwrap(),
    )
    .unwrap()
}

#[test]
fn native_operations_use_original_checked_owners_without_changing_source_identity() {
    let mut actual = program("");
    let mut expected = actual.clone();
    input(json!([
        {"op":"define_property", "target":{"kind":"object"}, "key":"answer",
            "value":{"kind":"number","bits":"4045000000000000"}},
        {"op":"delete_optional", "target":{"kind":"null"}, "strict":true, "chain":[
            {"op":"property", "key":"method", "shorted":true},
            {"op":"call", "args":[], "shorted":false, "boundary_before":true},
            {"op":"property", "key":"answer", "shorted":false}
        ]},
        {"op":"if", "condition":{"kind":"boolean","value":true}, "then":[
            {"op":"block", "body":[{"op":"value", "value":{"kind":"string","value":"kept"}}]}
        ], "else":[{"op":"empty"}]}
    ]))
    .admit_into_empty_script(&mut actual)
    .unwrap();
    let body = &actual.script.as_ref().unwrap().body;
    let StatementIr::Expression(TypedExpr {
        kind: ValueKind::Object,
        possible_kinds,
        expr: ExprIr::ObjectPropertyDefinition(definition),
        ..
    }) = &body.statements[0]
    else {
        panic!("original ordered property definition");
    };
    assert_eq!(
        *possible_kinds,
        crate::KindSet::from_kind(ValueKind::Object)
    );
    assert_eq!(definition.target().kind, ValueKind::Object);
    assert!(matches!(definition.property(), ObjectPropertyIr::Data {
        key, value: TypedExpr { expr: ExprIr::Number(0x4045_0000_0000_0000), .. }, is_shorthand: false
    } if key == "answer"));
    let StatementIr::Expression(TypedExpr {
        kind: ValueKind::Boolean,
        expr: ExprIr::DeleteOptionalPropertyChain(deletion),
        ..
    }) = &body.statements[1]
    else {
        panic!("original terminal property Reference");
    };
    assert_eq!(
        deletion.key(),
        &PropertyKeyIr::StaticString("answer".into())
    );
    assert_eq!(deletion.prefix().len(), 2);
    assert!(matches!(
        deletion.prefix()[1],
        OptionalChainOperationIr::Call {
            boundary_before: true,
            ..
        }
    ));
    assert_eq!(deletion.strictness(), Strictness::Strict);
    assert!(!deletion.shorted());
    assert!(SynchronousLoopBodyIr::new(&StatementIr::Block(body.clone())).is_ok());
    expected.script.as_mut().unwrap().body = body.clone();
    assert_eq!(actual, expected, "only the admitted body may change");
}

#[test]
fn checked_rejections_leave_the_actual_lowered_program_unchanged() {
    let base = program("");
    for (body, expected) in [
        (
            json!([{"op":"define_property", "target":{"kind":"null"}, "key":"x", "value":{"kind":"undefined"}}]),
            IrRobustnessError::ObjectPropertyDefinition(
                ObjectPropertyDefinitionError::NonObjectTarget,
            ),
        ),
        (
            json!([{"op":"delete_optional", "target":{"kind":"object"}, "chain":[], "strict":false}]),
            IrRobustnessError::DeleteOptionalPropertyChain(
                InvalidDeleteOptionalPropertyChainIr::NonPropertyTerminal,
            ),
        ),
        (
            json!([{"op":"delete_optional", "target":{"kind":"null"}, "chain":[
            {"op":"property", "key":"x", "shorted":true},
            {"op":"call", "args":[], "shorted":false, "boundary_before":false}
        ], "strict":false}]),
            IrRobustnessError::DeleteOptionalPropertyChain(
                InvalidDeleteOptionalPropertyChainIr::NonPropertyTerminal,
            ),
        ),
    ] {
        let mut actual = base.clone();
        assert_eq!(
            input(body).admit_into_empty_script(&mut actual),
            Err(expected)
        );
        assert_eq!(actual, base);
    }
    for operation in ["await", "yield"] {
        let mut actual = base.clone();
        let candidate = input(
            json!([{"op":"if", "condition":{"kind":"boolean","value":false},
            "then":[{"op":"block","body":[{"op":operation,"value":{"kind":"null"}}]}], "else":[]} ]),
        );
        assert_eq!(
            candidate.admit_into_empty_script(&mut actual),
            Err(IrRobustnessError::SynchronousBody(
                SynchronousLoopBodyError::Suspension
            ))
        );
        assert_eq!(
            actual, base,
            "even an unselected arm requires the real synchronous proof"
        );
    }
}

#[test]
fn native_wire_cannot_import_opaque_owners_or_replace_existing_source() {
    for body in [
        json!([{"op":"generator_if", "source_id":0}]),
        json!([{"op":"value", "value":{"kind":"function","function_id":"forged"}}]),
        json!([{"op":"value", "value":{"kind":"object", "possible_kinds":["number"]}}]),
        json!([{"op":"await", "value":{"kind":"null"}, "resume_state":123}]),
        json!([{"op":"delete_optional", "target":{"kind":"object"}, "strict":false,
            "chain":[{"op":"private_property", "private_name_id":0, "shorted":false}]}]),
    ] {
        let mut actual = program("");
        let original = actual.clone();
        assert!(matches!(
            input(body).admit_into_empty_script(&mut actual),
            Err(IrRobustnessError::Wire(_))
        ));
        assert_eq!(actual, original);
    }
    for mut actual in [
        program("0;"),
        crate::lower(&parse("", ParseOptions::module()).unwrap()),
    ] {
        let original = actual.clone();
        assert_eq!(
            input(json!([])).admit_into_empty_script(&mut actual),
            Err(IrRobustnessError::NonEmptyScript)
        );
        assert_eq!(actual, original);
    }
}

#[test]
fn admission_bounds_and_schema_are_checked_before_retaining_native_input() {
    assert!(matches!(
        IrRobustnessInput::from_bytes(&vec![b' '; 16_385]),
        Err(IrRobustnessError::ByteLimit)
    ));
    assert!(matches!(
        IrRobustnessInput::from_bytes(br#"{"schema_version":2,"body":[]}"#),
        Err(IrRobustnessError::UnsupportedSchema)
    ));
    assert!(matches!(
        IrRobustnessInput::from_bytes(br#"{"schema_version":1,"body":[],"source_id":0}"#),
        Err(IrRobustnessError::Wire(_))
    ));
    let wide = json!({"schema_version":1,"body":vec![json!({"op":"empty"}); 256]});
    assert!(matches!(
        IrRobustnessInput::from_bytes(&serde_json::to_vec(&wide).unwrap()),
        Err(IrRobustnessError::NodeLimit)
    ));
    let mut nested = json!({"op":"empty"});
    for _ in 0..17 {
        nested = json!({"op":"block","body":[nested]});
    }
    let deep = json!({"schema_version":1,"body":[nested]});
    assert!(matches!(
        IrRobustnessInput::from_bytes(&serde_json::to_vec(&deep).unwrap()),
        Err(IrRobustnessError::DepthLimit)
    ));
    for (length, accepted) in [(512, true), (513, false)] {
        let wire = json!({"schema_version":1,"body":[{"op":"value","value":{"kind":"string","value":"x".repeat(length)}}]});
        let result = IrRobustnessInput::from_bytes(&serde_json::to_vec(&wire).unwrap());
        if accepted {
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(IrRobustnessError::StringLimit)));
        }
    }
}

#[test]
fn native_number_bits_retain_signed_zero_and_nan_payload_without_metadata_overrides() {
    for bits in [
        0u64,
        0x8000_0000_0000_0000,
        0x7ff0_0000_0000_0001,
        0xffff_ffff_ffff_ffff,
    ] {
        let mut actual = program("");
        input(json!([{"op":"value","value":{"kind":"number","bits":format!("{bits:016x}")}}]))
            .admit_into_empty_script(&mut actual)
            .unwrap();
        assert!(matches!(&actual.script.unwrap().body.statements[0],
            StatementIr::Expression(TypedExpr { kind: ValueKind::Number, expr: ExprIr::Number(actual), .. }) if *actual == bits));
    }
    for bits in ["0", "7FF0000000000000", "not-a-number----"] {
        let mut actual = program("");
        assert!(matches!(
            input(json!([{"op":"value","value":{"kind":"number","bits":bits}}]))
                .admit_into_empty_script(&mut actual),
            Err(IrRobustnessError::Wire(_))
        ));
    }
}
