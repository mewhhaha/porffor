use scriptw_front::{
    encode_js_string_utf16, parse, parse_script_utf16, prepare_dynamic_function,
    prepare_dynamic_function_utf16, prepare_eval_source, prepare_eval_source_utf16,
    DirectEvalParseContext, EarlyErrorCode, EvalInvocationContext, EvalParseContext,
    FunctionParseKind, ParseCode, ParseDiagnosticPhase, ParseError, ParseOptions,
    JS_STRING_SURROGATE_SENTINEL,
};

const STRICT_GENERATOR: &str = "function* yield() { 'use strict'; (yield 3) + (yield 4); }";
const RESERVED: [&str; 9] = [
    "implements",
    "interface",
    "let",
    "package",
    "private",
    "protected",
    "public",
    "static",
    "yield",
];

fn syntax_error(error: &ParseError) {
    assert_eq!(error.diagnostic().error_type(), Some("SyntaxError"));
}

fn typed_strict_error(error: &ParseError) {
    syntax_error(error);
    assert_eq!(error.diagnostic().phase(), ParseDiagnosticPhase::Early);
    let ParseCode::Early(code) = error.diagnostic().code else {
        panic!("expected typed strict binding rejection: {error:?}");
    };
    assert_eq!(
        code.code(),
        EarlyErrorCode::StrictModeReservedBindingIdentifier
    );
}

fn units(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

#[test]
fn generator_declaration_own_strict_directive_rejects_its_name_in_every_source_route() {
    typed_strict_error(&parse(STRICT_GENERATOR, ParseOptions::script()).unwrap_err());
    typed_strict_error(&parse_script_utf16(&units(STRICT_GENERATOR)).unwrap_err());
    syntax_error(&parse(STRICT_GENERATOR, ParseOptions::module()).unwrap_err());
    for context in [
        EvalParseContext::Indirect,
        EvalParseContext::Direct(DirectEvalParseContext {
            strict_caller: false,
            invocation: EvalInvocationContext::Function,
            private_names: vec![],
        }),
    ] {
        typed_strict_error(&prepare_eval_source(STRICT_GENERATOR.into(), &context).unwrap_err());
        typed_strict_error(
            &prepare_eval_source_utf16(&units(STRICT_GENERATOR), &context).unwrap_err(),
        );
    }
    typed_strict_error(
        &prepare_dynamic_function(FunctionParseKind::Ordinary, &[STRICT_GENERATOR.into()])
            .unwrap_err(),
    );
    typed_strict_error(
        &prepare_dynamic_function_utf16(FunctionParseKind::Ordinary, &[units(STRICT_GENERATOR)])
            .unwrap_err(),
    );
}

#[test]
fn all_callable_forms_recheck_strict_reserved_header_bindings() {
    for name in RESERVED {
        for prefix in ["function", "function*", "async function", "async function*"] {
            for source in [
                format!("{prefix} {name}() {{ 'use strict'; }}"),
                format!("({prefix} {name}() {{ 'use strict'; }});"),
                format!("{prefix} f({name}) {{ 'use strict'; }}"),
                format!("({prefix} f({name}) {{ 'use strict'; }});"),
            ] {
                syntax_error(
                    &parse(&source, ParseOptions::script())
                        .expect_err(&format!("accepted strict header {source:?}")),
                );
            }
        }
        for source in [
            format!("({name}) => {{ 'use strict'; }};"),
            format!("async ({name}) => {{ 'use strict'; }};"),
            format!("({{ method({name}) {{ 'use strict'; }} }});"),
            format!("({{ *method({name}) {{ 'use strict'; }} }});"),
            format!("({{ async method({name}) {{ 'use strict'; }} }});"),
            format!("({{ async *method({name}) {{ 'use strict'; }} }});"),
            format!("({{ set value({name}) {{ 'use strict'; }} }});"),
            format!("class C {{ method({name}) {{}} }}"),
        ] {
            syntax_error(
                &parse(&source, ParseOptions::script())
                    .expect_err(&format!("accepted strict header {source:?}")),
            );
        }
    }
}

#[test]
fn escaped_reserved_bindings_use_decoded_symbols() {
    for name in RESERVED {
        let escaped = name
            .chars()
            .map(|ch| format!("\\u{{{:X}}}", u32::from(ch)))
            .collect::<String>();
        for source in [
            format!("function {escaped}() {{ 'use strict'; }}"),
            format!("function f({escaped}) {{ 'use strict'; }}"),
            format!("function* {escaped}() {{ 'use strict'; }}"),
        ] {
            syntax_error(
                &parse(&source, ParseOptions::script())
                    .expect_err(&format!("accepted decoded strict binding {source:?}")),
            );
            syntax_error(&parse_script_utf16(&units(&source)).unwrap_err());
        }
    }
}

#[test]
fn sloppy_yield_contexts_and_directive_lookalikes_stay_valid() {
    for source in [
        "function* yield() { yield 1; }",
        "function yield(yield) { yield: yield (yield + yield (0)); }",
        "function* g() { function f(yield) { yield (yield + yield (0)); } }",
        "function* g() { (function yield() {}); }",
        "function f() { function* yield() {} }",
        "function* yield() { { 'use strict'; } yield 1; }",
        "function yield(yield) { 'use strict' + ''; return yield; }",
        "function yield(yield) { 'use\\x20strict'; return yield; }",
        "function* yield() { ; 'use strict'; yield 1; }",
        "function f() { 'use strict'; } function* yield() { yield 1; }",
    ] {
        parse(source, ParseOptions::script())
            .unwrap_or_else(|error| panic!("rejected valid scope {source:?}: {error}"));
        prepare_eval_source_utf16(&units(source), &EvalParseContext::Indirect).unwrap();
    }
    for name in RESERVED {
        let source = format!("function {name}({name}) {{ return {name}; }}");
        parse(&source, ParseOptions::script()).unwrap();
    }
}

#[test]
fn property_names_inferred_names_and_computed_keys_are_not_bindings() {
    for source in [
        "({ yield: function() { 'use strict'; } });",
        "({ yield: function*() { 'use strict'; } });",
        "({ yield: async function() { 'use strict'; } });",
        "({ yield: async function*() { 'use strict'; } });",
        "({ yield: () => { 'use strict'; } });",
        "({ yield: async () => { 'use strict'; } });",
        "({ yield() { 'use strict'; } });",
        "({ *yield() { 'use strict'; } });",
        "({ async yield() { 'use strict'; } });",
        "({ async *yield() { 'use strict'; } });",
        "({ [(function yield() {})]() { 'use strict'; } });",
        "({ get [(function yield() {})]() { 'use strict'; return 1; } });",
        "class C { yield() {} *yield() {} }",
        "function f({yield: ordinary}) { return ordinary; }",
    ] {
        parse(source, ParseOptions::script()).unwrap_or_else(|error| {
            panic!("property/display name became a binding {source:?}: {error}")
        });
    }
}

#[test]
fn strict_header_rule_retains_other_existing_early_errors() {
    for source in [
        "function f(a, a) { 'use strict'; }",
        "function f(a = 1) { 'use strict'; }",
        "function f(eval) { 'use strict'; }",
        "function arguments() { 'use strict'; }",
        "function* g(yield) {}",
        "function* g() { function yield() {} }",
        "function* g() { yield: 1 }",
    ] {
        syntax_error(&parse(source, ParseOptions::script()).unwrap_err());
    }
}

#[test]
fn strict_binding_spans_follow_utf8_and_actual_utf16_positions() {
    let utf8 = format!("/*😀{JS_STRING_SURROGATE_SENTINEL}*/\r\n{STRICT_GENERATOR}");
    let error = parse(&utf8, ParseOptions::script()).unwrap_err();
    typed_strict_error(&error);
    let offset = utf8.find("yield").unwrap();
    let span = error.diagnostic().span.unwrap();
    assert_eq!((span.start, span.end), (offset, offset + 1));

    let mut prefix = units(&format!("/*😀{JS_STRING_SURROGATE_SENTINEL}"));
    prefix.push(0xD800);
    prefix.extend(units("*/\r\nfunction* "));
    let offset = encode_js_string_utf16(&prefix).len();
    let mut source = prefix;
    source.extend(units("yield() { 'use strict'; yield 1; }"));
    for error in [
        parse_script_utf16(&source).unwrap_err(),
        prepare_eval_source_utf16(&source, &EvalParseContext::Indirect).unwrap_err(),
    ] {
        typed_strict_error(&error);
        let span = error.diagnostic().span.unwrap();
        assert_eq!((span.start, span.end), (offset, offset + 1));
    }
}

#[test]
fn original_generator_syntax_inputs_keep_all_invalid_and_valid_controls() {
    let invalid = [
        "function* g() { yield\n* foo }",
        "function* g() { yield ? yield : yield }",
        STRICT_GENERATOR,
        "function f() { 'use strict'; yield: 1 }",
        "function* g() { yield: 1 }",
        "function* g() { yield = 10; }",
        "function* g() { yield 3 + yield 4; }",
        "function f() { 'use strict'; var yield = 13; }",
        "function f() { (function* yield() {}); }",
        "function* g() { (function* yield() {}); }",
        "function* g() { function yield() {} }",
        "function* g() { function* yield() {} }",
        "function* g(yield) { yield (10); }",
    ];
    for (ordinal, source) in invalid.iter().enumerate() {
        let error = if ordinal < 2 {
            prepare_dynamic_function_utf16(FunctionParseKind::Ordinary, &[units(source)])
        } else {
            prepare_eval_source_utf16(&units(source), &EvalParseContext::Indirect)
        }
        .expect_err(&format!("accepted original invalid #{ordinal}: {source:?}"));
        syntax_error(&error);
    }
    let original =
        include_str!("../../../test262/vendor/test262/test/staging/sm/generators/syntax.js");
    parse_script_utf16(&units(original)).expect("unchanged original outer fixture remains valid");
}

#[test]
fn contextual_let_names_are_valid_for_all_sloppy_function_expression_goals() {
    for prefix in ["function", "function*", "async function", "async function*"] {
        for name in ["let", "\\u006cet"] {
            let source = format!("var selected = ({prefix} {name}() {{}});");
            parse(&source, ParseOptions::script()).unwrap();
            parse_script_utf16(&units(&source)).unwrap();
            for context in [
                EvalParseContext::Indirect,
                EvalParseContext::Direct(DirectEvalParseContext {
                    strict_caller: false,
                    invocation: EvalInvocationContext::Function,
                    private_names: vec![],
                }),
            ] {
                prepare_eval_source(source.clone(), &context).unwrap();
                prepare_eval_source_utf16(&units(&source), &context).unwrap();
            }
            prepare_dynamic_function(FunctionParseKind::Ordinary, &[source.clone()]).unwrap();
            prepare_dynamic_function_utf16(FunctionParseKind::Ordinary, &[units(&source)]).unwrap();
        }
    }
}

#[test]
fn contextual_let_expression_names_retain_inherited_and_own_strict_rejections() {
    for prefix in ["function", "function*", "async function", "async function*"] {
        for name in ["let", "\\u006cet"] {
            let source = format!("({prefix} {name}() {{}});");
            let strict_source = format!("'use strict'; {source}");
            syntax_error(&parse(&strict_source, ParseOptions::script()).unwrap_err());
            syntax_error(&parse_script_utf16(&units(&strict_source)).unwrap_err());
            syntax_error(&parse(&source, ParseOptions::module()).unwrap_err());
            let strict_caller = EvalParseContext::Direct(DirectEvalParseContext {
                strict_caller: true,
                invocation: EvalInvocationContext::Function,
                private_names: vec![],
            });
            syntax_error(&prepare_eval_source(source, &strict_caller).unwrap_err());
            let own_strict = format!("({prefix} {name}() {{ 'use strict'; }});");
            typed_strict_error(&parse(&own_strict, ParseOptions::script()).unwrap_err());
            typed_strict_error(&parse_script_utf16(&units(&own_strict)).unwrap_err());
            typed_strict_error(
                &prepare_eval_source_utf16(&units(&own_strict), &EvalParseContext::Indirect)
                    .unwrap_err(),
            );
            typed_strict_error(
                &prepare_dynamic_function(FunctionParseKind::Ordinary, &[own_strict.clone()])
                    .unwrap_err(),
            );
        }
    }
}

#[test]
fn contextual_name_admission_preserves_anonymous_and_reserved_name_boundaries() {
    for prefix in ["function", "function*", "async function", "async function*"] {
        parse(&format!("({prefix}() {{}});"), ParseOptions::script()).unwrap();
        for name in [
            "class", "const", "enum", "export", "extends", "import", "super",
        ] {
            syntax_error(
                &parse(
                    &format!("({prefix} {name}() {{}});"),
                    ParseOptions::script(),
                )
                .unwrap_err(),
            );
        }
    }
    for source in [
        "(function* yield() {});",
        "(async function await() {});",
        "(async function* yield() {});",
        "(async function* await() {});",
    ] {
        syntax_error(&parse(source, ParseOptions::script()).unwrap_err());
    }
}
