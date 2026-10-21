const ENGINE_SOURCE: &str = include_str!("../../src/lib.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

#[test]
fn every_backend_dispatcher_exhaustively_routes_the_closed_domain() {
    for (start, end, goal) in [
        ("pub fn run_script(", "pub fn run_module(", "Script"),
        ("pub fn run_module(", "fn run_source(", "Module"),
    ] {
        let wrapper = bounded(ENGINE_SOURCE, start, end);
        assert_eq!(wrapper.matches("self.run_source(").count(), 1);
        assert!(wrapper.contains(&format!(
            "self.run_source(source, ParseGoal::{goal}, options, run)"
        )));
        assert!(!wrapper.contains("run.backend"));
        assert!(!wrapper.contains("self.run_with_spec_exec("));
        assert!(!wrapper.contains("self.run_source_with_cached_wasm("));
    }
    for (start, end, oracle_call, wasm_call) in [
        (
            "fn run_source(",
            "/// Executes one Script while keeping ECMAScript abrupt completion distinct",
            "self.run_with_spec_exec(",
            "self.run_source_with_cached_wasm(",
        ),
        (
            "fn observe_source(",
            "/// Runs a script through the Wasm-AOT backend on the calling thread.",
            "self.observe_with_spec_exec(",
            "self.observe_source_with_cached_wasm(",
        ),
        (
            "pub fn run_compiled_unit(",
            "/// Developer-only differential oracle path (Boa interpreter).",
            "self.run_with_spec_exec(",
            "self.run_with_wasm_aot(",
        ),
    ] {
        let dispatcher = bounded(ENGINE_SOURCE, start, end);
        if start == "fn run_source(" {
            let dispatch_at = dispatcher.find("match run.backend {").unwrap();
            for validation in [
                "validate_execution_intl_profile(&options.intl_profile, run.backend)?;",
                "validate_execution_language_features(options.language_features, run.backend)?;",
            ] {
                assert_eq!(dispatcher.matches(validation).count(), 1);
                assert!(dispatcher.find(validation).unwrap() < dispatch_at);
            }
            assert_eq!(dispatcher.matches("                goal,\n").count(), 2);
            assert!(!dispatcher.contains("ParseGoal::"));
        }
        assert_eq!(
            dispatcher.matches("match run.backend {").count(),
            1,
            "dispatcher `{start}` must own one exhaustive backend match"
        );
        assert_eq!(dispatcher.matches("=>").count(), 2, "dispatcher `{start}`");
        assert!(
            !dispatcher.contains("if run.backend"),
            "dispatcher `{start}`"
        );
        assert!(!dispatcher.contains("_ =>"), "dispatcher `{start}`");
        assert!(!dispatcher.contains("unreachable!"), "dispatcher `{start}`");

        let oracle_arm = bounded(
            dispatcher,
            "ExecutionBackend::SpecExec =>",
            "ExecutionBackend::WasmAot =>",
        );
        assert_eq!(
            oracle_arm.matches(oracle_call).count(),
            1,
            "dispatcher `{start}`"
        );
        assert!(!oracle_arm.contains(wasm_call), "dispatcher `{start}`");

        let wasm_arm = dispatcher
            .split_once("ExecutionBackend::WasmAot =>")
            .unwrap_or_else(|| panic!("dispatcher `{start}` is missing the WasmAot arm"))
            .1;
        assert_eq!(
            wasm_arm.matches(wasm_call).count(),
            1,
            "dispatcher `{start}`"
        );
        assert!(!wasm_arm.contains(oracle_call), "dispatcher `{start}`");
    }
}
