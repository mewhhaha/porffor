use super::*;
use crate::FunctionProtocolIr;
use scriptw_front::decode_js_string_utf16;

fn arguments(values: &[&str]) -> Vec<Vec<u16>> {
    values
        .iter()
        .map(|value| value.encode_utf16().collect())
        .collect()
}

#[test]
fn constructor_grammars_retain_protocol_and_anonymous_unbound_display_name() {
    for (kind, protocol, args) in [
        (
            DynamicFunctionKind::Ordinary,
            FunctionProtocolIr::OrdinaryCallAndConstruct,
            arguments(&["x", "return x;"]),
        ),
        (
            DynamicFunctionKind::Generator,
            FunctionProtocolIr::Generator,
            arguments(&["x", "yield x;"]),
        ),
        (
            DynamicFunctionKind::Async,
            FunctionProtocolIr::Async,
            arguments(&["x", "return x;"]),
        ),
        (
            DynamicFunctionKind::AsyncGenerator,
            FunctionProtocolIr::AsyncGenerator,
            arguments(&["x", "yield x;"]),
        ),
    ] {
        let unit = lower_runtime_function(kind, &args, HostSurfacePolicy::default()).unwrap();
        assert_eq!(unit.kind(), kind);
        let function = unit
            .program()
            .script
            .as_ref()
            .unwrap()
            .functions
            .iter()
            .find(|function| &function.id == unit.function_id())
            .unwrap();
        assert_eq!(function.protocol, protocol);
        assert_eq!(function.name, "anonymous");
        assert!(!function.is_named_expression);
        assert!(function.captured_bindings.is_empty());
    }
}

#[test]
fn source_unit_transport_retains_lone_surrogates_and_exact_constructor_to_string() {
    let body = vec![114, 101, 116, 117, 114, 110, 32, 39, 0xd800, 39, 59];
    let args = vec![body];
    let unit = lower_runtime_function(
        DynamicFunctionKind::Ordinary,
        &args,
        HostSurfacePolicy::default(),
    )
    .unwrap();
    let function = unit
        .program()
        .script
        .as_ref()
        .unwrap()
        .functions
        .iter()
        .find(|function| &function.id == unit.function_id())
        .unwrap();
    let CallableToStringRepresentation::ExactSource(source) = &function.to_string_representation
    else {
        panic!("source function")
    };
    assert_eq!(
        decode_js_string_utf16(source),
        FunctionParseKind::Ordinary.source_text_utf16(&args)
    );
}

#[test]
fn independent_parameter_and_body_grammars_preserve_real_parse_rejections() {
    for args in [
        arguments(&["a/*", "*/){}"]),
        arguments(&["a", "a", "'use strict'; return 1;"]),
        arguments(&["a)", "return 1;"]),
        arguments(&["return )"]),
    ] {
        assert!(matches!(
            lower_runtime_function(
                DynamicFunctionKind::Ordinary,
                &args,
                HostSurfacePolicy::default()
            ),
            Err(RuntimeFunctionCompileError::Parse(_))
        ));
    }
    assert!(matches!(
        lower_runtime_function(
            DynamicFunctionKind::Ordinary,
            &arguments(&["yield 1;"]),
            HostSurfacePolicy::default()
        ),
        Err(RuntimeFunctionCompileError::Parse(_))
    ));
}
