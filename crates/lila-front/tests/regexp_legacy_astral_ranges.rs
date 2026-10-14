use lila_front::{parse, ParseOptions};

fn assert_pattern_syntax(pattern: &str, flags: &str, valid: bool) {
    let source = format!("/{pattern}/{flags};");
    for options in [ParseOptions::script(), ParseOptions::module()] {
        let parsed = parse(&source, options);
        if valid {
            let parsed = parsed.unwrap_or_else(|error| panic!("{source}: {error}"));
            assert_eq!(parsed.source().source_text, source);
        } else {
            assert!(
                parsed.is_err(),
                "invalid range or escape accepted: {source}"
            );
        }
    }
}

#[test]
fn legacy_raw_and_escaped_astral_ranges_use_code_unit_endpoints() {
    for flags in ["", "i", "dgm"] {
        for pattern in [
            r"[😀-\uFFFF]",
            r"[\uD83D\uDE00-\uFFFF]",
            "[a-😀]",
            r"[a-\uD83D\uDE00]",
            r"[\uD83D-\uDE00]",
            r"[\😀]",
        ] {
            assert_pattern_syntax(pattern, flags, true);
        }
        for pattern in [
            "[😀-😁]",
            r"[\uD83D\uDE00-\uD83D\uDE01]",
            "[a-😀-z]",
            r"[a-\uD83D\uDE00-z]",
            r"[\uE000-😀]",
            r"[\uE000-\uD83D\uDE00]",
            "[z-a]",
        ] {
            assert_pattern_syntax(pattern, flags, false);
        }
    }
}

#[test]
fn unicode_range_validation_keeps_scalar_endpoints() {
    for flags in ["u", "ui", "v", "vi"] {
        for pattern in [
            "[😀-😁]",
            r"[\uD83D\uDE00-\uD83D\uDE01]",
            r"[\u{1F600}-\u{1F601}]",
            r"[\uE000-😀]",
            r"[\uD800-\uD801]",
        ] {
            assert_pattern_syntax(pattern, flags, true);
        }
        for pattern in [
            r"[😀-\uFFFF]",
            r"[\uD83D\uDE00-\uFFFF]",
            "[😁-😀]",
            r"[\uD83D\uDE01-\uD83D\uDE00]",
            r"[\😀]",
            r"[\u{110000}]",
            "[z-a]",
        ] {
            assert_pattern_syntax(pattern, flags, false);
        }
    }
}

#[test]
fn unpaired_escape_lookahead_preserves_the_following_range_atom() {
    for flags in ["", "i", "u", "ui", "v", "vi"] {
        assert_pattern_syntax(r"[\uD800\u0061-\u0062]", flags, true);
        assert_pattern_syntax(r"[\uD800\u0062-a]", flags, false);
        assert_pattern_syntax(r"[\uD800\uD802-\uD801]", flags, false);
    }
    assert_pattern_syntax(r"[z-\u{61}]", "", false);
    assert_pattern_syntax(r"[z-\u{61}]", "u", false);
    assert_pattern_syntax(r"[a-\u{61}]", "", true);
    assert_pattern_syntax(r"[a-\u{61}]", "u", true);
}

#[test]
fn capture_names_keep_unicode_identifier_grammar_in_legacy_patterns() {
    for flags in ["", "i", "u", "v"] {
        for pattern in [
            r"(?<𐐀>a)\k<𐐀>",
            r"(?<\uD801\uDC00>a)\k<\uD801\uDC00>",
            r"(?<\u{10400}>a)\k<\u{10400}>",
        ] {
            assert_pattern_syntax(pattern, flags, true);
        }
        for pattern in [r"(?<😀>a)", r"(?<\uD800>a)", r"(?<\uD83D\uDE00>a)"] {
            assert_pattern_syntax(pattern, flags, false);
        }
    }
}

#[test]
fn utf16_prepared_sources_keep_raw_pattern_units_and_their_parser_owner() {
    use boa_ast::{Expression, Statement, StatementListItem};
    use lila_front::{prepare_eval_source_utf16, EvalParseContext, SourceTextEncoding};
    let high = [0xd83d];
    let low = [0xdc38];
    for flags in ["", "u", "v"] {
        for pattern in [
            [r"\uD83D".encode_utf16().collect::<Vec<_>>(), low.to_vec()].concat(),
            [high.to_vec(), r"\uDC38".encode_utf16().collect()].concat(),
            [
                vec![91],
                high.to_vec(),
                r"\uDC38".encode_utf16().collect(),
                vec![93],
            ]
            .concat(),
        ] {
            let source = [
                vec![47],
                pattern.clone(),
                format!("/{flags};").encode_utf16().collect(),
            ]
            .concat();
            let parsed = prepare_eval_source_utf16(&source, &EvalParseContext::Indirect).unwrap();
            assert_eq!(parsed.encoding, SourceTextEncoding::EncodedUtf16);
            assert_eq!(parsed.compiler_source_utf16(), source);
            parsed.with_compiler_session(|script, interner| {
                let StatementListItem::Statement(statement) = &script.statements().statements()[0]
                else {
                    panic!("literal statement");
                };
                let Statement::Expression(Expression::RegExpLiteral(regexp)) = statement.as_ref()
                else {
                    panic!("literal regexp");
                };
                assert_eq!(interner.resolve_expect(regexp.pattern()).utf16(), pattern);
            });
            let script = lila_front::parse_script_utf16(&source).unwrap();
            assert_eq!(script.compiler_source_utf16(), source);
        }
    }
}

#[test]
fn utf16_error_spans_follow_actual_columns_and_literal_marker_text() {
    use lila_front::{
        encode_js_string_utf16, parse_script_utf16, ParseDiagnosticKind, SourceTextEncoding,
    };
    let marker = "\u{F0000}D83D";
    let ordinary = format!("var text = '{marker}';");
    let parsed = parse(&ordinary, ParseOptions::script()).unwrap();
    assert_eq!(parsed.source().source_text, ordinary);
    assert_eq!(parsed.source().encoding, SourceTextEncoding::Utf8);
    let lila_front::ParsedSource::Script(parsed) = parsed else {
        unreachable!();
    };
    assert_eq!(
        parsed.compiler_source_utf16(),
        ordinary.encode_utf16().collect::<Vec<_>>()
    );
    for terminator in ["\n", "\r\n", "\u{2028}"] {
        let units = [
            format!("// {marker}").encode_utf16().collect::<Vec<_>>(),
            vec![0xd83d],
            format!("{terminator}var text = '").encode_utf16().collect(),
            vec![0xdc38],
            "'; @;".encode_utf16().collect(),
        ]
        .concat();
        let error = parse_script_utf16(&units).unwrap_err();
        assert_eq!(
            error.diagnostic().kind(),
            ParseDiagnosticKind::MalformedJavaScript
        );
        let span = error
            .diagnostic()
            .span
            .expect("actual unexpected token has a span");
        let encoded = encode_js_string_utf16(&units);
        assert_eq!(&encoded[span.start..span.end], "@");
    }
}
