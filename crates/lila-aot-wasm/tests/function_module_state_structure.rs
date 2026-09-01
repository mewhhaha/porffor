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
            "enumFunctionModuleState<'a>{Main(&'aFinalizedModuleGlobals),Internal,}",
            "implFunctionModuleState<'_>{constfnreturn_abi(&self)->ReturnAbi{matchself{",
            "Self::Main(_)=>ReturnAbi::MainExport,Self::Internal=>ReturnAbi::MultiValue,}}}"
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
        15
    );
    assert_eq!(SOURCE.matches("FunctionModuleState::Main").count(), 4);
    assert_eq!(SOURCE.matches("FunctionModuleState::Internal").count(), 7);
}

#[test]
fn exactly_five_constructors_choose_their_named_module_states() {
    for (start, end, expected, rejected) in [
        (
            "    fn new_main(",
            "    fn new_function(",
            "FunctionModuleState::Main(module_globals)",
            "FunctionModuleState::Internal",
        ),
        (
            "    fn new_function(",
            "    fn new_host_builtin(",
            "FunctionModuleState::Internal",
            "FunctionModuleState::Main",
        ),
        (
            "    fn new_host_builtin(",
            "    fn new_runtime_operation_helper(",
            "FunctionModuleState::Internal",
            "FunctionModuleState::Main",
        ),
        (
            "    fn new_runtime_operation_helper(",
            "    fn new_standard_builtin(",
            "FunctionModuleState::Internal",
            "FunctionModuleState::Main",
        ),
        (
            "    fn new_standard_builtin(",
            "    fn new(\n        body: &'a BlockIr,",
            "FunctionModuleState::Internal",
            "FunctionModuleState::Main",
        ),
    ] {
        let constructor = bounded(SOURCE, start, end);
        assert_eq!(constructor.matches(expected).count(), 1, "{start}");
        assert!(!constructor.contains(rejected), "{start}");
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
}

#[test]
fn all_four_module_state_projections_are_borrowed_and_exhaustive() {
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
            ".main_frame_cache_bindings().map(|binding|binding.name.clone()).collect(),",
            "FunctionModuleState::Internal=>collect_hoisted_vars_block_root(body),};"
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
            "FunctionModuleState::Internal=>return,};",
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
            "FunctionModuleState::Internal=>return,};",
            "module_globals.emit_verify_and_clear_anchor_root(function);}"
        )
    );
}
