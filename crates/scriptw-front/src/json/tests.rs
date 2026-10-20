use super::*;

#[test]
fn json_retains_exact_source_order_duplicate_names_and_utf16() {
    let source =
        " \t\r\n{\"a\":1,\"a\":2,\"x\":\"\\uD800\\uDC00\\uDFFF😀\",\"b\":[null,true,false]} ";
    let parsed = parse_json(source.into()).unwrap();
    assert_eq!(parsed.source_text(), source);
    let JsonValue::Object(entries) = parsed.value() else {
        panic!("object");
    };
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].0, vec![97]);
    assert_eq!(entries[1].0, vec![97]);
    assert_eq!(entries[0].1, JsonValue::Number(1f64.to_bits()));
    assert_eq!(entries[1].1, JsonValue::Number(2f64.to_bits()));
    assert_eq!(
        entries[2].1,
        JsonValue::String(vec![0xd800, 0xdc00, 0xdfff, 0xd83d, 0xde00])
    );
    assert_eq!(
        entries[3].1,
        JsonValue::Array(vec![
            JsonValue::Null,
            JsonValue::Boolean(true),
            JsonValue::Boolean(false)
        ])
    );
}

#[test]
fn json_decimal_conversion_preserves_negative_zero_and_overflow() {
    for (source, number) in [
        ("-0", -0.0),
        ("-0.0e9999", -0.0),
        ("1e400", f64::INFINITY),
        ("-1e400", f64::NEG_INFINITY),
        ("1e-400", 0.0),
        ("6.022e23", 6.022e23),
    ] {
        assert_eq!(
            parse_json(source.into()).unwrap().value(),
            &JsonValue::Number(number.to_bits())
        );
    }
}

#[test]
fn javascript_grammar_and_incomplete_json_are_rejected_at_a_byte_boundary() {
    for source in [
        "",
        "undefined",
        "NaN",
        "Infinity",
        "+1",
        "01",
        "0x10",
        "1.",
        "1e",
        "-.1",
        "[1,]",
        "{\"a\":1,}",
        "{'a':1}",
        "{a:1}",
        "true false",
        "/* x */0",
        "// x\n0",
        "\u{a0}0",
        "\"\n\"",
        "\"\\x41\"",
        "\"\\u123\"",
        "\"\\v\"",
        "[",
        "{\"a\":",
        "[1}",
    ] {
        let error = parse_json(source.into()).unwrap_err();
        assert!(error.byte_offset() <= source.len(), "{source}");
        assert!(source.is_char_boundary(error.byte_offset()), "{source}");
        assert!(!error.message().is_empty());
    }
}

#[test]
fn json_containers_parse_and_release_using_source_sized_explicit_frames() {
    let depth = 4096;
    let source = format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
    let parsed = parse_json(source).unwrap();
    let mut value = parsed.value();
    for _ in 0..depth {
        let JsonValue::Array(children) = value else {
            panic!("array");
        };
        assert_eq!(children.len(), 1);
        value = &children[0];
    }
    assert_eq!(value, &JsonValue::Number(0.0f64.to_bits()));
    let cloned = parsed.clone();
    assert_eq!(cloned, parsed);
    drop(parsed);
    drop(cloned);
    assert!(parse_json(format!("{}0{}!", "[".repeat(depth), "]".repeat(depth))).is_err());
}
