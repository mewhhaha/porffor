const CLI_SOURCE: &str = include_str!("../../../lila-cli/tests/cli/intl.rs");
const FIXTURE: &str =
    include_str!("../../../lila-cli/tests/fixtures/wasm_intl_canonical_locale_tag_roles.js");
const CONTRACT: &str = include_str!(
    "../../../../docs/rust-rewrite/contracts/intl-canonical-locale-tag-invocation-authority.md"
);
const TASK: &str = include_str!("../../../../tasks/23-intl402.md");

#[test]
fn contract_task_and_public_fixture_own_the_authority() {
    for marker in [
        "canonical locale tag invocation authority",
        "transpose tag, language, script, region, base-name, and validity roles",
        "intl_canonical_locale_tag_invocation_structure",
    ] {
        assert!(CONTRACT.contains(marker), "contract marker `{marker}`");
        assert!(TASK.contains(marker), "task marker `{marker}`");
    }
    assert_eq!(
        CLI_SOURCE
            .matches("run_wasm_intl_canonical_locale_tag_roles_fixture_succeeds")
            .count(),
        1
    );
    assert_eq!(
        CLI_SOURCE
            .matches("wasm_intl_canonical_locale_tag_roles.js")
            .count(),
        1
    );
    for witness in [
        "locale.language !== \"en\"",
        "locale.script !== \"Latn\"",
        "locale.region !== \"US\"",
        "locale.baseName !== \"en-Latn-US\"",
        "resolved.locale !== \"en-US-u-ca-iso8601\"",
    ] {
        assert!(FIXTURE.contains(witness), "fixture witness `{witness}`");
    }
}
