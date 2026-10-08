//! Developer robustness observation of the original uncached product pipeline.
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilerInspectionStage {
    IrInput,
    Preparation,
    Lowering,
    IrAdmission,
    Emission,
    RuntimeSetup,
    Validation,
}

#[derive(Debug)]
pub struct CompilerInspectionFailure {
    stage: CompilerInspectionStage,
    error: EngineError,
}
impl CompilerInspectionFailure {
    pub fn stage(&self) -> CompilerInspectionStage {
        self.stage
    }
    pub fn error(&self) -> &EngineError {
        &self.error
    }
}

impl Engine {
    /// No source execution, program cache, alternate lowerer or validator.
    /// The callback runs before each actual stage, so a killed worker retains
    /// the last entered stage rather than guessing one from its exit status.
    pub fn inspect_compilation(
        &self,
        source: &str,
        goal: ParseGoal,
        options: CompileOptions,
        mut entered: impl FnMut(CompilerInspectionStage) + Send,
    ) -> Result<Artifact, CompilerInspectionFailure> {
        run_on_sized_stack(|| {
            entered(CompilerInspectionStage::Preparation);
            let prepared = self
                .prepare_compilation(source, goal, &options)
                .map_err(|error| CompilerInspectionFailure {
                    stage: CompilerInspectionStage::Preparation,
                    error,
                })?;
            entered(CompilerInspectionStage::Lowering);
            let unit = self
                .compile_prepared_on_current_thread(prepared)
                .map_err(|error| CompilerInspectionFailure {
                    stage: CompilerInspectionStage::Lowering,
                    error,
                })?;
            self.inspect_emission(&unit, &mut entered)
        })
    }

    /// Developer-only native admission probe. No supplied source/function ids,
    /// environments or continuation certificates can enter this input domain.
    /// An actual empty Script owns the unchanged product compilation context;
    /// admitted self-contained statements use the same emitter and validator.
    /// This does not execute source or replace the product source compiler.
    pub fn inspect_ir_compilation(
        &self,
        bytes: &[u8],
        options: CompileOptions,
        mut entered: impl FnMut(CompilerInspectionStage) + Send,
    ) -> Result<Artifact, CompilerInspectionFailure> {
        run_on_sized_stack(|| {
            entered(CompilerInspectionStage::IrInput);
            let input = lila_ir::IrRobustnessInput::from_bytes(bytes).map_err(|error| {
                CompilerInspectionFailure {
                    stage: CompilerInspectionStage::IrInput,
                    error: EngineError::new(error.to_string()),
                }
            })?;
            entered(CompilerInspectionStage::Preparation);
            let prepared = self
                .prepare_compilation("", ParseGoal::Script, &options)
                .map_err(|error| CompilerInspectionFailure {
                    stage: CompilerInspectionStage::Preparation,
                    error,
                })?;
            entered(CompilerInspectionStage::Lowering);
            let mut unit = self
                .compile_prepared_on_current_thread(prepared)
                .map_err(|error| CompilerInspectionFailure {
                    stage: CompilerInspectionStage::Lowering,
                    error,
                })?;
            entered(CompilerInspectionStage::IrAdmission);
            input
                .admit_into_empty_script(&mut unit.ir)
                .map_err(|error| CompilerInspectionFailure {
                    stage: CompilerInspectionStage::IrAdmission,
                    error: EngineError::new(error.to_string()),
                })?;
            self.inspect_emission(&unit, &mut entered)
        })
    }

    fn inspect_emission(
        &self,
        unit: &CompilationUnit,
        entered: &mut impl FnMut(CompilerInspectionStage),
    ) -> Result<Artifact, CompilerInspectionFailure> {
        entered(CompilerInspectionStage::Emission);
        let artifact =
            self.emit_wasm_on_current_thread(unit)
                .map_err(|error| CompilerInspectionFailure {
                    stage: CompilerInspectionStage::Emission,
                    error,
                })?;
        entered(CompilerInspectionStage::RuntimeSetup);
        let runtime = shared_wasm_engine().map_err(|error| CompilerInspectionFailure {
            stage: CompilerInspectionStage::RuntimeSetup,
            error,
        })?;
        entered(CompilerInspectionStage::Validation);
        WasmtimeModule::validate(&runtime, &artifact.bytes).map_err(|error| {
            CompilerInspectionFailure {
                stage: CompilerInspectionStage::Validation,
                error: EngineError::new(format!("emitted Wasm validation failed: {error:#}")),
            }
        })?;
        Ok(artifact)
    }
}
