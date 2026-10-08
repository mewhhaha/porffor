//! Product pipeline evidence, without a second parser/lowerer/emitter.
use lila_engine::{
    CompileOptions, CompilerInspectionStage, Engine, ModuleLoadingPolicy, RealmBuilder,
};
use lila_front::{ParseDiagnosticPhase, ParseGoal};

#[test]
fn inspected_artifact_is_the_original_product_artifact_and_rejections_stop_before_emission() {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let source = "function twice(value) { return value + value; } twice(7);";
    let mut stages = Vec::new();
    let options = CompileOptions {
        module_loading_policy: ModuleLoadingPolicy::RejectAll,
        ..CompileOptions::default()
    };
    let artifact = engine
        .inspect_compilation(source, ParseGoal::Script, options.clone(), |stage| {
            stages.push(stage)
        })
        .unwrap();
    let original = engine
        .emit_wasm(&engine.compile_script(source, options.clone()).unwrap())
        .unwrap();
    assert_eq!(artifact.bytes, original.bytes);
    assert_eq!(
        stages,
        [
            CompilerInspectionStage::Preparation,
            CompilerInspectionStage::Lowering,
            CompilerInspectionStage::Emission,
            CompilerInspectionStage::RuntimeSetup,
            CompilerInspectionStage::Validation
        ]
    );
    stages.clear();
    let error = engine
        .inspect_compilation("let value = ;", ParseGoal::Script, options, |stage| {
            stages.push(stage)
        })
        .unwrap_err();
    assert_eq!(error.stage(), CompilerInspectionStage::Preparation);
    assert_eq!(stages, [CompilerInspectionStage::Preparation]);
    assert_eq!(
        error.error().parse_diagnostic().unwrap().phase(),
        ParseDiagnosticPhase::Parse
    );
}

#[test]
fn native_ir_rejections_retain_input_and_actual_admission_stage_without_emission() {
    let engine = Engine::new(RealmBuilder::new().build());
    let options = CompileOptions {
        module_loading_policy: ModuleLoadingPolicy::RejectAll,
        ..CompileOptions::default()
    };
    for (bytes, expected, reason) in [
        (br#"{"schema_version":2,"body":[]}"#.as_slice(), CompilerInspectionStage::IrInput, "schema"),
        (br#"{"schema_version":1,"body":[{"op":"define_property","target":{"kind":"null"},"key":"x","value":{"kind":"undefined"}}]}"#.as_slice(),
            CompilerInspectionStage::IrAdmission, "NonObjectTarget"),
        (br#"{"schema_version":1,"body":[{"op":"if","condition":{"kind":"boolean","value":false},"then":[{"op":"await","value":{"kind":"null"}}],"else":[]}]}"#.as_slice(),
            CompilerInspectionStage::IrAdmission, "Suspension"),
    ] {
        let mut stages = Vec::new();
        let failure = engine.inspect_ir_compilation(bytes, options.clone(), |stage| stages.push(stage)).unwrap_err();
        assert_eq!(failure.stage(), expected);
        assert!(failure.error().to_string().contains(reason), "{failure:?}");
        if expected == CompilerInspectionStage::IrInput {
            assert_eq!(stages, [CompilerInspectionStage::IrInput]);
        } else {
            assert_eq!(stages, [CompilerInspectionStage::IrInput, CompilerInspectionStage::Preparation,
                CompilerInspectionStage::Lowering, CompilerInspectionStage::IrAdmission]);
        }
    }
}
