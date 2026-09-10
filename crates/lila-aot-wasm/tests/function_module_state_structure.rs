use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../src/emit.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized_code(source: &str) -> String {
    let source = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut normalized = String::new();
    let mut in_string = false;
    let mut escaped = false;
    for character in source.chars() {
        if in_string {
            normalized.push(character);
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
        } else if character == '"' {
            in_string = true;
            normalized.push(character);
        } else if !character.is_whitespace() {
            normalized.push(character);
        }
    }
    normalized
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
fn function_module_state_is_the_exact_private_no_capability_domain() {
    let declaration = bounded(
        SOURCE,
        concat!(
            "pub(crate) enum ReturnAbi {\n",
            "    MainExport,\n",
            "    MultiValue,\n",
            "}\n"
        ),
        "/// Closed inputs for compiling the one exported main body.",
    );
    assert_eq!(
        normalized_code(declaration),
        concat!(
            "enumFunctionModuleState<'a>{Main(&'aFinalizedModuleGlobals),",
            "PreparedScript(&'aPreparedScriptUnit),Internal,}",
            "implFunctionModuleState<'_>{constfnparameter_count(&self)->usize{matchself{",
            "Self::Main(_)=>0,Self::Internal=>JS_FUNCTION_PARAM_COUNT,",
            "Self::PreparedScript(_)=>PREPARED_SCRIPT_PARAM_COUNT,}}",
            "constfnreturn_abi(&self)->ReturnAbi{matchself{",
            "Self::Main(_)=>ReturnAbi::MainExport,",
            "Self::Internal|Self::PreparedScript(_)=>ReturnAbi::MultiValue,}}}"
        )
    );
    assert!(!declaration.contains("#["));
    let normalized_source = normalized_code(SOURCE);
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!normalized_source.contains(&format!("{capability}forFunctionModuleState")));
    }
    assert!(!SOURCE.contains("== FunctionModuleState::"));
    assert!(!SOURCE.contains("!= FunctionModuleState::"));
    assert!(!SOURCE.contains("matches!(module_state"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "FunctionModuleState"),
        36
    );
    assert_eq!(SOURCE.matches("FunctionModuleState::Main").count(), 8);
    assert_eq!(
        SOURCE
            .matches("FunctionModuleState::PreparedScript")
            .count(),
        13
    );
    assert_eq!(SOURCE.matches("FunctionModuleState::Internal").count(), 11);
}

#[test]
fn exactly_six_constructors_choose_their_named_module_states() {
    for (start, end, expected) in [
        (
            "    fn new_main(",
            "    fn new_prepared_script(",
            "FunctionModuleState::Main(module_globals)",
        ),
        (
            "    fn new_prepared_script(",
            "    fn new_function(",
            "FunctionModuleState::PreparedScript(unit)",
        ),
        (
            "    fn new_function(",
            "    fn new_host_builtin(",
            "FunctionModuleState::Internal",
        ),
        (
            "    fn new_host_builtin(",
            "    fn new_runtime_operation_helper(",
            "FunctionModuleState::Internal",
        ),
        (
            "    fn new_runtime_operation_helper(",
            "    fn new_standard_builtin(",
            "FunctionModuleState::Internal",
        ),
        (
            "    fn new_standard_builtin(",
            "    fn new(\n        body: &'a BlockIr,",
            "FunctionModuleState::Internal",
        ),
    ] {
        let constructor = bounded(SOURCE, start, end);
        assert_eq!(constructor.matches(expected).count(), 1, "{start}");
        assert_eq!(
            constructor.matches("FunctionModuleState::").count(),
            1,
            "{start}"
        );
    }

    let builder = bounded(
        SOURCE,
        "pub(crate) struct FunctionBuilder<'a> {",
        "impl<'a> FunctionBuilder<'a> {",
    );
    assert_eq!(
        builder
            .matches("module_state: FunctionModuleState<'a>,")
            .count(),
        1
    );

    let shared_constructor = bounded(
        SOURCE,
        "    fn new(\n        body: &'a BlockIr,",
        "    /// Wasm function index of the shared object-read runtime helper.",
    );
    assert_eq!(
        shared_constructor
            .matches("module_state: FunctionModuleState<'a>,")
            .count(),
        1
    );
    assert_eq!(
        shared_constructor
            .matches("            module_state,\n")
            .count(),
        1
    );
    assert!(
        shared_constructor
            .find("let return_abi = module_state.return_abi();")
            .unwrap()
            < shared_constructor
                .find("            module_state,\n")
                .unwrap()
    );
    assert!(
        shared_constructor
            .find("let param_local_count = module_state.parameter_count() as u32;")
            .unwrap()
            < shared_constructor
                .find("            module_state,\n")
                .unwrap()
    );
}

#[test]
fn module_state_storage_and_anchor_projections_are_borrowed_and_exhaustive() {
    assert_eq!(SOURCE.matches("match &module_state {").count(), 1);
    assert_eq!(SOURCE.matches("match &self.module_state {").count(), 2);
    assert!(!SOURCE.contains("let FunctionModuleState::Main(module_globals) = self.module_state"));

    let shared_constructor = bounded(
        SOURCE,
        "    fn new(\n        body: &'a BlockIr,",
        "    /// Wasm function index of the shared object-read runtime helper.",
    );
    let construction_policy = bounded(
        shared_constructor,
        "        let return_abi = module_state.return_abi();",
        "        let self_binding_local_count = usize::from(self_binding_name.is_some());",
    );
    assert_eq!(
        normalized_code(construction_policy),
        concat!(
            "lethoisted_vars=match&module_state{FunctionModuleState::Main(_)=>",
            "script_global_bindings.expect(\"main builder must carry the global binding plan\")",
            ".main_frame_write_bindings().map(|binding|binding.name.clone()).collect(),",
            "FunctionModuleState::PreparedScript(unit)ifunit.has_global_variable_environment()=>{",
            "unit.global_bindings.main_frame_write_bindings()",
            ".map(|binding|binding.name.clone()).collect()}",
            "FunctionModuleState::Internal|FunctionModuleState::PreparedScript(_)=>{",
            "collect_hoisted_vars_block_root(body)}};"
        )
    );

    let initialize = bounded(
        SOURCE,
        "    fn initialize_runtime_gc_anchor_root(&self, function: &mut Function) {",
        "    /// Verifies and clears the capability root on a real main exit.",
    );
    assert_eq!(
        normalized_code(initialize),
        concat!(
            "letmodule_globals=match&self.module_state{",
            "FunctionModuleState::Main(module_globals)=>module_globals,",
            "FunctionModuleState::Internal|FunctionModuleState::PreparedScript(_)=>return,};",
            "module_globals.emit_initialize_anchor_root(function);}"
        )
    );

    let verify = bounded(
        SOURCE,
        "    pub(crate) fn verify_and_clear_runtime_gc_anchor_root(",
        "    fn ensure_heap_ptr_after_static_data(&self, function: &mut Function) {",
    );
    assert_eq!(
        normalized_code(verify),
        concat!(
            "&self,function:&mutFunction){letmodule_globals=match&self.module_state{",
            "FunctionModuleState::Main(module_globals)=>module_globals,",
            "FunctionModuleState::Internal|FunctionModuleState::PreparedScript(_)=>return,};",
            "module_globals.emit_verify_and_clear_anchor_root(function);}"
        )
    );
}

#[test]
fn prepared_script_context_projections_keep_their_closed_role_policy() {
    assert_eq!(SOURCE.matches("match self.module_state {").count(), 4);
    let projections = bounded(
        SOURCE,
        "    pub(crate) fn has_global_script_bindings(&self) -> bool {",
        "    pub(crate) const fn return_abi(&self) -> ReturnAbi {",
    );
    assert_eq!(
        normalized_code(projections),
        concat!(
            "matchself.module_state{FunctionModuleState::Main(_)=>true,",
            "FunctionModuleState::PreparedScript(unit)=>unit.has_global_variable_environment(),",
            "FunctionModuleState::Internal=>false,}}",
            "pub(crate)fndirect_eval_derived_constructor_owner(&self)->Option<&FunctionId>{",
            "matchself.module_state{FunctionModuleState::PreparedScript(unit)=>match&unit.kind{",
            "PreparedScriptKind::DirectEval(context)=>context.derived_constructor_owner(),",
            "PreparedScriptKind::RealmScript|PreparedScriptKind::IndirectEval=>None,},",
            "FunctionModuleState::Main(_)|FunctionModuleState::Internal=>{",
            "self.captured_direct_eval_execution_context_local?;",
            "letfunction_id=self.function_id.as_ref().expect(\"capturing arrow has an id\");",
            "letunit=self.functions.prepared_scripts().iter().find_map(|entry|match&entry.outcome{",
            "PreparedScriptOutcome::Executable(unit)ifunit.function_ids.contains(function_id)=>{",
            "Some(unit)}PreparedScriptOutcome::Executable(_)|",
            "PreparedScriptOutcome::DeferredSyntaxError{..}=>None,})",
            ".expect(\"capturing arrow belongs to an independently prepared Script\");",
            "letPreparedScriptKind::DirectEval(context)=&unit.kindelse{",
            "panic!(\"only direct eval supplies a caller execution context\");};",
            "context.derived_constructor_owner()}}}",
            "pub(crate)fndirect_eval_execution_context_local(&self)->Option<u32>{",
            "matchself.module_state{FunctionModuleState::PreparedScript(unit)",
            "ifmatches!(&unit.kind,PreparedScriptKind::DirectEval(_))=>{Some(9)}",
            "FunctionModuleState::Main(_)|FunctionModuleState::Internal|",
            "FunctionModuleState::PreparedScript(_)=>{",
            "self.captured_direct_eval_execution_context_local}}}",
            "pub(crate)fndirect_eval_private_environment_param_local(&self)->Option<u32>{",
            "matchself.module_state{FunctionModuleState::PreparedScript(unit)",
            "ifmatches!(&unit.kind,PreparedScriptKind::DirectEval(_))=>{Some(8)}",
            "FunctionModuleState::Main(_)|FunctionModuleState::Internal|",
            "FunctionModuleState::PreparedScript(_)=>None,}}"
        )
    );
}

#[test]
fn prepared_script_role_controls_instantiation_and_script_completion() {
    let instantiation = bounded(
        SOURCE,
        "        self.initialize_direct_eval_execution_context(&mut function)?;",
        "        self.init_runtime_roots(&mut function)?;",
    );
    assert_eq!(
        normalized_code(instantiation),
        concat!(
            "ifletFunctionModuleState::PreparedScript(unit)=self.module_state{",
            "self.emit_instantiate_prepared_script_declarations(unit,&mutfunction)?;}"
        )
    );
    let completion = bounded(
        SOURCE,
        "        if matches!(self.return_abi(), ReturnAbi::MultiValue)",
        "        self.normalize_base_class_constructor_result(&mut function);",
    );
    assert_eq!(
        normalized_code(completion),
        concat!(
            "&&!matches!(self.module_state,FunctionModuleState::PreparedScript(_))",
            "&&!self.current_function_meta().is_some_and(|meta|",
            "meta.protocol.class_kind()==ClassFunctionKind::Constructor){",
            "self.emit_statement_result(&mutfunction,ValueKind::Undefined);}"
        )
    );
}
