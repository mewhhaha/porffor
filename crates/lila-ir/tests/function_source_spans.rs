use lila_front::{parse, ParseOptions};
use lila_ir::{lower, CallableToStringRepresentation};

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
