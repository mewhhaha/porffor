use lila_front::{parse, ParseOptions};

#[test]
fn line_terminators_separated_by_comments_are_skipped_between_function_tokens() {
    for terminator in ["\n", "\r", "\r\n", "\u{2028}", "\u{2029}"] {
        for function in ["function", "function*", "async function", "async function*"] {
            let source = format!(
                "{function}{terminator}// name{terminator}f{terminator}// parameters{terminator}({terminator}// body{terminator}){terminator}// block{terminator}{{}}"
            );
            assert!(parse(&source, ParseOptions::script()).is_ok(), "{source:?}");
        }
    }
}

#[test]
fn comments_do_not_hide_restricted_line_terminators() {
    for terminator in ["\n", "\r", "\r\n", "\u{2028}", "\u{2029}"] {
        let source = format!("function f() {{ throw{terminator}// comment{terminator}42; }}");
        assert!(parse(source, ParseOptions::script()).is_err());
    }
}
