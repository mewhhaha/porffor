use lila_front::{parse, ParseCode, ParseOptions};

#[test]
fn literal_and_comment_nuls_are_accepted_in_both_parse_goals() {
    for source in [
        "'a\0b';",
        "\"a\0b\";",
        "`a\0b`;",
        "tag`a\0b`;",
        "/a\0b/;",
        "// before\0after\n23;",
        "/* before\0after */23;",
    ] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            parse(source, options).unwrap_or_else(|error| panic!("{source:?}: {error}"));
        }
    }
}

#[test]
fn nuls_between_tokens_do_not_truncate_the_source_or_become_whitespace() {
    for source in ["\0", "23;\0", "var\0value = 1;", "'ok';\0let = ;"] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            let error = parse(source, options).expect_err("NUL cannot separate tokens");
            assert_eq!(error.diagnostic().code, ParseCode::Malformed, "{source:?}");
        }
    }
}

#[test]
fn a_nul_in_a_literal_or_comment_does_not_hide_later_invalid_syntax() {
    for source in ["'\0'; let = ;", "`\0`; let = ;", "/*\0*/ let = ;"] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            assert!(parse(source, options).is_err(), "{source:?}");
        }
    }
}
