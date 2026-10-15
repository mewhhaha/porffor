use lila_front::{ParseError, ParseOptions, parse, parse_script_utf16};

fn syntax_error(error: ParseError, source: &str) {
    assert_eq!(
        error.diagnostic().error_type(),
        Some("SyntaxError"),
        "wrong rejection for {source:?}: {error}"
    );
}

fn rejects_under_all_goals(source: &str) {
    for source in [source.to_owned(), format!("'use strict'; {source}")] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            syntax_error(
                parse(&source, options)
                    .expect_err(&format!("accepted invalid object rest pattern: {source}")),
                &source,
            );
        }
        syntax_error(
            parse_script_utf16(&source.encode_utf16().collect::<Vec<_>>()).expect_err(&format!(
                "accepted invalid UTF-16 object rest pattern: {source}"
            )),
            &source,
        );
    }
}

fn accepts_under_all_goals(source: &str) {
    for source in [source.to_owned(), format!("'use strict'; {source}")] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(&source, options)
                .unwrap_or_else(|error| panic!("rejected valid source {source:?}: {error}"));
        }
        parse_script_utf16(&source.encode_utf16().collect::<Vec<_>>())
            .unwrap_or_else(|error| panic!("rejected valid UTF-16 source {source:?}: {error}"));
    }
}

#[test]
fn pinned_object_assignment_rest_before_comma_is_a_syntax_error() {
    // Keep the actual source shape, including the unparenthesized comma expression.
    let source = include_str!(
        "../../../test262/vendor/test262/test/language/expressions/assignment/dstr/obj-rest-before-comma-invalid.js"
    );
    rejects_under_all_goals(source);
}

#[test]
fn trailing_commas_reject_bare_computed_default_and_access_rest_patterns() {
    for source in [
        "var rest; 0, {...rest,} = {};",
        "var rest; ({...rest,} = source);",
        "var value, rest; ({value, ...rest,} = source);",
        "var value, rest; ({[key()]: value, ...rest,} = source);",
        "var value, rest; ({value = fallback(), ...rest,} = source);",
        "var value, rest; ({[key()]: value = fallback(), ...rest,} = source);",
        "var target = {}; ({...target.rest,} = source);",
        "var target = {}; ({...target[key()],} = source);",
        "var target = {}; ({...((target.rest)),} = source);",
        "var rest; ({...rest /* before comma */, /* before brace */} = source);",
        "var rest; ({...rest\n,\n} = source);",
    ] {
        rejects_under_all_goals(source);
    }
}

#[test]
fn nested_object_rest_trailing_commas_cannot_become_patterns() {
    for source in [
        "var rest; ({outer: {...rest,}} = source);",
        "var rest; ({[key()]: {...rest,}} = source);",
        "var rest; ({outer: {...rest,} = fallback()} = source);",
        "var rest; ({[key()]: {...rest,} = fallback()} = source);",
        "var rest; [{...rest,}] = source;",
        "var rest; [{...rest,} = fallback()] = source;",
        "var rest; [...{...rest,}] = source;",
        "var rest; ({outer: [{...rest,}]} = source);",
    ] {
        rejects_under_all_goals(source);
    }
}

#[test]
fn iterable_loop_heads_share_the_object_pattern_conversion_rule() {
    for source in [
        "var rest; for ({...rest,} of source) {}",
        "var rest; for ({...rest,} in source) {}",
        "var value, rest; for ({[key()]: value = fallback(), ...rest,} of source) {}",
        "var rest; for ({outer: {...rest,}} in source) {}",
        "var rest; for ([{...rest,}] of source) {}",
    ] {
        rejects_under_all_goals(source);
    }
}

#[test]
fn object_expressions_keep_spreads_before_properties_and_trailing_commas() {
    for source in [
        "var source = {}; ({...source,});",
        "var source = {}; 0, {...source,};",
        "var source = {}; ({value: 1, ...source,});",
        "var source = {}; ({...source, value: 1});",
        "var source = {}; ({...source, value: 1,});",
        "var source = {}; ({...source, ...source,});",
        "var source = {}; ({outer: {...source,},});",
        "var source = {}; ({[key()]: {...source,},});",
        "var source = {}; [{...source,}];",
        "var source = {}; ({...source /* before comma */, /* before brace */});",
        "var source = {}; ({...source\n,\n});",
        "var source = {}; (() => ({...source,}));",
    ] {
        accepts_under_all_goals(source);
    }
}

#[test]
fn valid_assignment_patterns_keep_rest_and_ordinary_trailing_commas() {
    for source in [
        "var rest; ({...rest} = source);",
        "var value, rest; ({value, ...rest} = source);",
        "var value, rest; ({[key()]: value = fallback(), ...rest} = source);",
        "var target = {}; ({...target.rest} = source);",
        "var rest; ({outer: {...rest},} = source);",
        "var rest; ({[key()]: {...rest} = fallback(),} = source);",
        "var rest; [{...rest},] = source;",
        "var value; ({value,} = source);",
        "var value; ({[key()]: value = fallback(),} = source);",
        "var value; ({value = ({...source,}),} = source);",
        "var value, rest; ({value = ({...source,}), ...rest} = source);",
        "var value, rest; ({[({...source,}).key]: value, ...rest} = source);",
        "var rest; for ({...rest} of source) {}",
        "var rest; for ({...rest} in source) {}",
    ] {
        accepts_under_all_goals(source);
    }
}

#[test]
fn rest_must_still_be_last_and_target_a_simple_assignment_location() {
    for source in [
        "var value, rest; ({...rest, value} = source);",
        "var value, rest; ({...rest, [key()]: value} = source);",
        "var rest, other; ({...rest, ...other} = source);",
        "var rest; ({...rest = fallback()} = source);",
        "var rest; ({...{rest}} = source);",
        "var rest; ({...[rest]} = source);",
    ] {
        rejects_under_all_goals(source);
    }
}
