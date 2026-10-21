use super::*;

#[test]
fn diagnostic_coordinates_use_code_points_and_encoded_byte_ranges() {
    let source = "a😀\r\né\u{2028}z";
    for (line, column, start, end) in [
        (1, 1, 0, 1),
        (1, 2, 1, 5),
        (1, 3, 5, 6),
        (2, 1, 7, 9),
        (2, 2, 9, 12),
        (3, 1, 12, 13),
        (3, 2, 13, 13),
    ] {
        let position = boa_ast::Position::new(line, column);
        assert_eq!(
            source_span_at_position(source, position),
            Some(SourceSpan { start, end })
        );
        assert_eq!(
            source_span_utf16_at_position(&source.encode_utf16().collect::<Vec<_>>(), position),
            Some(SourceSpan { start, end })
        );
    }
    assert_eq!(byte_offset_for_line_col(source, 0, 1), None);
    assert_eq!(byte_offset_for_line_col(source, 1, 4), None);
    assert_eq!(byte_offset_for_line_col(source, 4, 1), None);

    let units = [0xD800, b'x' as u16, 0xD83D, 0xDE00];
    let first_end = encode_js_string_utf16(&units[..1]).len();
    assert_eq!(
        source_span_utf16_at_position(&units, boa_ast::Position::new(1, 1)),
        Some(SourceSpan {
            start: 0,
            end: first_end
        })
    );
    assert_eq!(
        source_span_utf16_at_position(&units, boa_ast::Position::new(1, 3)),
        Some(SourceSpan {
            start: first_end + 1,
            end: first_end + 5
        })
    );
    let eof = encode_js_string_utf16(&units).len();
    assert_eq!(
        source_span_utf16_at_position(&units, boa_ast::Position::new(1, 4)),
        Some(SourceSpan {
            start: eof,
            end: eof
        })
    );
}

#[test]
fn syntax_error_reports_structured_diagnostic_with_byte_span_when_available() {
    let err =
        parse("let x = ;", ParseOptions::script()).expect_err("broken initializer should fail");
    assert_eq!(
        err.diagnostic().kind(),
        ParseDiagnosticKind::MalformedJavaScript
    );
    assert_eq!(err.diagnostic().phase(), ParseDiagnosticPhase::Parse);
    assert_eq!(err.diagnostic().error_type(), Some("SyntaxError"));
    assert_eq!(err.diagnostic().code, ParseCode::Malformed);
    assert!(
        err.diagnostic().span.is_some(),
        "diagnostic should carry Boa's source position when available: {err:?}"
    );
}

#[test]
fn nul_outside_a_literal_or_comment_reports_a_malformed_diagnostic_with_span() {
    let err = parse("let x = 0;\0", ParseOptions::script())
        .expect_err("NUL is not a valid token between statements");
    assert_eq!(
        err.diagnostic().kind(),
        ParseDiagnosticKind::MalformedJavaScript
    );
    assert_eq!(err.diagnostic().code, ParseCode::Malformed);
    assert_eq!(
        err.diagnostic().span,
        Some(SourceSpan { start: 10, end: 11 })
    );
}

/// Ledger L1's injection channel, closed.
///
/// boa renders a `TokenKind::StringLiteral` as its raw contents
/// (`boa_parser/src/lexer/token.rs:313`) and interpolates the found token
/// into `Error::Unexpected` / `Error::Expected`, so a program can put a
/// whole `ContainsAll` fragment set of the one table into the message boa produces for an
/// ordinary syntax error. `classify_parse_failure` refuses the two
/// interpolating shapes, so this stays `Malformed` — a syntax error we do
/// not model — rather than becoming a forged `E_ILLEGAL_BREAK`.
#[test]
fn user_source_text_cannot_forge_an_early_error_classification() {
    let err = parse(
        "var x = \"illegal break statement\" \"y\";",
        ParseOptions::script(),
    )
    .expect_err("two adjacent string literals are a syntax error");
    assert_eq!(err.diagnostic().code, ParseCode::Malformed, "{err}");
}
