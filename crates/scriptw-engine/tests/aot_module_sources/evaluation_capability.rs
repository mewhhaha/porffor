use super::{observe, run};

#[test]
fn source_only_host_records_reject_actual_evaluation_without_fabricating_a_body() {
    for entry in [
        "import * as empty from 'host:first';",
        "import source first from 'host:first'; await import('host:first');",
        "await import('./unsupported.js');",
    ] {
        let error = observe(
            entry,
            &[("./unsupported.js", "import * as empty from 'host:first';")],
            &[(&["host:first"], "ModuleSource")],
        )
        .expect_err("a source representation never supplies an evaluation body");
        assert_eq!(
            error.runtime_unavailable_capabilities(),
            vec![scriptw_ir::RuntimeUnavailableCapability::HostModuleEvaluation],
            "{error}"
        );
        assert_eq!(
            error.wasm_execution_failure_kind(),
            Some(scriptw_engine::WasmExecutionFailureKind::UnavailableCapability)
        );
        assert_eq!(error.wasm_javascript_exception_constructor_name(), None);
        assert!(error.parse_diagnostic().is_none(), "{error}");
        assert!(error.ir_diagnostic().is_none(), "{error}");
    }
}

#[test]
fn unchosen_host_evaluation_requests_do_not_block_source_imports_or_run_bodies() {
    run(
        r#"
import source first from 'host:first';
if (false) {
  await import('host:first');
  await import('./unsupported.js');
}
if ((await import.source('host:first')) !== first) throw 'source-only identity';
print('native-module-source:ok');
"#,
        &[(
            "./unsupported.js",
            "import * as empty from 'host:first'; throw 'body ran';",
        )],
        &[(&["host:first"], "ModuleSource")],
    );
}
