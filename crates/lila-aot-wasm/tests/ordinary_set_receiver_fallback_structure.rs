use std::fs;
use std::path::Path;

const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/object.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_ordinary_set_outlined_receiver.js");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/ordinary-set-receiver-fallback.md");
const TASK: &str = include_str!("../../../tasks/10-object-model-descriptors-exotics.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source
        .find(earlier)
        .unwrap_or_else(|| panic!("missing earlier operation `{earlier}`"));
    let later_offset = source
        .find(later)
        .unwrap_or_else(|| panic!("missing later operation `{later}`"));
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn count_in_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
                .matches(needle)
                .count()
        })
        .sum()
}

#[test]
fn ordinary_set_receiver_fallback_is_an_exact_private_non_derived_domain() {
    let declaration_region = bounded(
        EMIT_SOURCE,
        concat!(
            "pub(crate) enum OrdinarySetDataOnReceiverEmission {\n",
            "    Inline,\n",
            "    Outlined,\n",
            "}\n\n"
        ),
        "\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub(crate) enum CompletionKind",
    );
    let declaration = bounded(
        declaration_region,
        "enum OrdinarySetReceiverFallback {",
        "\n}",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(variants, ["Allowed,", "Denied,"]);
    assert_eq!(
        declaration_region
            .lines()
            .filter(|line| line.contains("enum OrdinarySetReceiverFallback"))
            .collect::<Vec<_>>(),
        ["enum OrdinarySetReceiverFallback {"]
    );
    assert!(!declaration_region.contains("#["));
    assert!(!declaration_region.contains("pub enum OrdinarySetReceiverFallback"));
    assert!(!declaration_region.contains("pub(crate) enum OrdinarySetReceiverFallback"));
    assert!(!declaration_region.contains("impl OrdinarySetReceiverFallback"));

    for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
        assert!(!EMIT_SOURCE.contains(&format!("{capability} for OrdinarySetReceiverFallback")));
    }

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "OrdinarySetReceiverFallback"),
        6,
        "the declaration, two producers, typed consumer and two exhaustive arms own every mention"
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "OrdinarySetReceiverFallback::Allowed"),
        2
    );
    assert_eq!(
        count_in_rust_sources(&source_root, "OrdinarySetReceiverFallback::Denied"),
        2
    );
}

#[test]
fn helper_identity_and_receiver_fallback_are_one_exhaustive_projection() {
    let consumer = bounded(
        EMIT_SOURCE,
        "    fn compile_ordinary_set_helper(",
        "\n    /// Compiles the shared object-define-data runtime helper.",
    );
    let projection = normalized(bounded(
        consumer,
        "let (runtime_helper, receiver_generic_write_allowed) = match receiver_fallback {",
        "let mut function = self.begin_helper_body(runtime_helper);",
    ));
    assert_eq!(
        projection,
        concat!(
            "OrdinarySetReceiverFallback::Allowed=>(RuntimeHelperId::OrdinarySet,true),",
            "OrdinarySetReceiverFallback::Denied=>{",
            "(RuntimeHelperId::OrdinarySetWithoutReceiverFallback,false)}};"
        )
    );
    assert_eq!(consumer.matches("match receiver_fallback {").count(), 1);
    assert_eq!(
        consumer
            .matches("OrdinarySetReceiverFallback::Allowed =>")
            .count(),
        1
    );
    assert_eq!(
        consumer
            .matches("OrdinarySetReceiverFallback::Denied =>")
            .count(),
        1
    );
    for forbidden in [
        "_ =>",
        "matches!",
        "receiver_fallback ==",
        "receiver_fallback !=",
        "unreachable!",
        ".helper()",
        ".allows_receiver_generic_write()",
    ] {
        assert!(!consumer.contains(forbidden), "found `{forbidden}`");
    }

    let normalized_consumer = normalized(consumer);
    let set_call = concat!(
        "self.emit_ordinary_set_result_with_receiver_fallback(",
        "target_payload_local,target_tag_local,receiver_payload_local,receiver_tag_local,",
        "key_payload_local,key_tag_local,value_payload_local,value_tag_local,",
        "self.result_local,receiver_generic_write_allowed,&mutfunction,)?;"
    );
    assert_eq!(normalized_consumer.matches(set_call).count(), 1);
    assert_before(
        consumer,
        "match receiver_fallback {",
        "self.begin_helper_body(runtime_helper)",
    );
    assert_before(
        consumer,
        "self.begin_helper_body(runtime_helper)",
        "self.emit_builtin_arg_to_locals(0,",
    );
    assert_before(
        &normalized_consumer,
        concat!(
            "self.emit_builtin_arg_to_locals(4,realm_environment_local,",
            "realm_environment_tag_local,&mutfunction,)"
        ),
        "self.emit_ordinary_set_result_with_receiver_fallback(",
    );
}

#[test]
fn exactly_two_helper_producers_and_filings_choose_their_fallback_policy() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "compile_ordinary_set_helper("),
        3,
        "the definition and two runtime-helper producers are the complete call census"
    );

    let producers = bounded(
        EMIT_SOURCE,
        "    let ordinary_set_helper_function = uses_heap",
        "    let decimal_to_binary64_helper_function = uses_heap",
    );
    assert_eq!(producers.matches("compile_ordinary_set_helper(").count(), 2);
    assert_eq!(
        producers
            .matches("compile_ordinary_set_helper(OrdinarySetReceiverFallback::Allowed)")
            .count(),
        1
    );
    assert_eq!(
        producers
            .matches("compile_ordinary_set_helper(OrdinarySetReceiverFallback::Denied)")
            .count(),
        1
    );
    assert_before(
        producers,
        "OrdinarySetReceiverFallback::Allowed",
        "OrdinarySetReceiverFallback::Denied",
    );

    let helper_filings = normalized(bounded(
        EMIT_SOURCE,
        concat!(
            ".expect(\"array-write helper must exist when heap is enabled\"),\n",
            "        );"
        ),
        "        helper_bodies.insert(\n            RuntimeHelperId::DecimalToBinary64,",
    ));
    assert_eq!(
        helper_filings,
        concat!(
            "helper_bodies.insert(RuntimeHelperId::OrdinarySet,",
            "ordinary_set_helper_function.expect(\"ordinary-sethelpermustexistwhenheapisenabled\"),);",
            "helper_bodies.insert(RuntimeHelperId::OrdinarySetWithoutReceiverFallback,",
            "ordinary_set_without_receiver_fallback_helper_function.expect(",
            "\"ordinary-setno-fallbackhelpermustexistwhenheapisenabled\"),);"
        )
    );
}

#[test]
fn contract_and_existing_product_witness_pin_the_closed_seam() {
    assert!(CONTRACT.contains("OrdinarySetReceiverFallback"));
    assert!(CONTRACT
        .contains("cargo test -p lila-aot-wasm --test ordinary_set_receiver_fallback_structure"));
    assert!(TASK.contains("OrdinarySetReceiverFallback"));
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_preserves_outlined_ordinary_set_receiver_semantics()"));
    assert!(CLI_TESTS.contains("wasm_ordinary_set_outlined_receiver.js"));
    for marker in [
        "throw \"inherited setter receiver\"",
        "Reflect.set({}, symbol, 8, receiver)",
        "throw \"mapped arguments receiver write\"",
        "otherGlobal.Reflect.set([], \"length\", -1)",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker `{marker}`"
        );
    }
}
