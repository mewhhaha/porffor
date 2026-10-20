use super::*;
use scriptw_front::ParseError;
use scriptw_ir::{IrDiagnostic, RuntimeGlobalScriptCompileError, RuntimeGlobalScriptKind};
use wasmtime::{Func, FuncType, HeapType, RefType, Rooted, ValType};
mod fixture;
fn execution() -> WasmAgentExecutionControl {
    let e = WasmAgentExecutionControl::new();
    e.arm(std::time::Instant::now(), Some(30_000));
    e
}

#[test]
fn only_actual_malformed_parse_can_produce_the_syntax_message_variant() {
    let CompileRejection::Syntax(message) =
        CompileRejection::classify(RuntimeGlobalScriptCompileError::Parse(
            ParseError::malformed("actual source rejection", None),
        ))
        .unwrap();
    assert_eq!(
        message,
        "actual source rejection".encode_utf16().collect::<Vec<_>>()
    );
    let error = CompileRejection::classify(RuntimeGlobalScriptCompileError::Parse(
        ParseError::unsupported_parser_feature("SyntaxError: unexpected token", None),
    ))
    .err()
    .unwrap();
    assert_eq!(
        error
            .downcast_ref::<EngineError>()
            .unwrap()
            .parse_diagnostic()
            .unwrap()
            .kind(),
        scriptw_front::ParseDiagnosticKind::UnsupportedParserFeature
    );
    for diagnostic in [
        IrDiagnostic::unsupported("SyntaxError looking unsupported gap"),
        IrDiagnostic::rejected(
            scriptw_front::EarlyErrorCode::DuplicateLexicalDeclaration,
            "coded lowering SyntaxError",
            None,
        ),
    ] {
        let error = CompileRejection::classify(RuntimeGlobalScriptCompileError::Lowering(vec![
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
        CompileRejection::classify(RuntimeGlobalScriptCompileError::InvalidCompilerOutput(
            "missing root"
        ))
        .is_err()
    );
}

#[test]
fn global_source_kind_domain_excludes_direct_eval_shadow_and_function_kinds() {
    assert_eq!(
        sources::decode_kind(0).unwrap(),
        RuntimeGlobalScriptKind::IndirectEval
    );
    assert_eq!(
        sources::decode_kind(1).unwrap(),
        RuntimeGlobalScriptKind::RealmScript
    );
    for word in [-1, 2, 3, 4, i32::MAX, i32::MIN] {
        assert!(sources::decode_kind(word).is_err());
    }
}

#[test]
fn global_import_and_factory_types_bind_nonnull_core_owners_and_exact_status_result() {
    let engine = shared_wasm_engine().unwrap();
    let signature = fixture::signature(&engine);
    let mut parameters = signature.arguments.clone();
    parameters[2] = ValType::Ref(RefType::new(
        true,
        HeapType::ConcreteStruct(signature.realm.clone()),
    ));
    assert!(NativeGlobalScriptSignature::validate(FuncType::new(
        &engine,
        parameters,
        signature.function.results()
    ))
    .is_err());
    let owner = ValType::Ref(RefType::new(
        false,
        HeapType::ConcreteStruct(signature.program.clone()),
    ));
    factory::validate_initializer_type(&FuncType::new(&engine, [], [owner.clone()]), &signature)
        .unwrap();
    let mut parameters = vec![owner];
    parameters.extend(signature.arguments[..3].iter().cloned());
    let mut results = signature.completion.clone();
    results.push(ValType::I32);
    factory::validate_factory_type(
        &FuncType::new(&engine, parameters.clone(), results.clone()),
        &signature,
    )
    .unwrap();
    parameters.swap(2, 3);
    assert!(factory::validate_factory_type(
        &FuncType::new(&engine, parameters, results.clone()),
        &signature
    )
    .is_err());
    results.pop();
    let parameters = std::iter::once(ValType::Ref(RefType::new(
        false,
        HeapType::ConcreteStruct(signature.program.clone()),
    )))
    .chain(signature.arguments[..3].iter().cloned());
    assert!(factory::validate_factory_type(
        &FuncType::new(&engine, parameters, results),
        &signature
    )
    .is_err());
}

#[test]
fn global_requests_retain_exact_code_units_and_the_original_unarmed_or_expired_budget() {
    let mut store = fixture::store();
    let signature = fixture::signature(store.engine());
    let units = [0, 0xd800, 0x41, 0xdfff, 0xdb80, 0xdc00];
    let operands = fixture::operands(&mut store, &signature, &units);
    let request =
        ConvertedRequest::decode(&mut store, &operands, &signature, &execution()).unwrap();
    store.gc(None).unwrap();
    assert_eq!(request.source, units);
    let unarmed = WasmAgentExecutionControl::new();
    let expired = WasmAgentExecutionControl::new();
    expired.arm(
        std::time::Instant::now() - std::time::Duration::from_secs(1),
        Some(1),
    );
    for control in [&unarmed, &expired] {
        let before = control
            .remaining()
            .err()
            .unwrap()
            .wasm_execution_failure_kind();
        let error = ConvertedRequest::decode(&mut store, &operands, &signature, control)
            .err()
            .unwrap();
        assert_eq!(
            error
                .downcast_ref::<EngineError>()
                .unwrap()
                .wasm_execution_failure_kind(),
            before
        );
        assert_eq!(
            control
                .remaining()
                .err()
                .unwrap()
                .wasm_execution_failure_kind(),
            before
        );
    }
}

#[test]
fn global_syntax_and_throw_replies_survive_temporary_scope_exit_and_gc() {
    let mut store = fixture::store();
    let signature = fixture::signature(store.engine());
    let units = [0xd800, 0, 0xdfff];
    let reply = {
        let mut scope = RootScope::new(&mut store);
        EscapedReply::syntax(&mut scope, &units, &signature).unwrap()
    };
    store.gc(None).unwrap();
    let mut output = [Val::I32(0); 7];
    reply.write(&mut store, &mut output).unwrap();
    assert!(matches!(output[5], Val::I32(1)));
    let Val::AnyRef(Some(message)) = output[6] else {
        panic!("syntax message")
    };
    let message = message.as_eqref(&mut store).unwrap().unwrap();
    assert_eq!(
        &*wasm_gc_completion::string_units(&mut store, message).unwrap(),
        units
    );
    let source = fixture::string(&mut store, &signature, &units).to_anyref();
    let reply = {
        let mut scope = RootScope::new(&mut store);
        EscapedReply::entered(
            &mut scope,
            [
                Val::I32(WasmRuntimeValueTag::String.tag()),
                Val::I64(0),
                Val::AnyRef(Some(source)),
                Val::I32(CompletionKindIr::Throw.abi_code() as i32),
                Val::I32(0),
                Val::I32(2),
            ],
        )
        .unwrap()
    };
    store.gc(None).unwrap();
    reply.write(&mut store, &mut output).unwrap();
    assert!(matches!(output[5], Val::I32(2)) && matches!(output[6], Val::AnyRef(None)));
    let Val::AnyRef(Some(returned)) = output[2] else {
        panic!("throw reference")
    };
    assert!(Rooted::ref_eq(&store, &source, &returned).unwrap());
    assert!(EscapedReply::entered(
        &mut store,
        [
            Val::I32(0),
            Val::I64(0),
            Val::AnyRef(None),
            Val::I32(0),
            Val::I32(0),
            Val::I32(0)
        ]
    )
    .is_err());
}

#[test]
fn raw_external_callbacks_without_the_known_compilation_context_fail_closed() {
    let mut store = fixture::store();
    let signature = Arc::new(fixture::signature(store.engine()));
    let parameters = fixture::operands(&mut store, &signature, &[0x31]);
    let ty = signature.function.clone();
    let function = Func::new(&mut store, ty, move |caller, input, output| {
        compile(caller, input, output, &signature)
    });
    let mut output = [Val::I32(0); 7];
    let error = function
        .call(&mut store, &parameters, &mut output)
        .unwrap_err();
    assert!(error.downcast_ref::<EngineError>().is_some());
    assert!(format!("{error:#}").contains("no armed native source compiler context"));
}
