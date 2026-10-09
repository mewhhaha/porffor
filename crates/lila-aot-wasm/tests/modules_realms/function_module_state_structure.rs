use std::fs;
use std::path::Path;

const SOURCE: &str = include_str!("../../src/emit.rs");
const COMPILATION: &str = include_str!("../../src/emit/body_compilation.rs");
const ENTRY: &str = include_str!("../../src/emit/body_entry.rs");
const SOURCE_FAMILY: &str = concat!(
    include_str!("../../src/emit.rs"),
    include_str!("../../src/emit/body_compilation.rs"),
    include_str!("../../src/emit/body_entry.rs"),
    include_str!("../../src/emit/body_entry/literal_roots.rs"),
    include_str!("../../src/emit/template_objects.rs"),
);

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
        "/// Only a builder owning the planned entry can finish this body.",
    );
    let state = normalized_code(bounded(
        declaration,
        "enum FunctionModuleState<'a> {",
        "\n}",
    ))
    .replace(",)", ")");
    assert_eq!(
        state,
        concat!(
            "Main(&'aFinalizedModuleGlobals,PromiseRejectionPolicy),",
            "PreparedScript(&'aPreparedScriptUnit,&'acrate::function_entry::PlannedFunctionEntry),",
            "Internal(&'acrate::function_entry::PlannedFunctionEntry),RuntimeOperation(RuntimeHelperId),"
        )
    );
    let declaration = normalized_code(declaration);
    assert!(declaration.contains("Self::Main(_,_)=>StaticSignature::Main.parameter_count()"));
    assert!(declaration.contains(
        "Self::Internal(entry)|Self::PreparedScript(_,entry)=>{entry.signature().parameter_count()}"
    ));
    assert!(declaration.contains("Self::RuntimeOperation(helper)=>helper.parameter_types().len()"));
    assert!(declaration
        .contains("Self::Internal(entry)|Self::PreparedScript(_,entry)=>Some(entry.body())"));
    assert!(declaration.contains("Self::Main(_,_)|Self::RuntimeOperation(_)=>None"));
    assert!(!declaration.contains("#["));
    let normalized_source = normalized_code(SOURCE_FAMILY);
    for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
        assert!(!normalized_source.contains(&format!("{capability}forFunctionModuleState")));
    }
    assert!(!SOURCE.contains("== FunctionModuleState::"));
    assert!(!SOURCE.contains("!= FunctionModuleState::"));
    assert!(!SOURCE.contains("matches!(module_state"));

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_in_rust_sources(&source_root, "FunctionModuleState"),
        47
    );
    assert_eq!(
        SOURCE_FAMILY.matches("FunctionModuleState::Main").count(),
        9
    );
    assert_eq!(
        SOURCE_FAMILY
            .matches("FunctionModuleState::PreparedScript")
            .count(),
        12
    );
    assert_eq!(
        SOURCE_FAMILY
            .matches("FunctionModuleState::Internal")
            .count(),
        11
    );
}

#[test]
fn exactly_six_constructors_choose_their_named_module_states() {
    for (start, end, expected) in [
        (
            "    fn new_main(",
            "    fn new_prepared_script(",
            "FunctionModuleState::Main(module_globals, promise_rejection_policy)",
        ),
        (
            "    fn new_prepared_script(",
            "    fn new_function(",
            "FunctionModuleState::PreparedScript(",
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
            "FunctionModuleState::RuntimeOperation",
        ),
        (
            "    fn new_standard_builtin(",
            "    fn new(\n        schema: &'a RuntimeSchema,\n        body: &'a BlockIr,",
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
        "    fn new(\n        schema: &'a RuntimeSchema,\n        body: &'a BlockIr,",
        "    pub(crate) fn emitted_local_count(",
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
            .find("Function::with_parameters(")
            .unwrap()
            < shared_constructor
                .find("            module_state,\n")
                .unwrap()
    );
}

#[test]
fn module_state_storage_and_anchor_projections_are_borrowed_and_exhaustive() {
    assert_eq!(SOURCE.matches("match &module_state {").count(), 2);
    assert!(!SOURCE.contains("let FunctionModuleState::Main("));
    let shared_constructor = bounded(
        SOURCE,
        "    fn new(\n        schema: &'a RuntimeSchema,\n        body: &'a BlockIr,",
        "    pub(crate) fn emitted_local_count(",
    );
    let parameter_policy = bounded(
        shared_constructor,
        "        let parameter_types = match &module_state {",
        "        let return_abi = module_state.return_abi();",
    );
    assert_eq!(normalized_code(parameter_policy), concat!(
        "FunctionModuleState::Main(_,_)=>schema.signature(StaticSignature::Main),",
        "FunctionModuleState::PreparedScript(_,entry)|FunctionModuleState::Internal(entry)=>schema.signature(entry.signature()),",
        "FunctionModuleState::RuntimeOperation(helper)=>helper.definition(schema.layouts()),}.parameters().to_vec();"
    ));
    let construction_policy = bounded(
        shared_constructor,
        "        let hoisted_vars = match &module_state {",
        "        let template_source_body = template_source;",
    );
    assert!(!construction_policy.contains("_ =>"));
    for role in [
        "Main(_, _)",
        "PreparedScript(unit, _)",
        "Internal(_)",
        "RuntimeOperation(_)",
        "PreparedScript(_, _)",
    ] {
        assert!(construction_policy.contains(&format!("FunctionModuleState::{role}")));
    }
    // Only the role's actual planned entry supplies lexical/private roots.
    let roots = bounded(
        shared_constructor,
        "        let current_environment = schema",
        "        Self {",
    );
    assert!(roots.contains("reserve_gc_local::<Environment, Nullable>"));
    assert!(roots.contains("reserve_gc_local::<PrivateEnvironment, Nullable>"));
    assert!(roots.contains(".planned_body_entry()"));
    assert!(roots.contains("entry.lexical_environment().load(schema, &mut owned_body)"));
    assert!(roots.contains("entry.private_environment().load(schema, &mut owned_body)"));
    let clear = bounded(
        ENTRY,
        "    pub(super) fn clear_body_entry_roots(",
        "    /// Starts the body of `helper`",
    );
    for retained_root in [
        "context.clear(function)",
        "entry.clear(self.schema, function)",
        "self.current_environment.set_null(self.schema, function)",
    ] {
        assert!(clear.contains(retained_root));
    }
    assert!(normalized_code(clear)
        .contains("self.current_private_environment.set_null(self.schema,function)"));
    assert_eq!(
        COMPILATION
            .matches("self.clear_body_entry_roots(&mut function);")
            .count(),
        2
    );
}

#[test]
fn prepared_script_context_projections_keep_their_closed_role_policy() {
    assert_eq!(
        SOURCE_FAMILY.matches("match self.module_state {").count(),
        6
    );
    let projections = bounded(
        SOURCE,
        "    pub(crate) fn has_global_script_bindings(&self) -> bool {",
        "    pub(crate) const fn return_abi(&self) -> ReturnAbi {",
    );
    let projections = normalized_code(projections);
    assert!(projections.contains("FunctionModuleState::Main(_,_)=>true"));
    assert!(projections.contains("unit.has_global_variable_environment()"));
    assert!(projections.contains(
        "FunctionModuleState::Internal(_)|FunctionModuleState::RuntimeOperation(_)=>false"
    ));
    assert!(projections.contains("context.derived_constructor_owner()"));
    assert!(projections.contains("self.direct_eval_context.as_ref()?;"));
    assert!(projections.contains("self.direct_eval_context.as_ref()"));
    assert!(normalized_code(SOURCE).contains("entry.private_environment()"));
    assert!(!projections.contains("Some(9)"));
    assert!(!projections.contains("Some(8)"));
    assert!(projections.contains("self.direct_eval_context"));
}

#[test]
fn prepared_script_role_controls_instantiation_and_script_completion() {
    let instantiation = bounded(
        COMPILATION,
        "        self.initialize_direct_eval_execution_context(&mut function)?;",
        "        self.init_runtime_roots(&mut function)?;",
    );
    assert_eq!(
        normalized_code(instantiation),
        concat!(
            "ifletFunctionModuleState::PreparedScript(unit,_)=self.module_state{",
            "self.emit_instantiate_prepared_script_declarations(unit,&mutfunction)?;}"
        )
    );
    let completion = bounded(
        COMPILATION,
        "        if matches!(self.return_abi(), ReturnAbi::MultiValue)",
        "        self.normalize_base_class_constructor_result(&mut function);",
    );
    assert_eq!(
        normalized_code(completion),
        concat!(
            "&&!matches!(self.module_state,FunctionModuleState::PreparedScript(_,_))",
            "&&!self.current_function_meta().is_some_and(|meta|",
            "meta.protocol().class_kind()==ClassFunctionKind::Constructor){",
            "self.completion.value().set_undefined(&mutfunction);}"
        )
    );
}

#[test]
fn only_the_main_export_selects_the_host_rejection_policy() {
    let checkpoint = bounded(
        COMPILATION,
        "            let promise_rejection_policy = match self.module_state {",
        "            self.emit_capture_final_throw_constructor_name(&mut function)?;",
    );
    assert_eq!(
        normalized_code(checkpoint),
        concat!(
            "FunctionModuleState::Main(_,policy)=>policy,",
            "FunctionModuleState::PreparedScript(_,_)|FunctionModuleState::Internal(_)|FunctionModuleState::RuntimeOperation(_)=>{",
            "unreachable!(\"only the main export reports unhandled rejections\")}};",
            "self.emit_report_unhandled_rejection(promise_rejection_policy,&mutfunction)?;"
        )
    );
}
