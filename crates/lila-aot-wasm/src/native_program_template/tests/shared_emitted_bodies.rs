#![cfg(test)]
use super::*;

#[test]
fn thousand_method_program_shares_only_the_native_bodies() {
    let profile = lila_intl::IntlCompilationProfile::Minimal;
    let runtime = crate::runtime_artifact_with_inputs(
        &lila_intl::IntlDataSelection::new(profile.clone()),
        crate::RuntimeArtifactInputs::new(None)
            .with_source_compilation_profile(crate::RuntimeSourceCompilationProfile::NativeHost),
    )
    .unwrap();
    let proof = crate::ValidatedRuntimeArtifact::new(runtime, profile).unwrap();
    let mut source = String::from("({");
    for i in 0..1000 {
        source.push_str(&format!("['foo' + {i}]() {{return 'ok';}}, "));
    }
    source.push_str("['bar']() {return 'all ok'}});");
    let program = lila_ir::lower_runtime_global_script(
        lila_ir::RuntimeGlobalScriptKind::IndirectEval,
        &source.encode_utf16().collect::<Vec<_>>(),
        lila_ir::HostSurfacePolicy::default(),
    )
    .unwrap();
    let raw = crate::emit_runtime_global_script_with_validated_runtime(&program, &proof).unwrap();
    let template = NativeProgramTemplate::for_global_script(&raw).unwrap();
    fn definitions(bytes: &[u8]) -> u32 {
        wasmparser::Parser::new(0)
            .parse_all(bytes)
            .find_map(|payload| match payload.unwrap() {
                wasmparser::Payload::FunctionSection(reader) => Some(reader.count()),
                _ => None,
            })
            .unwrap()
    }
    // The owned payload transform adds one literal helper. The 1000 identical
    // typed method entries become one, retaining all original public entries.
    assert_eq!(
        definitions(raw.bytes()) + 1 - definitions(template.bytes()),
        999
    );
    wasmparser::Validator::new_with_features(wasmparser::WasmFeatures::all())
        .validate_all(template.bytes())
        .unwrap();
}
