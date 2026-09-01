const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const HTML_DDA_SOURCE: &str = include_str!("../src/builtins/host/html_dda.rs");
const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const CLI_SOURCE: &str = include_str!("../../lila-cli/tests/cli/language_errors.rs");
const FIXTURE_SOURCE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_htmldda_host_hook.js");
const GOLDEN_SOURCE: &str = include_str!("emit_golden.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn ordered(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let next = source[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing marker after byte {cursor}: {marker}"));
        cursor += next + marker.len();
    }
}

#[test]
fn host_html_dda_has_one_private_file_owner_and_exact_visibility() {
    assert_eq!(HOST_SOURCE.matches("\nmod html_dda;\n").count(), 1);
    assert!(!HOST_SOURCE.contains("\npub mod html_dda;\n"));
    assert!(!HOST_SOURCE.contains("\npub(crate) mod html_dda;\n"));
    assert!(!HOST_SOURCE.contains("\nmod html_dda {\n"));
    assert!(HTML_DDA_SOURCE.starts_with("use super::*;\n\n"));

    let expected_methods = [
        "pub(crate) fn compile_host_create_html_dda_builtin(",
        "pub(crate) fn compile_host_html_dda_builtin(",
    ];
    assert_eq!(
        HTML_DDA_SOURCE
            .lines()
            .map(str::trim_start)
            .filter(|line| line.starts_with("pub(crate) fn "))
            .collect::<Vec<_>>(),
        expected_methods
    );
    assert_eq!(HTML_DDA_SOURCE.matches(" fn ").count(), 2);
    assert!(!HTML_DDA_SOURCE.contains("pub(super) fn "));
    assert!(!HTML_DDA_SOURCE.contains("\npub fn "));
    for method in expected_methods {
        assert_eq!(HTML_DDA_SOURCE.matches(method).count(), 1);
        assert!(
            !HOST_SOURCE.contains(method),
            "host parent retained `{method}`"
        );
    }
}

#[test]
fn host_html_dda_creation_body_and_dispatch_map_are_closed() {
    let creation = bounded(
        HTML_DDA_SOURCE,
        "pub(crate) fn compile_host_create_html_dda_builtin(",
        "pub(crate) fn compile_host_html_dda_builtin(",
    );
    ordered(
        creation,
        &[
            ".get(&HostBuiltinId::HTMLDDA.function_id())",
            "html_dda_meta.length_name_configurable",
            "self.emit_function_value_payload(&html_dda_meta, function)?;",
            "ValueKind::Function.tag() as i64",
            "LocalSet(self.result_tag_local)",
        ],
    );
    assert_eq!(creation.matches("emit_function_value_payload(").count(), 1);
    assert_eq!(creation.matches("length_name_configurable").count(), 2);

    let callable_body = bounded(
        HTML_DDA_SOURCE,
        "pub(crate) fn compile_host_html_dda_builtin(",
        "\n    }\n}\n",
    );
    ordered(
        callable_body,
        &[
            "Instruction::I64Const(0)",
            "Instruction::LocalSet(self.result_local)",
            "ValueKind::Null.tag() as i64",
            "Instruction::LocalSet(self.result_tag_local)",
        ],
    );
    assert!(!callable_body.contains("reserve_temp_local"));
    assert!(!callable_body.contains("emit_function_value_payload"));

    let creation_dispatch = bounded(
        EMIT_SOURCE,
        "Some(HostBuiltinId::CreateHTMLDDA) => {",
        "Some(HostBuiltinId::HTMLDDA) => {",
    );
    assert_eq!(
        creation_dispatch
            .matches("self.compile_host_create_html_dda_builtin(&mut function)?")
            .count(),
        1
    );
    let callable_dispatch = bounded(
        EMIT_SOURCE,
        "Some(HostBuiltinId::HTMLDDA) => {",
        "Some(HostBuiltinId::ParseInt) => {",
    );
    assert_eq!(
        callable_dispatch
            .matches("self.compile_host_html_dda_builtin(&mut function)?")
            .count(),
        1
    );
    assert_eq!(
        EMIT_SOURCE
            .matches("self.compile_host_create_html_dda_builtin(&mut function)?")
            .count(),
        1
    );
    assert_eq!(
        EMIT_SOURCE
            .matches("self.compile_host_html_dda_builtin(&mut function)?")
            .count(),
        1
    );
}

#[test]
fn host_html_dda_fixture_and_golden_corpus_cover_the_owned_pair() {
    assert_eq!(
        CLI_SOURCE
            .matches("fn run_wasm_backend_succeeds_for_htmldda_host_hook_fixture()")
            .count(),
        1
    );
    assert_eq!(CLI_SOURCE.matches("wasm_htmldda_host_hook.js").count(), 1);
    for witness in [
        "IsHTMLDDA: __lilaCreateHTMLDDA()",
        "typeof $262.IsHTMLDDA !== \"undefined\"",
        "!!$262.IsHTMLDDA !== false",
        "$262.IsHTMLDDA == null",
        "$262.IsHTMLDDA() !== null",
        "new $262.IsHTMLDDA()",
        "Reflect.construct($262.IsHTMLDDA, [])",
        "items[Symbol.iterator] = $262.IsHTMLDDA",
        "class C extends $262.IsHTMLDDA {}",
        "if (prototypeGetterCalled)",
    ] {
        assert!(
            FIXTURE_SOURCE.contains(witness),
            "HTMLDDA fixture must retain `{witness}`"
        );
    }

    assert!(GOLDEN_SOURCE.contains(".join(\"../lila-cli/tests/fixtures\")"));
    assert!(GOLDEN_SOURCE.contains("path.extension().is_some_and(|ext| ext == \"js\")"));
}
