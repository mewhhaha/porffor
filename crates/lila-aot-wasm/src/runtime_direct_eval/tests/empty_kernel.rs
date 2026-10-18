use super::*;

pub(super) fn proven_empty_eval_kernel_has_only_source_independent_completion_code() {
    let profile = IntlCompilationProfile::Minimal;
    let runtime = crate::runtime_artifact_with_inputs(
        &IntlDataSelection::new(profile.clone()),
        crate::RuntimeArtifactInputs::new(None)
            .with_source_compilation_profile(crate::RuntimeSourceCompilationProfile::NativeHost),
    )
    .unwrap();
    let validated = ValidatedRuntimeArtifact::new(Arc::clone(&runtime), profile).unwrap();
    let mut previous = None;
    for source in ["/* first */", "// second", ""] {
        let program = request(&caller([]), &source.encode_utf16().collect::<Vec<_>>());
        let proof = program.empty_completion().unwrap();
        let kernel = crate::NativeProgramTemplate::for_empty_direct_eval(&proof, &validated);
        assert!(Arc::ptr_eq(kernel.runtime(), &runtime));
        assert!(kernel.permits_instance_reuse());
        assert!(kernel.program_payload().is_none());
        assert!(kernel.data_images().is_empty());
        assert!(kernel.array_factories().is_empty());
        assert!(kernel.literal_operands().is_empty());
        assert!(kernel.bytes().len() < 200);
        Validator::new_with_features(WasmFeatures::all())
            .validate_all(kernel.bytes())
            .unwrap();
        for item in Parser::new(0).parse_all(kernel.bytes()) {
            assert!(matches!(
                item.unwrap(),
                Payload::Version { .. }
                    | Payload::TypeSection(_)
                    | Payload::FunctionSection(_)
                    | Payload::ExportSection(_)
                    | Payload::CodeSectionStart { .. }
                    | Payload::CodeSectionEntry(_)
                    | Payload::End(_)
            ));
        }
        if let Some(previous) = &previous {
            assert_eq!(kernel.bytes(), previous);
        }
        previous = Some(kernel.bytes().to_vec());
    }
}
