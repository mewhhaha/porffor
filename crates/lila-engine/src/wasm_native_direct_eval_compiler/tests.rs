use super::*;
use lila_front::ParseError;
use lila_ir::{
    IrDiagnostic, RuntimeDirectEvalCaller, RuntimeDirectEvalCallerDescriptor,
    RuntimeDirectEvalCompileError, RuntimeDirectEvalPrivateScope,
};
use wasmtime::{Rooted, Val};
mod fixture;

#[test]
fn only_malformed_javascript_parse_rejections_become_direct_eval_syntax_messages() {
    let CompileRejection::Syntax(message) = CompileRejection::classify(
        RuntimeDirectEvalCompileError::Parse(ParseError::malformed("actual source error", None)),
    )
    .unwrap();
    assert_eq!(
        message,
        "actual source error".encode_utf16().collect::<Vec<_>>()
    );
    let unsupported = CompileRejection::classify(RuntimeDirectEvalCompileError::Parse(
        ParseError::unsupported_parser_feature("SyntaxError-looking parser capability", None),
    ))
    .err()
    .unwrap();
    assert_eq!(
        unsupported
            .downcast_ref::<EngineError>()
            .unwrap()
            .parse_diagnostic()
            .unwrap()
            .kind(),
        lila_front::ParseDiagnosticKind::UnsupportedParserFeature
    );
    for diagnostic in [
        IrDiagnostic::unsupported("SyntaxError-looking compiler gap"),
        IrDiagnostic::rejected(
            lila_front::EarlyErrorCode::DuplicateLexicalDeclaration,
            "classified lowering early error",
            None,
        ),
    ] {
        let error = CompileRejection::classify(RuntimeDirectEvalCompileError::Lowering(vec![
            diagnostic.clone(),
        ]))
        .err()
        .unwrap();
        assert_eq!(
            error.downcast_ref::<EngineError>().unwrap().ir_diagnostic(),
            Some(&diagnostic)
        );
    }
    assert!(
        CompileRejection::classify(RuntimeDirectEvalCompileError::InvalidCompilerOutput(
            "different entry role"
        ))
        .is_err()
    );
}

#[test]
fn factory_request_identity_includes_hidden_private_floor_and_exact_utf16_source() {
    let mut bytes = b"LILADE01".to_vec();
    bytes.extend([0, 1, 0, 0]);
    bytes.extend(0_u64.to_le_bytes());
    let descriptor = RuntimeDirectEvalCallerDescriptor::decode(&bytes).unwrap();
    let caller = RuntimeDirectEvalCaller::validate(
        descriptor.clone(),
        [RuntimeDirectEvalPrivateScope::new(7, 0)],
    )
    .unwrap();
    let other =
        RuntimeDirectEvalCaller::validate(descriptor, [RuntimeDirectEvalPrivateScope::new(9, 0)])
            .unwrap();
    assert_eq!(caller.descriptor(), other.descriptor());
    assert_eq!(caller.context(), other.context());
    assert_ne!(caller, other);
    let source: Vec<_> = "42;".encode_utf16().collect();
    let program =
        lila_ir::lower_runtime_direct_eval(&caller, &source, HostSurfacePolicy::default()).unwrap();
    factory::require_exact_program(&caller, &source, &program).unwrap();
    assert!(factory::require_exact_program(&other, &source, &program).is_err());
    assert!(factory::require_exact_program(&caller, &[0xd800], &program).is_err());
}

#[test]
fn actual_syntax_and_thrown_reference_roots_survive_temporary_scope_exit_and_gc() {
    let mut store = fixture::store();
    let (string, units) = fixture::string_types(store.engine());
    let text = [0xd800, 0, 0xdfff];
    let reply = {
        let mut roots = RootScope::new(&mut store);
        EscapedReply::syntax(&mut roots, &text, &string, &units).unwrap()
    };
    store.gc(None).unwrap();
    let mut output = [Val::I32(0); 7];
    reply.write(&mut store, &mut output).unwrap();
    let Val::AnyRef(Some(message)) = output[6] else {
        panic!("rooted syntax String");
    };
    assert!(matches!(output[5], Val::I32(1)));
    let reference = message.as_eqref(&mut store).unwrap().unwrap();
    assert_eq!(
        &*wasm_gc_completion::string_units(&mut store, reference).unwrap(),
        text
    );
    let reply = {
        let mut roots = RootScope::new(&mut store);
        EscapedReply::entered(
            &mut roots,
            [
                Val::I32(WasmRuntimeValueTag::String.tag()),
                Val::I64(0),
                Val::AnyRef(Some(message)),
                Val::I32(CompletionKindIr::Throw.abi_code() as i32),
                Val::I32(0),
                Val::I32(2),
            ],
        )
        .unwrap()
    };
    store.gc(None).unwrap();
    reply.write(&mut store, &mut output).unwrap();
    let Val::AnyRef(Some(returned)) = output[2] else {
        panic!("rooted thrown reference");
    };
    assert!(Rooted::ref_eq(&store, &message, &returned).unwrap());
    assert!(matches!(output[5], Val::I32(2)) && matches!(output[6], Val::AnyRef(None)));
}
