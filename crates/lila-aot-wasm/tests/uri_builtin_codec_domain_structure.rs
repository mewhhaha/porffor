const URI_SOURCE: &str = include_str!("../src/builtins/uri.rs");
const STRING_SOURCE: &str = include_str!("../src/builtins/string.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn uri_builtin_domain_carries_codec_kind_with_direction() {
    let domain = normalized(bounded(
        URI_SOURCE,
        "enum UriBuiltin {",
        "#[allow(non_upper_case_globals)]",
    ));
    assert_eq!(
        domain,
        "Escape,Unescape,Encode(UriCodecKind),Decode(UriCodecKind),}"
    );
    assert!(!domain.contains("EncodeUri,"));
    assert!(!domain.contains("EncodeUriComponent,"));
    assert!(!domain.contains("DecodeUri,"));
    assert!(!domain.contains("DecodeUriComponent,"));

    let producers = normalized(bounded(
        URI_SOURCE,
        "impl UriBuiltin {",
        "impl<'a> FunctionBuilder<'a> {",
    ));
    for mapping in [
        "constEncodeUri:Self=Self::Encode(UriCodecKind::Uri);",
        "constEncodeUriComponent:Self=Self::Encode(UriCodecKind::Component);",
        "constDecodeUri:Self=Self::Decode(UriCodecKind::Uri);",
        "constDecodeUriComponent:Self=Self::Decode(UriCodecKind::Component);",
    ] {
        assert_eq!(producers.matches(mapping).count(), 1, "mapping `{mapping}`");
    }
}

#[test]
fn uri_emitter_dispatches_one_closed_domain_after_string_coercion() {
    let emitter = normalized(
        URI_SOURCE
            .split_once("fn emit_uri_builtin(")
            .expect("missing URI builtin emitter")
            .1,
    );

    assert!(emitter.contains("builtin:UriBuiltin,"));
    assert_eq!(emitter.matches("matchbuiltin{").count(), 1);
    for arm in [
        "UriBuiltin::Escape=>{self.emit_annexb_escape_string_payload(string_local,function)?;}",
        "UriBuiltin::Unescape=>{self.emit_annexb_unescape_string_payload(string_local,function)?;}",
        "UriBuiltin::Encode(codec_kind)=>{self.emit_uri_encode_string_payload(string_local,codec_kind,function)?;}",
        "UriBuiltin::Decode(codec_kind)=>{self.emit_uri_decode_string_payload(string_local,codec_kind,function)?;}",
    ] {
        assert_eq!(emitter.matches(arm).count(), 1, "arm `{arm}`");
    }
    assert!(!emitter.contains("ifbuiltin"));
    assert!(!emitter.contains("_=>"));
    assert!(!emitter.contains("unreachable!"));

    let coercion = emitter
        .find("self.emit_value_to_string_payload")
        .expect("missing argument string coercion");
    let dispatch = emitter.find("matchbuiltin{").expect("missing URI dispatch");
    let result = emitter
        .find("LocalSet(self.result_local)")
        .expect("missing result publication");
    assert!(coercion < dispatch);
    assert!(dispatch < result);
}

#[test]
fn uri_codec_identity_has_no_fallback_capabilities() {
    let codec_domain = normalized(bounded(
        STRING_SOURCE,
        "pub(crate) enum UriCodecKind {",
        "enum RegExpExecResultMode",
    ));
    assert_eq!(codec_domain, "Uri,Component,}");
    assert!(!STRING_SOURCE.contains(")]\npub(crate) enum UriCodecKind"));
    assert!(!URI_SOURCE.contains(")]\nenum UriBuiltin"));
    for capability in ["Clone", "Copy", "PartialEq", "Eq"] {
        assert!(!STRING_SOURCE.contains(&format!("impl {capability} for UriCodecKind")));
        assert!(!URI_SOURCE.contains(&format!("impl {capability} for UriBuiltin")));
    }

    let encode = bounded(
        STRING_SOURCE,
        "pub(crate) fn emit_uri_encode_string_payload(",
        "pub(crate) fn emit_uri_decode_string_payload(",
    );
    assert!(encode.contains(
        "self.emit_uri_unescaped_codepoint_i32(codepoint_local, &codec_kind, function);"
    ));

    let unescaped = bounded(
        STRING_SOURCE,
        "fn emit_uri_unescaped_codepoint_i32(",
        "fn emit_uri_reserved_ascii_i32(",
    );
    assert!(unescaped.contains("codec_kind: &UriCodecKind,"));
    assert!(unescaped.contains("let punctuation = match codec_kind {"));
    assert!(unescaped.contains("UriCodecKind::Uri =>"));
    assert!(unescaped.contains("UriCodecKind::Component =>"));
    assert!(!unescaped.contains("_ =>"));

    let decode = bounded(
        STRING_SOURCE,
        "pub(crate) fn emit_uri_decode_string_payload(",
        "fn emit_uri_error_and_return(",
    );
    assert!(decode.contains("match codec_kind {"));
    assert!(decode.contains("UriCodecKind::Uri => {"));
    assert!(decode.contains("UriCodecKind::Component => {"));
    assert!(!decode.contains("codec_kind =="));
    assert!(!decode.contains("_ =>"));
}

#[test]
fn standard_dispatch_uses_six_fixed_uri_operations() {
    assert!(!STANDARD_SOURCE.contains("UriBuiltin"));
    assert!(!STANDARD_SOURCE.contains("emit_uri_builtin("));

    for (builtin, wrapper, variant) in [
        ("Escape", "emit_escape_builtin", "Escape"),
        ("Unescape", "emit_unescape_builtin", "Unescape"),
        ("EncodeUri", "emit_encode_uri_builtin", "EncodeUri"),
        (
            "EncodeUriComponent",
            "emit_encode_uri_component_builtin",
            "EncodeUriComponent",
        ),
        ("DecodeUri", "emit_decode_uri_builtin", "DecodeUri"),
        (
            "DecodeUriComponent",
            "emit_decode_uri_component_builtin",
            "DecodeUriComponent",
        ),
    ] {
        assert_eq!(
            STANDARD_SOURCE
                .matches(&format!("StandardBuiltinId::{builtin} =>"))
                .count(),
            1
        );
        assert_eq!(
            STANDARD_SOURCE
                .matches(&format!("self.{wrapper}(function)?"))
                .count(),
            1
        );
        let wrapper = bounded(
            URI_SOURCE,
            &format!("    pub(super) fn {wrapper}("),
            "\n    }",
        );
        assert_eq!(wrapper.matches("self.emit_uri_builtin(").count(), 1);
        assert_eq!(
            wrapper.matches(&format!("UriBuiltin::{variant}")).count(),
            1
        );
    }
}
