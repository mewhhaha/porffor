use lila_aot_wasm::emit;
use lila_front::{parse, ParseOptions};
use lila_ir::{lower_with_host_surface_policy, HostSurfacePolicy};
use wasmparser::{Parser, Payload, Validator, WasmFeatures};

fn intl_import_count(source: &'static str, host_surface_policy: HostSurfacePolicy) -> usize {
    std::thread::Builder::new()
        .name("intl-host-imports".to_string())
        .stack_size(64 * 1024 * 1024)
        .spawn(move || {
            let parsed = parse(source, ParseOptions::script()).expect("fixture should parse");
            let program = lower_with_host_surface_policy(&parsed, host_surface_policy);
            let artifact = emit(&program).expect("standalone provider caller must emit");
            let mut features = WasmFeatures::default();
            for feature in [
                WasmFeatures::THREADS,
                WasmFeatures::MULTI_MEMORY,
                WasmFeatures::REFERENCE_TYPES,
                WasmFeatures::FUNCTION_REFERENCES,
                WasmFeatures::GC,
                WasmFeatures::EXCEPTIONS,
                WasmFeatures::TAIL_CALL,
            ] {
                features.set(feature, true);
            }
            Validator::new_with_features(features)
                .validate_all(&artifact.bytes)
                .expect("optional Intl import must preserve function and call indices");
            let mut count = 0;
            for payload in Parser::new(0).parse_all(&artifact.bytes) {
                if let Payload::ImportSection(section) = payload.expect("section must decode") {
                    for import in section.into_imports() {
                        let import = import.expect("import must decode");
                        count +=
                            usize::from(import.module == "lila_host" && import.name == "intl_call");
                    }
                }
            }
            count
        })
        .expect("compiler worker should spawn")
        .join()
        .expect("compiler worker must not panic")
}

#[test]
fn locale_construction_declares_its_own_provider_import() {
    assert_eq!(
        intl_import_count("new Intl.Locale('iw');", HostSurfacePolicy::default()),
        1
    );
}

#[test]
fn locale_options_and_optional_imports_preserve_wasm_indices() {
    assert_eq!(
        intl_import_count(
            "var locale = new Intl.Locale('en', {calendar:'islamicc',numeric:true}); locale.calendar; Date.now(); Math.random();",
            HostSurfacePolicy::default(),
        ),
        1
    );
}

#[test]
fn created_realm_locale_dependencies_declare_the_provider_import() {
    assert_eq!(
        intl_import_count(
            "var foreign = __lilaCreateRealm().global; new foreign.Intl.Locale('iw').language;",
            HostSurfacePolicy::Test262,
        ),
        1
    );
}

#[test]
fn a_program_without_provider_callers_omits_the_intl_import() {
    assert_eq!(intl_import_count("1 + 1;", HostSurfacePolicy::default()), 0);
}

#[test]
fn likely_subtag_methods_declare_the_provider_and_preserve_wasm_indices() {
    assert_eq!(
        intl_import_count(
            "var locale = new Intl.Locale('en'); locale.maximize().minimize(); Date.now(); Math.random();",
            HostSurfacePolicy::default(),
        ),
        1,
    );
}

#[test]
fn detached_likely_subtag_methods_retain_provider_dependencies() {
    assert_eq!(
        intl_import_count(
            "var foreign = __lilaCreateRealm().global; var method = foreign.Intl.Locale.prototype.maximize; method.call(new Intl.Locale('en'));",
            HostSurfacePolicy::Test262,
        ),
        1,
    );
}

#[test]
fn number_format_and_both_primitive_locale_consumers_root_the_provider() {
    for source in [
        "new Intl.NumberFormat('en-US').format(123);",
        "(123).toLocaleString('en-US', {minimumFractionDigits:2});",
        "(123n).toLocaleString('en-US', {minimumFractionDigits:2});",
        "var foreign=__lilaCreateRealm().global; foreign.Intl.NumberFormat.prototype.formatRange.call(new Intl.NumberFormat('en-US'),1,2);",
    ] {
        assert_eq!(intl_import_count(source, HostSurfacePolicy::Test262), 1);
    }
}
