use scriptw_front::{parse, ParseOptions};
use scriptw_ir::{lower, CallableToStringRepresentation};

#[test]
fn function_source_spans_translate_utf16_positions_to_utf8_boundaries() {
    for (source, expected) in [
        ("function f() { /* é 🌱 */ }", "function f() { /* é 🌱 */ }"),
        (
            "(function f() { /* é 🌱 */ });",
            "function f() { /* é 🌱 */ }",
        ),
        (
            "function* f() { /* é 🌱 */ }",
            "function* f() { /* é 🌱 */ }",
        ),
        (
            "(function* f() { /* é 🌱 */ });",
            "function* f() { /* é 🌱 */ }",
        ),
        (
            "async function f() { /* é 🌱 */ }",
            "async function f() { /* é 🌱 */ }",
        ),
        (
            "(async function f() { /* é 🌱 */ });",
            "async function f() { /* é 🌱 */ }",
        ),
        (
            "async function* f() { /* é 🌱 */ }",
            "async function* f() { /* é 🌱 */ }",
        ),
        (
            "(async function* f() { /* é 🌱 */ });",
            "async function* f() { /* é 🌱 */ }",
        ),
        ("var f = () => { /* é 🌱 */ };", "() => { /* é 🌱 */ }"),
        (
            "var f = async () => { /* é 🌱 */ };",
            "async () => { /* é 🌱 */ }",
        ),
        (
            "({ async f() { /* é 🌱 */ } });",
            "async f() { /* é 🌱 */ }",
        ),
        (
            "({ async *f() { /* é 🌱 */ } });",
            "async *f() { /* é 🌱 */ }",
        ),
        (
            "class C { static async f() { /* é 🌱 */ } }",
            "async f() { /* é 🌱 */ }",
        ),
        (
            "(class C { async *f() { /* é 🌱 */ } });",
            "async *f() { /* é 🌱 */ }",
        ),
        ("class C { /* é 🌱 */ }", "class C { /* é 🌱 */ }"),
        ("(class C { /* é 🌱 */ });", "class C { /* é 🌱 */ }"),
    ] {
        let source = format!("/* André 🌱 */\n{source}");
        let parsed = parse(&source, ParseOptions::script()).expect("function source parses");
        let program = lower(&parsed);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.expect("script IR");
        assert!(
            script.functions.iter().any(|function| {
                function.to_string_representation
                    == CallableToStringRepresentation::ExactSource(expected.to_string())
            }),
            "missing exact source {expected:?} in {source:?}: {:?}",
            script
                .functions
                .iter()
                .map(|function| &function.to_string_representation)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn function_source_preserves_line_terminators_between_comments_and_tokens() {
    for terminator in ["\n", "\r", "\r\n", "\u{2028}", "\u{2029}"] {
        let declaration = [
            "function", "// a", "f", "// b", "(", "// c", "x", "// d", ",", "// e", "y", "// f",
            ")", "// g", "{", "// h", ";", "// i", ";", "// j", "}",
        ]
        .join(terminator);
        let source = format!("// before{terminator}{declaration}{terminator}// after");
        let parsed = parse(&source, ParseOptions::script()).expect("multiline declaration parses");
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.expect("script IR");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "f")
            .expect("declaration IR");
        assert_eq!(
            function.to_string_representation,
            CallableToStringRepresentation::ExactSource(declaration)
        );
    }
}

#[test]
fn prepared_callable_source_spans_preserve_lone_units_and_literal_marker_data() {
    use scriptw_front::{prepare_eval_source_utf16, EvalParseContext, ParsedSource};
    use scriptw_ir::decode_js_string_utf16;
    let prefix = [
        "/* é 🌱 \u{F0000}D83D ".encode_utf16().collect::<Vec<_>>(),
        vec![0xdc38],
        " */\r\n".encode_utf16().collect(),
    ]
    .concat();
    for callable in [
        "function f() { /* RAW */ return /\\uD83DLOW/; }",
        "function* f() { /* RAW */ return /\\uD83DLOW/; }",
        "async function f() { /* RAW */ return /\\uD83DLOW/; }",
        "async function* f() { /* RAW */ return /\\uD83DLOW/; }",
        "() => { /* RAW */ return /\\uD83DLOW/; }",
        "async () => { /* RAW */ return /\\uD83DLOW/; }",
        "class C { method() { /* RAW */ return /\\uD83DLOW/; } }",
    ] {
        let mut callable_units = Vec::new();
        for part in callable.split_inclusive("RAW") {
            if let Some(part) = part.strip_suffix("RAW") {
                callable_units.extend(part.encode_utf16());
                callable_units.push(0xd83d);
            } else {
                callable_units.extend(part.encode_utf16());
            }
        }
        let low_start = callable_units
            .windows(3)
            .position(|units| units == [76, 79, 87])
            .unwrap();
        callable_units.splice(low_start..low_start + 3, [0xdc38]);
        let source = [
            prefix.clone(),
            vec![40],
            callable_units.clone(),
            ");".encode_utf16().collect(),
        ]
        .concat();
        let parsed = prepare_eval_source_utf16(&source, &EvalParseContext::Indirect).unwrap();
        let program = lower(&ParsedSource::Script(parsed));
        assert!(
            program.is_wasm_supported(),
            "{callable}: {:?}",
            program.diagnostics
        );
        let script = program.script.unwrap();
        let expected = if callable.starts_with("class") {
            let start = callable_units
                .windows(7)
                .position(|units| units == "method(".encode_utf16().collect::<Vec<_>>())
                .unwrap();
            callable_units[start..callable_units.len() - 2].to_vec()
        } else {
            callable_units
        };
        assert!(script.functions.iter().any(|function| matches!(&function.to_string_representation, CallableToStringRepresentation::ExactSource(source) if decode_js_string_utf16(source) == expected)), "actual units absent for {callable}");
    }
}
