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
