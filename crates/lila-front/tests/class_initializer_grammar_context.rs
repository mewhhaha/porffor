use lila_front::{parse, ParseOptions};

#[test]
fn script_field_initializers_read_await_as_an_identifier_across_callable_contexts() {
    let fields = r#"
        value = await;
        #private = await;
        static value = await;
        static #staticPrivate = await;
        accessor item = await;
        accessor #item = await;
        static accessor other = await;
        static accessor #other = await;
        arrow = () => await;
    "#;
    for wrapper in [
        "function* make()",
        "async function make()",
        "async function* make()",
    ] {
        let source = format!("var await = 7; {wrapper} {{ return class {{ {fields} }}; }}");
        for directive in ["", "'use strict';"] {
            parse(format!("{directive}{source}"), ParseOptions::script()).unwrap_or_else(|error| {
                panic!("field initializer owns its grammar: {error}\n{source}")
            });
        }
    }
}

#[test]
fn field_initializers_do_not_suspend_the_enclosing_callable() {
    for (wrapper, field) in [
        ("async function make()", "value = await 1;"),
        ("async function make()", "#value = await 1;"),
        ("async function make()", "static accessor value = await 1;"),
        ("async function* make()", "accessor #value = await 1;"),
        ("function* make()", "value = yield 1;"),
        ("function* make()", "static #value = yield;"),
        ("function* make()", "accessor value = yield 1;"),
        ("async function* make()", "static accessor #value = yield;"),
    ] {
        let source = format!("{wrapper} {{ return class {{ {field} }}; }}");
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let error = parse(&source, options).expect_err("field initializer cannot suspend");
            assert_eq!(
                error.diagnostic().error_type(),
                Some("SyntaxError"),
                "{source}"
            );
        }
    }
    for source in [
        "async () => class { value = await 1 };",
        "class C { value = await; }",
        "async function make() { return class { accessor #value = await; }; }",
    ] {
        let error = parse(source, ParseOptions::module())
            .expect_err("Module goal still reserves await in initializer identifiers");
        assert_eq!(error.diagnostic().error_type(), Some("SyntaxError"));
    }
}

#[test]
fn computed_names_and_nested_callables_keep_their_own_suspension_grammar() {
    for source in [
        "async function make() { return class { [await key()] = 1; static accessor [await key()] = 2; }; }",
        "function* make() { return class { [yield 'key'] = 1; static accessor [yield 'other'] = 2; }; }",
        "async function make() { return class { value = async () => await 1; #value = function* () { yield 2; }; }; }",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options)
                .unwrap_or_else(|error| panic!("suspension stays in its owning context: {error}\n{source}"));
        }
    }
}
