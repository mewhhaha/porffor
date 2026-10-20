use super::*;

pub(super) fn proven_primitive_eval_kernel_shares_code_and_keeps_literal_operands_per_call() {
    let profile = IntlCompilationProfile::Minimal;
    let runtime = crate::runtime_artifact_with_inputs(
        &IntlDataSelection::new(profile.clone()),
        crate::RuntimeArtifactInputs::new(None)
            .with_source_compilation_profile(crate::RuntimeSourceCompilationProfile::NativeHost),
    )
    .unwrap();
    let validated = ValidatedRuntimeArtifact::new(Arc::clone(&runtime), profile).unwrap();
    let mut previous = None;
    for source in [
        "1",
        "987.25",
        "false",
        "true",
        "null",
        "/* new owner */ 1e309",
    ] {
        let program = request(&caller([]), &source.encode_utf16().collect::<Vec<_>>());
        let proof = program.primitive_literal().unwrap();
        let template = crate::NativeProgramTemplate::for_primitive_direct_eval(&proof, &validated);
        assert!(Arc::ptr_eq(template.runtime(), &runtime));
        assert!(template.permits_instance_reuse());
        assert!(template.program_payload().is_none());
        assert!(template.data_images().is_empty());
        assert!(template.array_factories().is_empty());
        assert!(template.literal_operands().is_empty());
        assert!(template.empty_direct_eval_export().is_none());
        assert!(template.primitive_direct_eval_export().is_some());
        assert!(template.bytes().len() < 200);
        Validator::new_with_features(WasmFeatures::all())
            .validate_all(template.bytes())
            .unwrap();
        for payload in Parser::new(0).parse_all(template.bytes()) {
            match payload.unwrap() {
                Payload::TypeSection(types) => {
                    let types = types
                        .into_iter_err_on_gc_types()
                        .collect::<Result<Vec<_>, _>>()
                        .unwrap();
                    assert_eq!(types.len(), 1);
                    assert_eq!(
                        types[0].params(),
                        &[wasmparser::ValType::I32, wasmparser::ValType::I64]
                    );
                    assert_eq!(types[0].results().len(), 6);
                }
                Payload::Version { .. }
                | Payload::FunctionSection(_)
                | Payload::ExportSection(_)
                | Payload::CodeSectionStart { .. }
                | Payload::CodeSectionEntry(_)
                | Payload::End(_) => {}
                _ => panic!("scalar kernel cannot import, allocate or retain source state"),
            }
        }
        if let Some(previous) = &previous {
            assert_eq!(template.bytes(), previous);
        }
        previous = Some(template.bytes().to_vec());
    }
}
