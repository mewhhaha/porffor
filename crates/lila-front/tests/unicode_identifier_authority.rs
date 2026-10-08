use lila_front::{parse, ParseOptions};

fn escaped_identifier(name: &str) -> String {
    name.chars()
        .map(|ch| format!("\\u{{{:X}}}", u32::from(ch)))
        .collect()
}

#[test]
fn raw_and_escaped_unicode_bindings_and_private_names_use_the_same_grammar() {
    for name in [
        "a\u{0301}", // Mn
        "a\u{093e}", // Mc
        "a\u{203f}", // Pc
        "\u{309b}",  // ID_Start, excluded from XID_Start
        "\u{037a}",  // ID_Start, excluded from XID_Start
        "a\u{200c}\u{200d}",
        "\u{10400}",
        "\u{088f}",  // Unicode 17 start supplement
        "\u{11db0}", // Unicode 17 astral start supplement
        "a\u{1acf}", // Unicode 17 continuation supplement
    ] {
        for spelling in [name.to_string(), escaped_identifier(name)] {
            for source in [
                format!("let {spelling} = 1; {spelling};"),
                format!("class C {{ #{spelling} = 1; read() {{ return this.#{spelling}; }} }}"),
            ] {
                for options in [ParseOptions::script(), ParseOptions::module()] {
                    parse(&source, options)
                        .unwrap_or_else(|error| panic!("valid identifier {source:?}: {error}"));
                }
            }
        }
    }
}

#[test]
fn alphabetic_and_numeric_approximations_cannot_admit_invalid_identifiers() {
    for name in [
        "\u{0345}a", // Alphabetic combining mark, not ID_Start
        "\u{0301}a",
        "\u{093e}a",
        "\u{203f}a",
        "\u{200c}a",
        "a\u{00b2}", // Numeric, not ID_Continue
        "a\u{1f600}",
    ] {
        for spelling in [name.to_string(), escaped_identifier(name)] {
            for source in [
                format!("let {spelling} = 1;"),
                format!("class C {{ #{spelling} = 1; }}"),
            ] {
                for options in [ParseOptions::script(), ParseOptions::module()] {
                    assert!(parse(&source, options).is_err(), "accepted {source:?}");
                }
            }
        }
    }
}

#[test]
fn character_classification_preserves_reserved_word_and_module_await_errors() {
    for source in ["let default = 1;", r"let \u0064efault = 1;", "let for = 1;"] {
        for options in [ParseOptions::script(), ParseOptions::module()] {
            assert!(parse(source, options).is_err(), "accepted {source:?}");
        }
    }
    assert!(parse("let await = 1;", ParseOptions::script()).is_ok());
    assert!(parse("let await = 1;", ParseOptions::module()).is_err());
    for options in [ParseOptions::script(), ParseOptions::module()] {
        assert!(parse(
            "class C { #default = 1; read() { return this.#default; } }",
            options
        )
        .is_ok());
    }
    assert!(parse(
        "const value = 1; export { value as default, value as 'await' };",
        ParseOptions::module()
    )
    .is_ok());
}
