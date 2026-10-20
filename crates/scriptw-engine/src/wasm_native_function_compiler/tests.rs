use super::*;
use scriptw_front::ParseError;
use scriptw_ir::{IrDiagnostic, RuntimeFunctionCompileError};
use wasmtime::{FuncType, HeapType, RefType, Rooted, ValType};

mod fixture;

fn execution() -> WasmAgentExecutionControl {
    let control = WasmAgentExecutionControl::new();
    control.arm(std::time::Instant::now(), Some(30_000));
    control
}

#[test]
fn syntax_classification_retains_parser_capability_origin_despite_message_text() {
    let malformed =
        RuntimeFunctionCompileError::Parse(ParseError::malformed("true syntax rejection", None));
    let CompileRejection::Syntax(message) = CompileRejection::classify(malformed).unwrap();
    assert_eq!(
        message,
        "true syntax rejection".encode_utf16().collect::<Vec<_>>()
    );
    let unsupported = RuntimeFunctionCompileError::Parse(ParseError::unsupported_parser_feature(
        "SyntaxError: unexpected token",
        None,
    ));
    let error = CompileRejection::classify(unsupported).unwrap_err();
    let error = error
        .downcast_ref::<EngineError>()
        .expect("existing typed EngineError carrier");
    assert_eq!(
        error.parse_diagnostic().unwrap().kind(),
        scriptw_front::ParseDiagnosticKind::UnsupportedParserFeature
    );
    assert!(error.ir_diagnostic().is_none());
}

#[test]
fn only_coded_lowering_syntax_can_enter_the_syntax_message_branch() {
    let diagnostic = IrDiagnostic::rejected(
        scriptw_front::EarlyErrorCode::DuplicateLexicalDeclaration,
        "actual early rejection",
        None,
    );
    assert!(matches!(
        CompileRejection::classify(RuntimeFunctionCompileError::Lowering(vec![diagnostic]))
            .unwrap(),
        CompileRejection::Syntax(_)
    ));
    let diagnostic = IrDiagnostic::unsupported("SyntaxError in unsupported source");
    let error = CompileRejection::classify(RuntimeFunctionCompileError::Lowering(vec![
        diagnostic.clone()
    ]))
    .unwrap_err();
    assert_eq!(
        error.downcast_ref::<EngineError>().unwrap().ir_diagnostic(),
        Some(&diagnostic)
    );
    for error in [
        RuntimeFunctionCompileError::Lowering(vec![]),
        RuntimeFunctionCompileError::InvalidCompilerOutput("missing validated root"),
    ] {
        assert!(CompileRejection::classify(error)
            .unwrap_err()
            .is::<EngineError>());
    }
}

#[test]
fn constructor_kind_words_are_closed_before_frontend_entry() {
    for (word, kind) in [
        (0, DynamicFunctionKind::Ordinary),
        (1, DynamicFunctionKind::Generator),
        (2, DynamicFunctionKind::Async),
        (3, DynamicFunctionKind::AsyncGenerator),
    ] {
        assert_eq!(sources::decode_kind(word).unwrap(), kind);
    }
    for word in [-1, 4, i32::MAX, i32::MIN] {
        assert!(sources::decode_kind(word).is_err());
    }
}

#[test]
fn host_projection_admission_checks_scalar_width_nullability_and_result_sum() {
    let engine = shared_wasm_engine().unwrap();
    let signature = fixture::signature(&engine);
    for slot in [0, 1, 2] {
        let mut arguments = signature.arguments.clone();
        arguments[slot] = ValType::F64;
        assert!(NativeFunctionSignature::validate(FuncType::new(
            &engine,
            arguments,
            signature.function.results()
        ))
        .is_err());
    }
    let mut arguments = signature.arguments.clone();
    let heap = arguments[4].unwrap_ref().heap_type().clone();
    arguments[4] = ValType::Ref(RefType::new(true, heap));
    assert!(NativeFunctionSignature::validate(FuncType::new(
        &engine,
        arguments,
        signature.function.results()
    ))
    .is_err());
    let mut results = signature.function.results().collect::<Vec<_>>();
    results.pop();
    assert!(NativeFunctionSignature::validate(FuncType::new(
        &engine,
        signature.arguments.clone(),
        results
    ))
    .is_err());
}

#[test]
fn factory_admission_uses_actual_owner_and_semantic_types_without_dummy_functions() {
    let engine = shared_wasm_engine().unwrap();
    let signature = fixture::signature(&engine);
    let owner = ValType::Ref(RefType::new(
        false,
        HeapType::ConcreteStruct(signature.program.clone()),
    ));
    let mut parameters = vec![owner.clone()];
    parameters.extend(signature.arguments.iter().cloned());
    let mut results = signature.completion.clone();
    results.push(ValType::I32);
    factory::validate_initializer_type(&FuncType::new(&engine, [], [owner]), &signature).unwrap();
    factory::validate_factory_type(
        &FuncType::new(&engine, parameters.clone(), results.clone()),
        &signature,
    )
    .unwrap();
    parameters[0] = ValType::Ref(RefType::new(
        true,
        HeapType::ConcreteStruct(signature.program.clone()),
    ));
    assert!(factory::validate_factory_type(
        &FuncType::new(&engine, parameters, results.clone()),
        &signature
    )
    .is_err());
    results.pop();
    assert!(factory::validate_factory_type(
        &FuncType::new(
            &engine,
            std::iter::once(ValType::Ref(RefType::new(
                false,
                HeapType::ConcreteStruct(signature.program.clone())
            )))
            .chain(signature.arguments.iter().cloned()),
            results
        ),
        &signature
    )
    .is_err());
}

#[test]
fn converted_sources_keep_exact_utf16_and_reject_non_string_entries_before_parsing() {
    let mut store = fixture::store();
    let signature = fixture::signature(store.engine());
    let units = [0, 0xd800, 0x41, 0xdc00, 0xd83d, 0xde00];
    let operands = fixture::operands(&mut store, &signature, &[&units, &[]]);
    store.gc(None).unwrap();
    let request =
        ConvertedRequest::decode(&mut store, &operands, &signature, &execution()).unwrap();
    assert_eq!(request.arguments, vec![units.to_vec(), vec![]]);
    let Val::AnyRef(Some(array)) = operands[4] else {
        panic!("source array")
    };
    let array = array.as_array(&store).unwrap().unwrap();
    let mut fields = vec![Val::I32(0); signature.stored.fields().len()];
    fields[scriptw_aot_wasm::GcHostField::StoredTag
        .definition()
        .index() as usize] = Val::I32(WasmRuntimeValueTag::Number.tag());
    fields[scriptw_aot_wasm::GcHostField::StoredScalar
        .definition()
        .index() as usize] = Val::I64(42.0_f64.to_bits() as i64);
    fields[scriptw_aot_wasm::GcHostField::StoredReference
        .definition()
        .index() as usize] = Val::AnyRef(None);
    let pre = wasmtime::StructRefPre::new(&mut store, signature.stored.clone());
    let bad = wasmtime::StructRef::new(&mut store, &pre, &fields).unwrap();
    array.set(&mut store, 0, Val::from(bad)).unwrap();
    assert!(
        ConvertedRequest::decode(&mut store, &operands, &signature, &execution())
            .unwrap_err()
            .is::<EngineError>()
    );
}

#[test]
fn converted_source_decode_preserves_unarmed_and_expired_execution_authority() {
    let mut store = fixture::store();
    let signature = fixture::signature(store.engine());
    let operands = fixture::operands(&mut store, &signature, &[&[0x41]]);
    let unarmed = WasmAgentExecutionControl::new();
    let expired = WasmAgentExecutionControl::new();
    expired.arm(
        std::time::Instant::now() - std::time::Duration::from_secs(1),
        Some(1),
    );
    for control in [&unarmed, &expired] {
        let before = control
            .remaining()
            .unwrap_err()
            .wasm_execution_failure_kind();
        assert!(before.is_some());
        let error =
            ConvertedRequest::decode(&mut store, &operands, &signature, control).unwrap_err();
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
                .unwrap_err()
                .wasm_execution_failure_kind(),
            before
        );
    }
}

#[test]
fn syntax_and_throw_completion_roots_survive_scope_exit_and_collection() {
    let mut store = fixture::store();
    let signature = fixture::signature(store.engine());
    let units = [0xd800, 0, 0xdc00, 0xd83d, 0xde00];
    let syntax = {
        let mut roots = RootScope::new(&mut store);
        EscapedReply::syntax(&mut roots, &units, &signature).unwrap()
    };
    store.gc(None).unwrap();
    let mut output = [Val::I32(0); 6];
    syntax.write(&mut store, &mut output).unwrap();
    let Val::AnyRef(Some(message)) = output[5] else {
        panic!("syntax message")
    };
    store.gc(None).unwrap();
    let message = message.as_eqref(&mut store).unwrap().unwrap();
    assert_eq!(
        &*wasm_gc_completion::string_units(&mut store, message).unwrap(),
        units
    );
    let original = fixture::string(&mut store, &signature, &units).to_anyref();
    let completion = {
        let mut roots = RootScope::new(&mut store);
        EscapedReply::completion(
            &mut roots,
            [
                Val::I32(WasmRuntimeValueTag::String.tag()),
                Val::I64(0),
                Val::AnyRef(Some(original)),
                Val::I32(CompletionKindIr::Throw.abi_code() as i32),
                Val::I32(0),
            ],
        )
        .unwrap()
    };
    store.gc(None).unwrap();
    completion.write(&mut store, &mut output).unwrap();
    assert!(
        matches!(output[3], Val::I32(kind) if i64::from(kind)==CompletionKindIr::Throw.abi_code())
    );
    assert!(matches!(output[5], Val::AnyRef(None)));
    let Val::AnyRef(Some(returned)) = output[2] else {
        panic!("retained thrown value")
    };
    assert!(Rooted::ref_eq(&store, &original, &returned).unwrap());
}
