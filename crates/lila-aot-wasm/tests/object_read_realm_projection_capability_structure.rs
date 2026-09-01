use std::fs;
use std::path::Path;

const OBJECTS_SOURCE: &str = include_str!("../src/objects.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/object-read-realm-projection-capability.md");
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
fn object_read_realm_projections_are_exact_non_capability_domains() {
    let declaration_start = OBJECTS_SOURCE
        .find("enum OutlinedObjectReadRealmArgument {")
        .expect("missing outlined object-read Realm declaration");
    assert_eq!(
        OBJECTS_SOURCE[..declaration_start]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .map(str::trim),
        Some("};")
    );
    let declarations = bounded(
        OBJECTS_SOURCE,
        "enum OutlinedObjectReadRealmArgument {",
        "fn outlined_object_read_realm_argument(",
    );
    assert!(!declarations.contains("#["));
    assert_eq!(
        normalized(declarations),
        concat!(
            "TrustedCurrentEnvironment,MainRealmFallback,}",
            "enumObjectReadRevocationErrorRealm{",
            "TrustedCurrentEnvironment,MainRealmFallback,}"
        )
    );

    for domain in [
        "OutlinedObjectReadRealmArgument",
        "ObjectReadRevocationErrorRealm",
    ] {
        for capability in ["Clone", "Copy", "Debug", "Default", "PartialEq", "Eq"] {
            assert!(!OBJECTS_SOURCE.contains(&format!("{capability} for {domain}")));
        }
    }

    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&src, "OutlinedObjectReadRealmArgument"),
        10
    );
    assert_eq!(
        count_in_rust_sources(&src, "ObjectReadRevocationErrorRealm"),
        10
    );
    for row in ["TrustedCurrentEnvironment", "MainRealmFallback"] {
        assert_eq!(
            count_in_rust_sources(&src, &format!("OutlinedObjectReadRealmArgument::{row}")),
            4
        );
        assert_eq!(
            count_in_rust_sources(&src, &format!("ObjectReadRevocationErrorRealm::{row}")),
            4
        );
    }
}

#[test]
fn all_four_source_rows_project_exhaustively_into_both_domains() {
    let projections = normalized(bounded(
        OBJECTS_SOURCE,
        "fn outlined_object_read_realm_argument(",
        "#[cfg(test)]",
    ));
    assert_eq!(
        projections,
        concat!(
            "source:ObjectReadErrorRealmSource,)->OutlinedObjectReadRealmArgument{",
            "matchsource{ObjectReadErrorRealmSource::GlobalFallback=>{",
            "OutlinedObjectReadRealmArgument::MainRealmFallback}",
            "ObjectReadErrorRealmSource::StandardBuiltinEnvironment|",
            "ObjectReadErrorRealmSource::ObjectReadHelperArgument|",
            "ObjectReadErrorRealmSource::ProxyDispatchHelperArgument=>{",
            "OutlinedObjectReadRealmArgument::TrustedCurrentEnvironment}}}",
            "fnobject_read_revocation_error_realm(",
            "source:ObjectReadErrorRealmSource,)->ObjectReadRevocationErrorRealm{",
            "matchsource{ObjectReadErrorRealmSource::GlobalFallback=>{",
            "ObjectReadRevocationErrorRealm::MainRealmFallback}",
            "ObjectReadErrorRealmSource::StandardBuiltinEnvironment|",
            "ObjectReadErrorRealmSource::ObjectReadHelperArgument|",
            "ObjectReadErrorRealmSource::ProxyDispatchHelperArgument=>{",
            "ObjectReadRevocationErrorRealm::TrustedCurrentEnvironment}}}"
        )
    );

    let unit = normalized(bounded(
        OBJECTS_SOURCE,
        "fn object_read_realm_projection_excludes_ordinary_lexical_environments() {",
        "\n    }\n}",
    ));
    for runtime_body_census in [
        concat!(
            "assert_eq!(object_read_helpers,vec![",
            "RuntimeHelperId::ObjectRead,RuntimeHelperId::ObjectReadProxy]);"
        ),
        concat!(
            "assert_eq!(proxy_dispatch_helpers,vec![",
            "RuntimeHelperId::ProxyCall,RuntimeHelperId::ProxyConstruct]);"
        ),
    ] {
        assert_eq!(
            unit.matches(runtime_body_census).count(),
            1,
            "{runtime_body_census}"
        );
    }
    for expected in [
        concat!(
            "matchoutlined_object_read_realm_argument(source){",
            "OutlinedObjectReadRealmArgument::TrustedCurrentEnvironment=>{}",
            "OutlinedObjectReadRealmArgument::MainRealmFallback=>{",
            "panic!(\"trustedobject-readsourcelostitsoutlinedRealmargument\")}}"
        ),
        concat!(
            "matchobject_read_revocation_error_realm(source){",
            "ObjectReadRevocationErrorRealm::TrustedCurrentEnvironment=>{}",
            "ObjectReadRevocationErrorRealm::MainRealmFallback=>{",
            "panic!(\"trustedobject-readsourcelostitsrevocation-errorRealm\")}}"
        ),
        concat!(
            "matchoutlined_object_read_realm_argument(",
            "ObjectReadErrorRealmSource::GlobalFallback){",
            "OutlinedObjectReadRealmArgument::MainRealmFallback=>{}",
            "OutlinedObjectReadRealmArgument::TrustedCurrentEnvironment=>{",
            "panic!(\"globalfallbackexposedanoutlinedlexicalenvironmentasRealmstate\")}}"
        ),
        concat!(
            "matchobject_read_revocation_error_realm(",
            "ObjectReadErrorRealmSource::GlobalFallback){",
            "ObjectReadRevocationErrorRealm::MainRealmFallback=>{}",
            "ObjectReadRevocationErrorRealm::TrustedCurrentEnvironment=>{",
            "panic!(\"globalfallbackexposedalexicalenvironmentasrevocationRealmstate\")}}"
        ),
    ] {
        assert_eq!(unit.matches(expected).count(), 1, "unit match `{expected}`");
    }
    for forbidden in [
        "assert_eq!(outlined_object_read_realm_argument",
        "assert_eq!(object_read_revocation_error_realm",
    ] {
        assert!(!unit.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn each_projection_consumer_keeps_its_exact_emission_policy() {
    let outlined_consumer = normalized(bounded(
        OBJECTS_SOURCE,
        "fn emit_outlined_object_read_realm_argument(&self, function: &mut Function) {",
        "\n    }\n\n    pub(crate) fn emit_object_read(",
    ));
    assert_eq!(
        outlined_consumer,
        concat!(
            "matchoutlined_object_read_realm_argument(",
            "self.object_read_error_realm_source()){",
            "OutlinedObjectReadRealmArgument::TrustedCurrentEnvironment=>{",
            "function.instruction(&Instruction::LocalGet(self.current_env_local));}",
            "OutlinedObjectReadRealmArgument::MainRealmFallback=>{",
            "function.instruction(&Instruction::I64Const(0));}}"
        )
    );

    let revocation_consumer = normalized(bounded(
        OBJECTS_SOURCE,
        "match object_read_revocation_error_realm(self.object_read_error_realm_source()) {",
        "        self.emit_return_current_completion(function);",
    ));
    assert_eq!(
        revocation_consumer,
        concat!(
            "ObjectReadRevocationErrorRealm::TrustedCurrentEnvironment=>self.",
            "emit_throw_current_function_realm_type_error(",
            "\"Proxyhandlerisnull\",self.result_local,self.result_tag_local,function,)?,",
            "ObjectReadRevocationErrorRealm::MainRealmFallback=>self.emit_throw_runtime_error(",
            "TYPE_ERROR_NAME,\"Proxyhandlerisnull\",self.result_local,",
            "self.result_tag_local,function,)?,}"
        )
    );

    let consumers = format!("{outlined_consumer}{revocation_consumer}");
    for forbidden in ["_=>", "==", "!=", "matches!(", "unreachable!"] {
        assert!(!consumers.contains(forbidden), "found `{forbidden}`");
    }
}

#[test]
fn contract_and_t10_record_the_source_equivalent_capability_closure() {
    for marker in [
        "two distinct private, non-derived",
        "ABI argument receives",
        "does not claim an object-model redesign",
    ] {
        assert!(
            CONTRACT.contains(marker),
            "missing contract marker `{marker}`"
        );
    }
    assert!(TASK.contains("object-read-realm-projection-capability.md"));
}
