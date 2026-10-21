use super::*;

#[test]
fn generator_declaration_parameter_yield_rejects_under_both_goals() {
    for source in [
        "function* g(x = yield) {}",
        "async function* g(x = yield) {}",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("YieldExpression in generator declaration parameters should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::GeneratorDeclarationParametersContainYield),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn generator_declaration_parameter_yield_boundaries_remain_valid() {
    for source in [
        "function* g(x = 1) { yield x; }",
        "async function* g(x = 1) { yield x; }",
        "function* outer(x = function*(){ yield 1; }) {}",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options)
                .expect("generator bodies and nested generators are Contains boundaries");
        }
    }
}

#[test]
fn generator_expression_parameter_yield_rejects_under_both_goals() {
    for source in [
        "0, function*(x = yield) {};",
        "const g = function* named(x = yield) {};",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("YieldExpression in generator expression parameters should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::GeneratorExpressionParametersContainYield),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn generator_expression_parameter_yield_boundaries_remain_valid() {
    for source in [
        "const g = function*(x = 1) { yield x; };",
        "const outer = function*(x = function*(){ yield 1; }) {};",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options)
                .expect("generator bodies and nested generators are Contains boundaries");
        }
    }
}

#[test]
fn async_generator_expression_parameter_yield_rejects_under_both_goals() {
    for source in [
        "(async function*(x = yield) {});",
        "const g = async function* named(x = yield) {};",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("YieldExpression in async generator expression parameters should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::AsyncGeneratorExpressionParametersContainYield),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn async_generator_expression_parameter_yield_boundaries_remain_valid() {
    for source in [
        "const g = async function*(x = 1) { yield x; };",
        "const outer = async function*(x = function*(){ yield 1; }) {};",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options)
                .expect("async-generator bodies and nested generators are Contains boundaries");
        }
    }
}

#[test]
fn async_generator_expression_parameter_await_rejects_under_both_goals() {
    for source in [
        "(async function*(x = await 1) {});",
        "const g = async function* named(x = await 1) {};",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("AwaitExpression in async generator expression parameters should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::AsyncGeneratorExpressionParametersContainAwait),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn async_generator_expression_parameter_await_boundaries_remain_valid() {
    for source in [
        "const g = async function*(x = 1) { await 1; yield x; };",
        "const outer = async function*(x = async function(){ await 1; }) {};",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options).expect(
                "async-generator bodies and nested async functions are Contains boundaries",
            );
        }
    }
}

#[test]
fn async_declaration_parameter_await_rejects_under_both_goals() {
    for source in [
        "async function f(x = await 1) {}",
        "async function* g(x = await 1) {}",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("AwaitExpression in async declaration parameters should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::AsyncDeclarationParametersContainAwait),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn async_declaration_parameter_await_boundaries_remain_valid() {
    for source in [
        "async function f(x = 1) { await 1; }",
        "async function* g(x = 1) { await 1; yield x; }",
        "async function outer(x = async function(){ await 1; }) {}",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options)
                .expect("async bodies and nested async functions are Contains boundaries");
        }
    }
}

#[test]
fn generator_method_parameter_yield_rejects_under_both_goals() {
    for source in [
        "({ *m(x = yield) {} });",
        "class C { *m(x = yield) {} }",
        "class C { static *m(x = yield) {} }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("YieldExpression in generator method parameters should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::GeneratorMethodParametersContainYield),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn async_generator_method_parameter_yield_rejects_under_both_goals() {
    for source in [
        "({ async *m(x = yield) {} });",
        "class C { async *m(x = yield) {} }",
        "class C { static async *m(x = yield) {} }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("YieldExpression in async-generator method parameters should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::AsyncGeneratorMethodParametersContainYield),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn async_generator_method_parameter_await_rejects_under_both_goals() {
    for source in [
        "({ async *m(x = await 1) {} });",
        "class C { async *m(x = await 1) {} }",
        "class C { static async *m(x = await 1) {} }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("AwaitExpression in async-generator method parameters should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::AsyncGeneratorMethodParametersContainAwait),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn generator_method_parameter_contains_boundaries_remain_valid() {
    for source in [
        "({ *m(x = 1) { yield x; } });",
        "class C { *m(x = function*(){ yield 1; }) {} }",
        "({ async *m(x = 1) { yield x; await 1; } });",
        "class C { async *m(x = function*(){ yield 1; }) {} }",
        "class C { static async *m(x = async function(){ await 1; }) {} }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options)
                .expect("method bodies and nested functions are Contains boundaries");
        }
    }
}

#[test]
fn arrow_parameter_yield_rejects_under_both_goals() {
    for source in [
        "function* outer() { (x = yield) => x; }",
        "function* outer() { async (x = yield) => x; }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("YieldExpression in arrow parameters should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::ArrowParametersContainYield),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn arrow_parameter_await_rejects_under_both_goals() {
    for source in [
        "async function outer() { (x = await 1) => x; }",
        "const f = async (x = await 1) => x;",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("AwaitExpression in arrow parameters should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::ArrowParametersContainAwait),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn arrow_parameter_contains_boundaries_remain_valid() {
    for source in [
        "const f = (x = function*(){ yield 1; }) => x;",
        "function* outer() { async (x = function*(){ yield 1; }) => x; }",
        "const f = async (x = async function(){ await 1; }) => await x;",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options)
                .expect("arrow bodies and nested functions are Contains boundaries");
        }
    }

    parse(
        "var yield = 1; const f = async (x = yield) => x;",
        ParseOptions::script(),
    )
    .expect("a sloppy-script yield identifier must not enable Yield grammar globally");
}

#[test]
fn async_function_expression_parameter_await_rejects_under_both_goals() {
    for source in [
        "(async function(x = await 1) {});",
        "const f = async function named(x = await 1) {};",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("AwaitExpression in async function expression parameters should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::AsyncFunctionExpressionParametersContainAwait),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn async_method_parameter_await_rejects_under_both_goals() {
    for source in [
        "({ async m(x = await 1) {} });",
        "class C { async m(x = await 1) {} }",
        "class C { static async m(x = await 1) {} }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("AwaitExpression in async method parameters should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::AsyncMethodParametersContainAwait),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn async_expression_and_method_parameter_await_boundaries_remain_valid() {
    for source in [
        "const f = async function(x = async function(){ await 1; }) { await 1; };",
        "const o = { async m(x = async function(){ await 1; }) { await 1; } };",
        "class C { static async m(x = async function(){ await 1; }) { await 1; } }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options)
                .expect("async bodies and nested async functions are Contains boundaries");
        }
    }
}

#[test]
fn callable_non_simple_parameters_with_use_strict_reject_at_every_producer() {
    // One source per remaining fixed-message site. Shared declaration and
    // method parsers need one caller here; the pinned cohort covers their
    // other grammar forms without duplicating this source-level contract.
    const PRODUCER_SOURCES: [&str; 16] = [
        "function f(a = 0) { 'use strict'; }",
        "(function(a = 0) { 'use strict'; });",
        "(function*(a = 0) { 'use strict'; });",
        "(async function(a = 0) { 'use strict'; });",
        "(async function*(a = 0) { 'use strict'; });",
        "(a = 0) => { 'use strict'; };",
        "async (a = 0) => { 'use strict'; };",
        "({ set x(a = 0) { 'use strict'; } });",
        "({ x(a = 0) { 'use strict'; } });",
        "({ *x(a = 0) { 'use strict'; } });",
        "({ async *x(a = 0) { 'use strict'; } });",
        "({ async x(a = 0) { 'use strict'; } });",
        "class C { set #x(a = 0) { 'use strict'; } }",
        "class C { set x({ a }) { 'use strict'; } }",
        "class C { #x(a = 0) { 'use strict'; } }",
        "class C { x(a = 0) { 'use strict'; } }",
    ];
    for source in PRODUCER_SOURCES {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("non-simple parameters plus an own directive should fail");
            assert_eq!(
                err.diagnostic().phase(),
                ParseDiagnosticPhase::Early,
                "{source:?}: {err:?}"
            );
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::CallableNonSimpleParametersContainUseStrict),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn duplicate_formal_parameter_wordings_report_one_early_error() {
    for source in [
        "function duplicate(a = 0, a) {}",
        "(a, a) => 0",
        "class Duplicate { method(a, a) {} }",
    ] {
        let err = parse(source, ParseOptions::script())
            .expect_err("duplicate formal parameters should fail in this context");
        assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
        assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::DuplicateFormalParameter),
            "{source:?}: {err:?}"
        );
    }
}

#[test]
fn duplicate_formal_parameter_fixture_preserves_the_sloppy_script_exception() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/../tests/fixtures/duplicate_formal_parameters.js"
    ));
    parse(source, ParseOptions::script())
        .expect("sloppy ordinary function with a simple duplicate list should parse");

    let err = parse(source, ParseOptions::module())
        .expect_err("module code is strict, so duplicate formal parameters should fail");
    assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
    assert_eq!(
        err.diagnostic().code,
        early(EarlyErrorCode::DuplicateFormalParameter),
        "{err:?}"
    );
}

#[test]
fn duplicate_catch_parameter_fixture_reports_one_early_error_in_both_goals() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/../tests/fixtures/duplicate_catch_parameter.js"
    ));
    for options in [ParseOptions::script(), ParseOptions::module()] {
        let err = parse(source, options)
            .expect_err("duplicate BoundNames in a catch parameter should fail");
        assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
        assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
        assert_eq!(
            err.diagnostic().code,
            early(EarlyErrorCode::DuplicateCatchParameter),
            "{err:?}"
        );
        assert!(err.diagnostic().span.is_some(), "{err:?}");
    }
}

#[test]
fn class_static_block_arguments_rejections_cover_both_forms_and_goals() {
    for source in [
        r"class C { static { (class { [argument\u0073]() {} }); } }",
        "const C = class { static { (() => arguments); } };",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("lexical arguments use in a class static block should fail");
            assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::ClassStaticBlockContainsArguments),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn class_static_block_arguments_stop_at_function_and_method_boundaries() {
    for source in [
        r#"class C {
                static {
                    function nested(value = arguments) { return arguments; }
                }
            }"#,
        r#"const C = class {
                static {
                    class Nested {
                        method(value = arguments) { return arguments; }
                    }
                }
            };"#,
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options)
                .expect("ordinary function and method bodies own their arguments bindings");
        }
    }
}

#[test]
fn class_static_block_await_rule_does_not_absorb_declaration_parameter_errors() {
    assert_eq!(
        classify_parse_failure(
            "invalid await usage in generator function parameters at line 1, col 1"
        )
        .map(ParseClassified::code),
        Some(EarlyErrorCode::AsyncDeclarationParametersContainAwait)
    );
}

#[test]
fn class_field_arguments_rejections_cover_all_field_forms_and_goals() {
    for source in [
        "class C { value = arguments; }",
        "const C = class { static value = arguments; };",
        "class C { #value = arguments; }",
        "const C = class { static #value = arguments; };",
        "class C { accessor value = arguments; }",
        "const C = class { static accessor value = arguments; };",
        "class C { accessor #value = arguments; }",
        "const C = class { static accessor #value = arguments; };",
        "class C { value = () => arguments; }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let err = parse(source, options)
                .expect_err("lexical arguments use in a class field should fail");
            assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Early);
            assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
            assert_eq!(
                err.diagnostic().code,
                early(EarlyErrorCode::ClassFieldContainsArguments),
                "{source:?}: {err:?}"
            );
            assert!(err.diagnostic().span.is_some(), "{source:?}: {err:?}");
        }
    }
}

#[test]
fn class_field_arguments_stop_at_function_and_method_boundaries() {
    for source in [
        "class C { value = function () { return arguments; }; }",
        "const C = class { static value = async function () { return arguments; }; };",
        "class C { #value = function* () { yield arguments; }; }",
        "const C = class { static #value = async function* () { yield arguments; }; };",
        "class C { accessor value = ({ method() { return arguments; } }); }",
        "const C = class { static accessor #value = ({ get value() { return arguments; } }); };",
        "class C { value = ({ arguments: 1, ['arguments']: 2 }); }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options).expect(
                "nested functions, methods and property names own no lexical arguments use",
            );
        }
    }
}
