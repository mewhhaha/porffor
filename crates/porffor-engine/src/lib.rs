use porffor_front::{parse, ParseGoal, ParseOptions, SourceUnit};
use porffor_ir::{lower, ProgramIr, ValueKind};
use wasmi::{Engine as WasmiEngine, Linker, Module as WasmiModule, Store, Value as WasmiValue};

const WASM_RESULT_TAG_EXPORT: &str = "result_tag";

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
    pub can_block: bool,
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
    pub ir_summary: String,
    pub diagnostics: Vec<String>,
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
                    porffor_ir::LoweringStage::AstReparsed => "ast-reparsed",
                    porffor_ir::LoweringStage::ScriptIrBuilt => "script-ir-built",
                    porffor_ir::LoweringStage::UnsupportedFeaturesRecorded => {
                        "unsupported-features-recorded"
                    }
                    porffor_ir::LoweringStage::WasmReady => "wasm-ready",
                })
                .collect(),
            invariants: unit.ir.invariants.clone(),
            ir_summary: unit.ir.ir_summary(),
            diagnostics: unit
                .ir
                .diagnostics
                .iter()
                .map(|diagnostic| diagnostic.message.clone())
                .collect(),
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
                        run.can_block,
                    )
                } else {
                    porffor_spec_exec::execute_script(
                        source,
                        unit.source.filename.as_deref(),
                        &run.argv,
                        run.can_block,
                    )
                }
                .map_err(|err| EngineError::new(err.to_string()))?;

                Ok(RunOutcome {
                    backend_used: ExecutionBackend::SpecExec,
                    note: outcome.note,
                })
            }
            ExecutionBackend::WasmAot => self.run_with_wasm_aot(unit),
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
                run.can_block,
            ),
            ParseGoal::Script => {
                porffor_spec_exec::execute_script(source, filename, &run.argv, run.can_block)
            }
        }
        .map_err(|err| EngineError::new(err.to_string()))?;

        Ok(RunOutcome {
            backend_used: ExecutionBackend::SpecExec,
            note: outcome.note,
        })
    }

    fn run_with_wasm_aot(&self, unit: &CompilationUnit) -> Result<RunOutcome, EngineError> {
        let artifact = porffor_aot_wasm::emit(&unit.ir).map_err(|err| {
            EngineError::new(format!(
                "{}. Product invariant: compile JavaScript directly to Wasm; do not ship interpreter-in-Wasm.",
                err
            ))
        })?;

        let engine = WasmiEngine::default();
        let module = WasmiModule::new(&engine, &artifact.bytes[..])
            .map_err(|err| EngineError::new(format!("wasmi module validation failed: {err}")))?;
        let mut store = Store::new(&engine, ());
        let linker = Linker::new(&engine);
        let instance = linker
            .instantiate(&mut store, &module)
            .and_then(|pre| pre.start(&mut store))
            .map_err(|err| EngineError::new(format!("wasmi instantiate failed: {err}")))?;
        let main = instance
            .get_typed_func::<(), i64>(&store, "main")
            .map_err(|err| EngineError::new(format!("wasmi export lookup failed: {err}")))?;
        let payload = main
            .call(&mut store, ())
            .map_err(|err| EngineError::new(format!("wasmi execution trapped: {err}")))?;
        let result_kind = instance
            .get_global(&store, WASM_RESULT_TAG_EXPORT)
            .ok_or_else(|| EngineError::new("wasmi export lookup failed: missing result_tag"))?
            .get(&store);
        let WasmiValue::I32(result_tag) = result_kind else {
            return Err(EngineError::new(
                "wasm result_tag export had unexpected type",
            ));
        };
        let result_kind = ValueKind::from_tag(result_tag)
            .ok_or_else(|| EngineError::new(format!("unknown wasm result tag: {result_tag}")))?;
        let note = render_wasm_completion(
            result_kind,
            payload,
            instance.get_memory(&store, "memory"),
            &store,
        )?;

        Ok(RunOutcome {
            backend_used: ExecutionBackend::WasmAot,
            note,
        })
    }
}

fn render_wasm_completion(
    kind: ValueKind,
    payload: i64,
    memory: Option<wasmi::Memory>,
    store: &Store<()>,
) -> Result<String, EngineError> {
    let rendered = match kind {
        ValueKind::Undefined => "undefined".to_string(),
        ValueKind::Null => "null".to_string(),
        ValueKind::Boolean => {
            if payload == 0 {
                "false".to_string()
            } else {
                "true".to_string()
            }
        }
        ValueKind::Number => format!("{}", f64::from_bits(payload as u64)),
        ValueKind::String => {
            let offset = ((payload as u64) >> 32) as usize;
            let len = ((payload as u64) & 0xFFFF_FFFF) as usize;
            let memory = memory.ok_or_else(|| {
                EngineError::new("wasm string result needs exported memory, but none exists")
            })?;
            let mut bytes = vec![0; len];
            memory
                .read(store, offset, &mut bytes)
                .map_err(|err| EngineError::new(format!("failed to read wasm memory: {err}")))?;
            String::from_utf8(bytes).map_err(|err| {
                EngineError::new(format!("wasm string result is not utf-8: {err}"))
            })?
        }
        ValueKind::Object => format!("handle@{}", payload as u64),
        ValueKind::Array => format!("handle@{}", payload as u64),
        ValueKind::Function => format!("handle@{}", payload as u64),
        ValueKind::Arguments => format!("handle@{}", payload as u64),
        ValueKind::Dynamic => {
            return Err(EngineError::new(
                "wasm completion used dynamic tag; expected concrete runtime tag",
            ));
        }
    };
    Ok(format!(
        "wasm-aot completion: {}({rendered})",
        kind.as_str()
    ))
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
    fn wasm_emit_succeeds_for_supported_script() {
        let unit = engine()
            .compile_script("1 + 1;", CompileOptions::default())
            .expect("script compile should succeed");
        let artifact = engine().emit_wasm(&unit).expect("wasm emit should succeed");
        assert_eq!(artifact.kind, ArtifactKind::Wasm);
        assert!(!artifact.bytes.is_empty());
    }

    #[test]
    fn wasm_emit_reports_unsupported_slice_precisely() {
        let unit = engine()
            .compile_script(
                "function f({ x }) { return x; }",
                CompileOptions::default(),
            )
            .expect("script compile should succeed");
        let err = engine()
            .emit_wasm(&unit)
            .expect_err("unsupported slice should fail");
        assert!(err
            .message()
            .contains("unsupported in porffor wasm-aot first slice"));
    }

    #[test]
    fn run_defaults_to_spec_exec() {
        let outcome = engine()
            .run_script("1 + 1;", CompileOptions::default(), RunOptions::default())
            .expect("spec exec should run a simple script");
        assert_eq!(outcome.backend_used, ExecutionBackend::SpecExec);
    }

    #[test]
    fn wasm_backend_runs_supported_script() {
        let outcome = engine()
            .run_script(
                "let x = 40; const y = 2; x + y;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run supported script");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(outcome.note.contains("number(42"));
    }

    #[test]
    fn wasm_backend_supports_remainder() {
        let outcome = engine()
            .run_script(
                "7 % 3;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run remainder");
        assert!(outcome.note.contains("number(1"));
    }

    #[test]
    fn wasm_backend_supports_assignment_and_if() {
        let outcome = engine()
            .run_script(
                "let x = 0; if (!x) { x = 5; } x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run assignment and if");
        assert!(outcome.note.contains("number(5"));
    }

    #[test]
    fn wasm_backend_rejects_const_assignment_precisely() {
        let err = engine()
            .run_script(
                "const x = 1; x = 2;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("const assignment should stay unsupported");
        assert!(err
            .message()
            .contains("unsupported in porffor wasm-aot first slice: assignment to const binding"));
    }

    #[test]
    fn wasm_backend_supports_hoisted_function_calls() {
        let outcome = engine()
            .run_script(
                "add(1, 2); function add(x, y) { return x + y; }",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run hoisted function call");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_direct_recursion() {
        let outcome = engine()
            .run_script(
                "function up(n) { if (n === 0) { return 0; } return up(n - 1) + 1; } up(3);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run recursion");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_object_property_read() {
        let outcome = engine()
            .run_script(
                "let o = { x: 1, y: 2 }; o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run object property read");
        assert!(outcome.note.contains("number(1"));
    }

    #[test]
    fn wasm_backend_supports_array_write_and_read() {
        let outcome = engine()
            .run_script(
                "let a = [1]; a[2] = 4; a[2];",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run array write and read");
        assert!(outcome.note.contains("number(4"));
    }

    #[test]
    fn wasm_backend_supports_missing_heap_reads() {
        let object_outcome = engine()
            .run_script(
                "let o = {}; o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should return undefined for missing property");
        assert!(object_outcome.note.contains("undefined("));

        let array_outcome = engine()
            .run_script(
                "let a = [1]; a[9];",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should return undefined for missing index");
        assert!(array_outcome.note.contains("undefined("));
    }

    #[test]
    fn wasm_backend_supports_object_bracket_write() {
        let outcome = engine()
            .run_script(
                "let o = { x: 1 }; o[\"x\"] = 3; o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should support object bracket write");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_object_return_from_function() {
        let outcome = engine()
            .run_script(
                "function box(x) { let o = { x: x }; return o; } let o = box(2); o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run object-return function");
        assert!(outcome.note.contains("number(2"));
    }

    #[test]
    fn wasm_backend_supports_chained_object_access() {
        let outcome = engine()
            .run_script(
                "let o = { inner: { x: 1 } }; o.inner.x = 4; o.inner.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run chained object access");
        assert!(outcome.note.contains("number(4"));
    }

    #[test]
    fn wasm_backend_supports_call_result_property_and_array_length() {
        let property_outcome = engine()
            .run_script(
                "function box(x) { let o = { x: x }; return o; } box(2).x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run call-result property read");
        assert!(property_outcome.note.contains("number(2"));

        let array_outcome = engine()
            .run_script(
                "let a = [1, 2, 3]; a.length;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run array length read");
        assert!(array_outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_nested_array_object_reads() {
        let outcome = engine()
            .run_script(
                "function make() { let o = { items: [{ x: 1 }, { x: 3 }] }; return o; } make().items[1].x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run nested heap reads");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_function_values_and_indirect_calls() {
        let alias_outcome = engine()
            .run_script(
                "function inc(x) { return x + 1; } let g = inc; g(2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run indirect call through alias");
        assert!(alias_outcome.note.contains("number(3"));

        let passthrough_outcome = engine()
            .run_script(
                "function inc(x) { return x + 1; } function pick() { return inc; } let g = pick(); g(2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run function pass-through");
        assert!(passthrough_outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_method_calls_and_this() {
        let method_outcome = engine()
            .run_script(
                "function inc(x) { return x + 1; } let o = { f: inc }; o.f(2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run method call");
        assert!(method_outcome.note.contains("number(3"));

        let bracket_outcome = engine()
            .run_script(
                "function inc(x) { return x + 1; } let o = { f: inc }; o[\"f\"](2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run bracket method call");
        assert!(bracket_outcome.note.contains("number(3"));

        let this_outcome = engine()
            .run_script(
                "function getX() { return this.x; } let o = { x: 3, f: getX }; o.f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run method call with this");
        assert!(this_outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_array_function_calls_and_undefined_this() {
        let array_outcome = engine()
            .run_script(
                "function inc(x) { return x + 1; } let a = [inc]; a[0](2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run array function call");
        assert!(array_outcome.note.contains("number(3"));

        let bare_this_outcome = engine()
            .run_script(
                "function check() { return this; } check();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should keep bare-call this undefined");
        assert!(bare_this_outcome.note.contains("undefined("));
    }

    #[test]
    fn wasm_backend_grows_heap_beyond_old_fixed_cap() {
        let source = format!(
            "let o = {{}}; {} o.k64;",
            (0..65)
                .map(|index| format!("o[\"k{index}\"] = {index};"))
                .collect::<Vec<_>>()
                .join(" ")
        );
        let outcome = engine()
            .run_script(
                &source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should grow heap");
        assert!(outcome.note.contains("number(64"));
    }

    #[test]
    fn wasm_backend_rejects_property_access_on_dynamic_target() {
        let err = engine()
            .run_script(
                "let v; if (true) { v = 1; } else { v = { x: 1 }; } v.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("dynamic property access should stay unsupported");
        assert!(err
            .message()
            .contains("unsupported in porffor wasm-aot first slice"));
    }

    #[test]
    fn wasm_backend_rejects_method_calls_and_array_length_brackets() {
        let method_err = engine()
            .run_script(
                "let obj = { f: 1 }; obj.f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("method call should stay unsupported");
        assert!(method_err
            .message()
            .contains("unsupported in porffor wasm-aot first slice: indirect call"));

        let length_err = engine()
            .run_script(
                "let a = [1]; a[\"length\"];",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("array length bracket should stay unsupported");
        assert!(length_err
            .message()
            .contains("unsupported in porffor wasm-aot first slice: array index must be number"));
    }

    #[test]
    fn wasm_backend_supports_script_closure_capture() {
        let outcome = engine()
            .run_script(
                "let x = 1; function f() { return x; } f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("script closure should run");
        assert!(outcome.note.contains("number(1"));
    }

    #[test]
    fn wasm_backend_supports_nested_function_declaration() {
        let outcome = engine()
            .run_script(
                "function outer() { let x = 1; function inner() { return x + 1; } return inner(); } outer();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("nested function declaration should run");
        assert!(outcome.note.contains("number(2"));
    }

    #[test]
    fn wasm_backend_supports_closure_mutation() {
        let outcome = engine()
            .run_script(
                "function outer() { let x = 1; function inc() { x = x + 1; return x; } inc(); return inc(); } outer();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("closure mutation should run");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_anonymous_function_expression() {
        let outcome = engine()
            .run_script(
                "let f = function (x) { return x + 1; }; f(2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("anonymous function expression should run");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_returned_closure_call() {
        let outcome = engine()
            .run_script(
                "function outer() { let x = 2; return function (y) { return x + y; }; } let f = outer(); f(3);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("returned closure should run");
        assert!(outcome.note.contains("number(5"));
    }

    #[test]
    fn wasm_backend_supports_object_closure_method() {
        let outcome = engine()
            .run_script(
                "function outer() { let x = 3; return { f: function () { return x; } }; } let o = outer(); o.f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("object closure method should run");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_var_closure_capture() {
        let outcome = engine()
            .run_script(
                "function outer() { var x = 1; return function () { return x; }; } let f = outer(); f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("var closure should run");
        assert!(outcome.note.contains("number(1"));
    }

    #[test]
    fn wasm_backend_supports_nested_recursive_closure_call() {
        let outcome = engine()
            .run_script(
                "function outer(n) { function loop(x) { if (x === 0) { return 0; } return loop(x - 1) + 1; } return loop(n); } outer(3);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("nested recursive function should run");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_arrow_function_basic() {
        let outcome = engine()
            .run_script(
                "let f = x => x + 1; f(2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("arrow function should run");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_arrow_function_block_body() {
        let outcome = engine()
            .run_script(
                "let f = x => { return x + 1; }; f(2);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("arrow block body should run");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_named_function_expression_recursion() {
        let outcome = engine()
            .run_script(
                "let f = function fact(n) { if (n === 0) { return 1; } return n * fact(n - 1); }; f(4);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("named function expression should run");
        assert!(outcome.note.contains("number(24"));
    }

    #[test]
    fn wasm_backend_supports_returned_arrow_closure_call() {
        let outcome = engine()
            .run_script(
                "function outer(x) { return y => x + y; } let f = outer(2); f(3);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("returned arrow closure should run");
        assert!(outcome.note.contains("number(5"));
    }

    #[test]
    fn wasm_backend_supports_arrow_lexical_this() {
        let outcome = engine()
            .run_script(
                "function make() { return () => this.x; } let o = { x: 3, f: make }; let g = o.f(); g();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("arrow lexical this should run");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_object_literal_shorthand_methods_and_accessors() {
        let shorthand_outcome = engine()
            .run_script(
                "let x = 1; let o = { x }; o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("object shorthand should run");
        assert!(shorthand_outcome.note.contains("number(1"));

        let method_outcome = engine()
            .run_script(
                "let o = { x: 3, f() { return this.x; } }; o.f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("concise method should run");
        assert!(method_outcome.note.contains("number(3"));

        let closure_method_outcome = engine()
            .run_script(
                "function make(x) { return { f() { return x; } }; } make(2).f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("closure method should run");
        assert!(closure_method_outcome.note.contains("number(2"));

        let getter_outcome = engine()
            .run_script(
                "let o = { get x() { return 1; } }; o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("getter should run");
        assert!(getter_outcome.note.contains("number(1"));

        let setter_outcome = engine()
            .run_script(
                "let o = { _x: 0, set x(v) { this._x = v; } }; o.x = 3; o._x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("setter should run");
        assert!(setter_outcome.note.contains("number(3"));

        let pair_outcome = engine()
            .run_script(
                "let o = { _x: 0, get x() { return this._x; }, set x(v) { this._x = v; } }; o.x = 4; o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("getter setter pair should run");
        assert!(pair_outcome.note.contains("number(4"));

        let arrow_method_outcome = engine()
            .run_script(
                "let o = { x: 3, f() { return (() => this.x)(); } }; o.f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("arrow in method should keep lexical this");
        assert!(arrow_method_outcome.note.contains("number(3"));

        let returned_accessor_outcome = engine()
            .run_script(
                "function make() { return { get x() { return 5; } }; } let o = make(); o.x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("returned accessor object should run");
        assert!(returned_accessor_outcome.note.contains("number(5"));
    }

    #[test]
    fn wasm_backend_supports_default_rest_and_arguments_core() {
        for (source, expected, label) in [
            (
                "function f(x = 1) { return x; } f();",
                "number(1",
                "default param basic",
            ),
            (
                "function f(x, y = x + 1) { return y; } f(2);",
                "number(3",
                "default param from earlier param",
            ),
            (
                "let f = (x = 1) => x + 1; f();",
                "number(2",
                "arrow default param",
            ),
            (
                "function third(...xs) { return xs[2]; } third(1, 2, 3);",
                "number(3",
                "rest param element",
            ),
            (
                "function len(...xs) { return xs.length; } len(1, 2, 3);",
                "number(3",
                "rest param length",
            ),
            (
                "function f(a, b) { return arguments.length; } f(1, 2, 3);",
                "number(3",
                "arguments length",
            ),
            (
                "function f() { return arguments[1]; } f(1, 2, 3);",
                "number(2",
                "arguments indexed read",
            ),
            (
                "function f(x) { arguments[0] = 3; return x; } f(1);",
                "number(3",
                "mapped arguments write to param",
            ),
            (
                "function f(x) { x = 4; return arguments[0]; } f(1);",
                "number(4",
                "mapped param write to arguments",
            ),
            (
                "function f(x = 1) { arguments[0] = 3; return x; } f(2);",
                "number(2",
                "unmapped default param arguments",
            ),
            (
                "function f(...xs) { arguments[0] = 9; return xs[0]; } f(1, 2);",
                "number(1",
                "unmapped rest arguments",
            ),
            (
                "function outer() { return (() => arguments[0])(); } outer(3);",
                "number(3",
                "arrow lexical arguments",
            ),
            (
                "let o = { x: 2, f(y = this.x) { return y; } }; o.f();",
                "number(2",
                "method default with this",
            ),
        ] {
            let outcome = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .unwrap_or_else(|_| panic!("{label} should run"));
            assert!(
                outcome.note.contains(expected),
                "{label} produced unexpected note: {}",
                outcome.note
            );
        }
    }

    #[test]
    fn wasm_backend_rejects_unsupported_param_and_arguments_forms() {
        for source in [
            "function f({ x }) { return x; }",
            "let f = ({ x }) => x;",
            "function f(x, x) { return x; }",
            "function f(x = y, y = 1) { return x; }",
            "function f(x = x) { return x; }",
            "let f = () => arguments;",
            "function f() { return arguments.callee; }",
            "({ get x(a) { return a; } })",
            "({ set x(v = 1) {} })",
        ] {
            let err = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect_err("unsupported param or arguments form should stay unsupported");
            assert!(!err.message().trim().is_empty());
        }

        let top_level_this_err = engine()
            .run_script(
                "let f = () => this;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("top-level arrow this should stay unsupported");
        assert!(top_level_this_err
            .message()
            .contains("unsupported in porffor wasm-aot first slice"));
    }

    #[test]
    fn wasm_backend_rejects_unsupported_object_literal_method_forms() {
        for source in [
            "({ [x]: 1 })",
            "({ ...x })",
            "({ async f() {} })",
            "({ *f() {} })",
            "({ get x(v) { return v; } })",
            "({ set x() {} })",
            "({ f({ x }) {} })",
            "({ f() { return super.x; } })",
        ] {
            let err = engine()
                .run_script(
                    source,
                    CompileOptions::default(),
                    RunOptions {
                        backend: ExecutionBackend::WasmAot,
                        ..RunOptions::default()
                    },
                )
                .expect_err("unsupported object literal form should stay unsupported");
            assert!(!err.message().trim().is_empty());
        }
    }

    #[test]
    fn wasm_backend_supports_implicit_undefined_function_return() {
        let outcome = engine()
            .run_script(
                "function f() { let x = 1; } f();",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run implicit undefined function");
        assert!(outcome.note.contains("undefined("));
    }

    #[test]
    fn wasm_backend_supports_while_loop() {
        let outcome = engine()
            .run_script(
                "let i = 0; while (i < 3) { i = i + 1; } i;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run while loop");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_do_while_loop() {
        let outcome = engine()
            .run_script(
                "let i = 0; do { i = i + 1; } while (i < 3); i;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run do while loop");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_supports_for_break_and_continue() {
        let outcome = engine()
            .run_script(
                "let i = 0; let sum = 0; for (; i < 5; i = i + 1) { if (i === 2) { continue; } if (i === 4) { break; } sum = sum + i; } sum;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run for loop");
        assert!(outcome.note.contains("number(4"));
    }

    #[test]
    fn wasm_backend_supports_update_and_compound_assignment() {
        let outcome = engine()
            .run_script(
                "let sum = 0; for (let i = 0; i < 4; i++) { sum += i; } sum;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run updates and compound assignment");
        assert!(outcome.note.contains("number(6"));
    }

    #[test]
    fn wasm_backend_preserves_postfix_result() {
        let outcome = engine()
            .run_script(
                "let i = 2; let x = i++; x + i;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should preserve postfix value");
        assert!(outcome.note.contains("number(5"));
    }

    #[test]
    fn wasm_backend_supports_switch_and_labels() {
        let outcome = engine()
            .run_script(
                "let x = 0; outer: while (x < 3) { x += 1; switch (x) { case 1: continue outer; case 2: debugger; break outer; default: x = 9; } } x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run switch and labels");
        assert!(outcome.note.contains("number(2"));
    }

    #[test]
    fn wasm_backend_supports_default_in_middle_switch() {
        let outcome = engine()
            .run_script(
                "let x = 0; switch (3) { case 1: x = 1; break; default: x = 9; break; case 3: x = 3; } x;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should run default-in-middle switch");
        assert!(outcome.note.contains("number(3"));
    }

    #[test]
    fn wasm_backend_rejects_const_update_precisely() {
        let err = engine()
            .run_script(
                "const x = 1; x++;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("const update should stay unsupported");
        assert!(err
            .message()
            .contains("unsupported in porffor wasm-aot first slice: update of const binding"));
    }

    #[test]
    fn wasm_backend_rejects_label_on_unsupported_statement_kind_precisely() {
        let err = engine()
            .run_script(
                "label: 1;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("unsupported label target should stay unsupported");
        assert!(err.message().contains(
            "unsupported in porffor wasm-aot first slice: label on unsupported statement kind"
        ));
    }

    #[test]
    fn wasm_backend_supports_hoisted_var() {
        let outcome = engine()
            .run_script(
                "x; var x = 1;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should support hoisted var");
        assert!(outcome.note.contains("undefined("));
    }

    #[test]
    fn wasm_backend_supports_var_in_for_and_duplicate_var() {
        let outcome = engine()
            .run_script(
                "var sum = 0; for (var i = 0; i < 4; i++) { sum += i; } var sum; sum;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should support var in for");
        assert!(outcome.note.contains("number(6"));
    }

    #[test]
    fn wasm_backend_rejects_unknown_kind_numeric_use() {
        let err = engine()
            .run_script(
                "var x; if (true) { x = 1; } else { x = \"a\"; } x + 1;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect_err("unknown kind numeric use should stay unsupported");
        assert!(err
            .message()
            .contains("unsupported in porffor wasm-aot first slice"));
    }
}
