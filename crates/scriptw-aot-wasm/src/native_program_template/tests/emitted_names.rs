#![cfg(test)]
use super::*;

#[test]
fn emitted_function_names_share_native_code_without_changing_public_symbols_or_payloads() {
    let profile = scriptw_intl::IntlCompilationProfile::Minimal;
    let runtime = crate::runtime_artifact_with_inputs(
        &scriptw_intl::IntlDataSelection::new(profile.clone()),
        crate::RuntimeArtifactInputs::new(None)
            .with_source_compilation_profile(crate::RuntimeSourceCompilationProfile::NativeHost),
    )
    .unwrap();
    let proof = crate::ValidatedRuntimeArtifact::new(runtime, profile).unwrap();
    let mut templates = Vec::new();
    for name in ["alpha", "longer_source_name"] {
        let source = format!("return function {name}(){{ return 17; }};");
        let program = scriptw_ir::lower_runtime_function(
            scriptw_ir::DynamicFunctionKind::Ordinary,
            &[source.encode_utf16().collect()],
            scriptw_ir::HostSurfacePolicy::default(),
        )
        .unwrap();
        let raw = crate::emit_runtime_function_with_validated_runtime(&program, &proof).unwrap();
        let mut found_source_name = false;
        for payload in wasmparser::Parser::new(0).parse_all(raw.bytes()) {
            if let wasmparser::Payload::CustomSection(section) = payload.unwrap() {
                if let wasmparser::KnownCustom::Name(reader) = section.as_known() {
                    for subsection in reader {
                        if let wasmparser::Name::Function(functions) = subsection.unwrap() {
                            for function in functions {
                                found_source_name |=
                                    function.unwrap().name.starts_with(&format!("js::{name}#"));
                            }
                        }
                    }
                }
            }
        }
        assert!(found_source_name, "public artifact retains {name}");
        let template = NativeProgramTemplate::for_function(&raw).unwrap();
        assert!(template.program_payload().unwrap().permits_instance_reuse());
        wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
            .validate_all(template.bytes())
            .unwrap();
        templates.push(template);
    }
    assert_eq!(templates[0].bytes(), templates[1].bytes());
    assert_ne!(
        templates[0].program_payload().unwrap().bytes(),
        templates[1].program_payload().unwrap().bytes()
    );
}
