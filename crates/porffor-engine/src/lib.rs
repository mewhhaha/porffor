use porffor_front::{parse, ParseGoal, ParseOptions, SourceUnit};
use porffor_ir::{lower, ProgramIr};

pub use porffor_runtime::{HostHooks, NullHostHooks, Realm, RealmBuilder};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactKind {
    Wasm,
    C,
    Native,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ExecutionBackend {
    #[default]
    SpecExec,
    WasmAot,
}

impl ExecutionBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            ExecutionBackend::SpecExec => "spec-exec",
            ExecutionBackend::WasmAot => "wasm-aot",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub kind: ArtifactKind,
    pub bytes: Vec<u8>,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompileOptions {
    pub filename: Option<String>,
    pub optimize: bool,
    pub target_triple: Option<String>,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            filename: None,
            optimize: true,
            target_triple: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunOptions {
    pub backend: ExecutionBackend,
    pub argv: Vec<String>,
    pub module_root: Option<String>,
    pub test_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationUnit {
    pub source: SourceUnit,
    pub ir: ProgramIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutcome {
    pub backend_used: ExecutionBackend,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InspectionReport {
    pub goal: ParseGoal,
    pub source_len: usize,
    pub stages: Vec<&'static str>,
    pub invariants: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineError {
    message: String,
}

impl EngineError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl core::fmt::Display for EngineError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for EngineError {}

pub struct Engine {
    realm: Realm,
}

impl Engine {
    pub fn new(realm: Realm) -> Self {
        Self { realm }
    }

    pub fn shell_name(&self) -> &str {
        &self.realm.shell_name
    }

    pub fn compile_script(
        &self,
        source: &str,
        options: CompileOptions,
    ) -> Result<CompilationUnit, EngineError> {
        self.compile(source, ParseGoal::Script, options)
    }

    pub fn compile_module(
        &self,
        source: &str,
        options: CompileOptions,
    ) -> Result<CompilationUnit, EngineError> {
        self.compile(source, ParseGoal::Module, options)
    }

    pub fn run_script(
        &self,
        source: &str,
        options: CompileOptions,
        run: RunOptions,
    ) -> Result<RunOutcome, EngineError> {
        if run.backend == ExecutionBackend::SpecExec {
            return self.run_with_spec_exec(
                source,
                options.filename.as_deref(),
                ParseGoal::Script,
                run,
            );
        }
        let unit = self.compile_script(source, options)?;
        self.run_compiled_unit(&unit, source, run)
    }

    pub fn run_module(
        &self,
        source: &str,
        options: CompileOptions,
        run: RunOptions,
    ) -> Result<RunOutcome, EngineError> {
        if run.backend == ExecutionBackend::SpecExec {
            return self.run_with_spec_exec(
                source,
                options.filename.as_deref(),
                ParseGoal::Module,
                run,
            );
        }
        let unit = self.compile_module(source, options)?;
        self.run_compiled_unit(&unit, source, run)
    }

    pub fn emit_wasm(&self, unit: &CompilationUnit) -> Result<Artifact, EngineError> {
        match porffor_aot_wasm::emit(&unit.ir) {
            Ok(wasm) => Ok(Artifact {
                kind: ArtifactKind::Wasm,
                bytes: wasm.bytes,
                description: wasm.invariant_note.to_string(),
            }),
            Err(err) => Err(EngineError::new(format!(
                "{}. Product invariant: compile JavaScript directly to Wasm; do not ship interpreter-in-Wasm.",
                err
            ))),
        }
    }

    pub fn emit_c(&self, unit: &CompilationUnit) -> Result<Artifact, EngineError> {
        match porffor_backend_c::emit(&unit.ir) {
            Ok(c) => Ok(Artifact {
                kind: ArtifactKind::C,
                bytes: c.source.into_bytes(),
                description: "shared IR to C artifact".to_string(),
            }),
            Err(err) => Err(EngineError::new(err)),
        }
    }

    pub fn emit_native(
        &self,
        unit: &CompilationUnit,
        target_triple: Option<&str>,
    ) -> Result<Artifact, EngineError> {
        match porffor_backend_native::emit(&unit.ir, target_triple) {
            Ok(native) => Ok(Artifact {
                kind: ArtifactKind::Native,
                bytes: Vec::new(),
                description: format!("native artifact placeholder for {:?}", native.target_triple),
            }),
            Err(err) => Err(EngineError::new(err)),
        }
    }

    pub fn inspect(&self, unit: &CompilationUnit) -> InspectionReport {
        InspectionReport {
            goal: unit.source.goal,
            source_len: unit.ir.source_len,
            stages: unit
                .ir
                .stages
                .iter()
                .map(|stage| match stage {
                    porffor_ir::LoweringStage::ParsedSource => "parsed-source",
                    porffor_ir::LoweringStage::EarlyErrorsPending => "early-errors-pending",
                    porffor_ir::LoweringStage::SpecIrPending => "spec-ir-pending",
                    porffor_ir::LoweringStage::BackendIrPending => "backend-ir-pending",
                    porffor_ir::LoweringStage::WasmCodegenPending => "wasm-codegen-pending",
                })
                .collect(),
            invariants: unit.ir.invariants.clone(),
        }
    }

    fn compile(
        &self,
        source: &str,
        goal: ParseGoal,
        options: CompileOptions,
    ) -> Result<CompilationUnit, EngineError> {
        let source = parse(
            source,
            ParseOptions {
                goal,
                filename: options.filename,
            },
        )
        .map_err(|err| EngineError::new(err.to_string()))?;
        let ir = lower(&source);
        Ok(CompilationUnit { source, ir })
    }

    fn run_compiled_unit(
        &self,
        unit: &CompilationUnit,
        source: &str,
        run: RunOptions,
    ) -> Result<RunOutcome, EngineError> {
        match run.backend {
            ExecutionBackend::SpecExec => {
                let outcome = if unit.source.goal == ParseGoal::Module {
                    porffor_spec_exec::execute_module(
                        source,
                        unit.source.filename.as_deref(),
                        porffor_spec_exec::ModuleHostConfig {
                            module_root: run.module_root.clone().map(Into::into),
                            test_path: run.test_path.clone().map(Into::into),
                        },
                        &run.argv,
                    )
                } else {
                    porffor_spec_exec::execute_script(
                        source,
                        unit.source.filename.as_deref(),
                        &run.argv,
                    )
                }
                .map_err(|err| EngineError::new(err.to_string()))?;

                Ok(RunOutcome {
                    backend_used: ExecutionBackend::SpecExec,
                    note: outcome.note,
                })
            }
            ExecutionBackend::WasmAot => Err(EngineError::new(
                "runtime execution for wasm is not implemented yet; shipped path must stay direct JS->Wasm, not interpreter-in-Wasm",
            )),
        }
    }

    fn run_with_spec_exec(
        &self,
        source: &str,
        filename: Option<&str>,
        goal: ParseGoal,
        run: RunOptions,
    ) -> Result<RunOutcome, EngineError> {
        let outcome = match goal {
            ParseGoal::Module => porffor_spec_exec::execute_module(
                source,
                filename,
                porffor_spec_exec::ModuleHostConfig {
                    module_root: run.module_root.clone().map(Into::into),
                    test_path: run.test_path.clone().map(Into::into),
                },
                &run.argv,
            ),
            ParseGoal::Script => porffor_spec_exec::execute_script(source, filename, &run.argv),
        }
        .map_err(|err| EngineError::new(err.to_string()))?;

        Ok(RunOutcome {
            backend_used: ExecutionBackend::SpecExec,
            note: outcome.note,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> Engine {
        Engine::new(RealmBuilder::new().build())
    }

    #[test]
    fn compile_script_marks_script_goal() {
        let unit = engine()
            .compile_script("let x = 1;", CompileOptions::default())
            .expect("script compile stub should succeed");
        assert_eq!(unit.source.goal, ParseGoal::Script);
        assert!(unit.ir.invariants.contains(&"direct-js-to-wasm-only"));
    }

    #[test]
    fn compile_module_marks_module_goal() {
        let unit = engine()
            .compile_module("export {};", CompileOptions::default())
            .expect("module compile stub should succeed");
        assert_eq!(unit.source.goal, ParseGoal::Module);
    }

    #[test]
    fn wasm_emit_error_mentions_product_rule() {
        let unit = engine()
            .compile_script("1 + 1;", CompileOptions::default())
            .expect("script compile stub should succeed");
        let err = engine()
            .emit_wasm(&unit)
            .expect_err("wasm backend should still be stubbed");
        assert!(err
            .message()
            .contains("compile JavaScript directly to Wasm"));
    }

    #[test]
    fn run_defaults_to_spec_exec() {
        let outcome = engine()
            .run_script("1 + 1;", CompileOptions::default(), RunOptions::default())
            .expect("spec exec should run a simple script");
        assert_eq!(outcome.backend_used, ExecutionBackend::SpecExec);
    }
}
