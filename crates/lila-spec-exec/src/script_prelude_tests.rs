use super::*;
use lila_runtime::rooted_snapshot::{
    SnapshotCompletionKind, SnapshotLimits, SnapshotOutcome, SnapshotValue,
};
use lila_runtime::{EmbeddedModuleEntryInput, EmbeddedModuleGraph};

#[derive(Debug, Default)]
struct RecordingHooks(Mutex<Vec<String>>);

impl HostHooks for RecordingHooks {
    fn print_line(&self, text: &str) {
        self.0.lock().unwrap().push(text.to_owned());
    }
}

fn observe(
    source: &str,
    prelude: Option<&str>,
) -> Result<ObservedExecutionOutcome, ExecutionError> {
    observe_script_with_prelude(
        source,
        Some("script-prelude.js"),
        ModuleLoadingPolicy::RejectAll,
        prelude,
        &[],
        false,
        Arc::new(RecordingHooks::default()),
    )
}

fn execute(source: &str, prelude: Option<&str>) -> Result<ExecutionOutcome, ExecutionError> {
    execute_script_with_prelude(
        source,
        Some("script-prelude.js"),
        ModuleLoadingPolicy::RejectAll,
        prelude,
        &[],
        false,
    )
}

fn number(value: f64) -> ObservedJsValue {
    ObservedJsValue::Number(ObservedNumber::from_f64(value))
}

fn lines(values: &[&str]) -> Vec<HostOutputEvent> {
    values
        .iter()
        .map(|value| HostOutputEvent::PrintLine((*value).into()))
        .collect()
}

#[test]
fn script_prelude_keeps_global_bindings_and_each_directive_prologue() {
    let prelude = r#"
        var harnessVar = 22;
        let harnessLexical = 20;
        function harnessSloppy() { return this === globalThis; }
    "#;
    let source = r#"
        "use strict";
        if ((function () { return this; })() !== undefined) throw 'entry lost strictness';
        if (!harnessSloppy()) throw 'prelude gained entry strictness';
        if (globalThis.harnessVar !== harnessVar) throw 'prelude var is not global';
        if (Object.prototype.hasOwnProperty.call(globalThis, 'harnessLexical')) throw 'lexical became a property';
        harnessLexical + harnessVar;
    "#;
    execute(source, Some(prelude)).expect("independent Script prelude executes before entry");
    let outcome = observe(source, Some(prelude)).unwrap();
    assert_eq!(outcome.completion, ObservedCompletion::Normal(number(42.0)));
}

#[test]
fn existing_script_wrappers_match_explicit_absent_prelude() {
    let source = "21 * 2";
    let legacy = execute_script_with_module_loading_policy(
        source,
        Some("script-prelude.js"),
        ModuleLoadingPolicy::RejectAll,
        &[],
        false,
    )
    .unwrap();
    assert_eq!(legacy, execute(source, None).unwrap());
    let legacy = observe_script_with_module_loading_policy(
        source,
        Some("script-prelude.js"),
        ModuleLoadingPolicy::RejectAll,
        &[],
        false,
        Arc::new(RecordingHooks::default()),
    )
    .unwrap();
    assert_eq!(legacy, observe(source, None).unwrap());
}

#[test]
fn entry_parse_rejection_precedes_prelude_and_prelude_errors_are_not_entry_syntax() {
    for error in [
        execute("const =", Some("throw 17;")).unwrap_err(),
        observe("const =", Some("throw 17;")).unwrap_err(),
    ] {
        assert!(error.is_entry_syntax_rejection());
        assert!(!error.is_global_prelude_failure());
    }
    for error in [
        execute("42", Some("const =")).unwrap_err(),
        observe("42", Some("const =")).unwrap_err(),
    ] {
        assert!(!error.is_entry_syntax_rejection());
        assert!(error.is_global_prelude_failure());
        assert_eq!(
            error.javascript_exception_phase(),
            Some(lila_runtime::OracleExceptionPhase::Runtime)
        );
        assert_eq!(
            error.javascript_exception_type(),
            Some(lila_runtime::OracleExceptionType::Syntax)
        );
    }
    let error = execute("throw new TypeError('entry');", None).unwrap_err();
    assert!(!error.is_global_prelude_failure());
    assert_eq!(
        error.javascript_exception_phase(),
        Some(lila_runtime::OracleExceptionPhase::Runtime)
    );
    assert_eq!(
        error.javascript_exception_type(),
        Some(lila_runtime::OracleExceptionType::Type)
    );
    for error in [
        execute("42", Some("throw new TypeError('prelude');")).unwrap_err(),
        observe("42", Some("throw new TypeError('prelude');")).unwrap_err(),
    ] {
        assert!(!error.is_entry_syntax_rejection());
        assert!(error.is_global_prelude_failure());
        assert_eq!(
            error.javascript_exception_phase(),
            Some(lila_runtime::OracleExceptionPhase::Runtime)
        );
        assert_eq!(
            error.javascript_exception_type(),
            Some(lila_runtime::OracleExceptionType::Type)
        );
    }
}

#[test]
fn observed_prelude_throw_stays_primary_drains_jobs_and_prevents_entry() {
    let hooks = Arc::new(RecordingHooks::default());
    let error = observe_script_with_prelude(
        "print('entry-must-not-run'); 42;",
        Some("script-prelude.js"),
        ModuleLoadingPolicy::RejectAll,
        Some(
            r#"
            Promise.resolve().then(() => { print('prelude-job'); throw new RangeError('secondary'); });
            throw -0;
        "#,
        ),
        &[],
        false,
        hooks.clone(),
    )
    .unwrap_err();
    assert!(error.is_global_prelude_failure());
    assert!(!error.is_entry_syntax_rejection());
    assert_eq!(
        error.javascript_exception_phase(),
        Some(lila_runtime::OracleExceptionPhase::Runtime)
    );
    assert_eq!(
        error.javascript_exception_type(),
        Some(lila_runtime::OracleExceptionType::Primitive)
    );
    assert_eq!(*hooks.0.lock().unwrap(), ["prelude-job".to_owned()]);
}

#[test]
fn prelude_and_entry_share_realms_hooks_and_a_single_final_job_checkpoint() {
    let hooks = Arc::new(RecordingHooks::default());
    let outcome = observe_script_with_prelude(
        r#"
        harnessRealm.evalScript("print('realm')");
        print('entry');
        harnessValue = 42;
        harnessValue;
    "#,
        Some("script-prelude.js"),
        ModuleLoadingPolicy::RejectAll,
        Some(
            r#"
            var harnessValue = 0;
            var harnessRealm = $262.createRealm();
            print('prelude');
            Promise.resolve().then(() => print('job:' + harnessValue));
        "#,
        ),
        &[],
        false,
        hooks.clone(),
    )
    .unwrap();
    assert_eq!(outcome.completion, ObservedCompletion::Normal(number(42.0)));
    assert_eq!(
        outcome.output_events,
        lines(&["prelude", "realm", "entry", "job:42"])
    );
    assert_eq!(
        *hooks.0.lock().unwrap(),
        ["prelude", "realm", "entry", "job:42"].map(str::to_owned)
    );
}

#[test]
fn prelude_imports_retain_the_explicit_reject_all_loader() {
    let outcome = observe(
        "print('entry'); void 0;",
        Some(
            r#"
            import('undeclared-prelude-module').then(
                () => print('unexpected module load'),
                error => print(error.message)
            );
        "#,
        ),
    )
    .unwrap();
    assert_eq!(
        outcome.completion,
        ObservedCompletion::Normal(ObservedJsValue::Undefined)
    );
    assert_eq!(
        outcome.output_events,
        lines(&["entry", "module loading disabled by host policy"])
    );
}

#[test]
fn embedded_entry_admission_uses_original_script_bytes_with_independent_prelude() {
    let source = "harnessAnswer;";
    let graph = Arc::new(
        EmbeddedModuleGraph::try_new(
            EmbeddedModuleEntryInput {
                goal: EmbeddedModuleGoal::Script,
                identity: "case/entry.js".into(),
                source: source.into(),
                meta_url: "lila://script-prelude/entry.js".into(),
            },
            vec![],
            vec![],
        )
        .unwrap(),
    );
    let outcome = observe_script_with_prelude(
        source,
        Some("case/entry.js"),
        ModuleLoadingPolicy::Embedded(graph),
        Some("var harnessAnswer = 42;"),
        &[],
        false,
        Arc::new(RecordingHooks::default()),
    )
    .unwrap();
    assert_eq!(outcome.completion, ObservedCompletion::Normal(number(42.0)));
}

#[test]
fn async_done_state_from_prelude_is_checked_after_entry_execution() {
    let prelude = "globalThis.__lilaTest262AsyncDone = { active: true, called: false };";
    let source = "__lilaTest262AsyncDone.called = true; void 0;";
    execute(source, Some(prelude)).unwrap();
    let outcome = observe(source, Some(prelude)).unwrap();
    assert_eq!(
        outcome.completion,
        ObservedCompletion::Normal(ObservedJsValue::Undefined)
    );
    for error in [
        execute("void 0;", Some(prelude)).unwrap_err(),
        observe("void 0;", Some(prelude)).unwrap_err(),
    ] {
        assert!(error.message().contains("async $DONE was not called"));
    }
}

#[test]
fn module_prelude_failures_share_checked_origin_without_changing_entry_failures() {
    for prelude in ["const =", "throw new TypeError('prelude');"] {
        let host = ModuleHostConfig {
            prelude: Some(prelude.into()),
            ..ModuleHostConfig::default()
        };
        for error in [
            execute_module("export const answer = 42;", None, host.clone(), &[], false)
                .unwrap_err(),
            observe_module(
                "export const answer = 42;",
                None,
                host,
                &[],
                false,
                Arc::new(RecordingHooks::default()),
            )
            .unwrap_err(),
        ] {
            assert!(error.is_global_prelude_failure());
            assert!(!error.is_entry_syntax_rejection());
            assert_eq!(
                error.javascript_exception_phase(),
                Some(lila_runtime::OracleExceptionPhase::Runtime)
            );
            assert_eq!(
                error.javascript_exception_type(),
                Some(if prelude == "const =" {
                    lila_runtime::OracleExceptionType::Syntax
                } else {
                    lila_runtime::OracleExceptionType::Type
                })
            );
        }
    }
    let error = execute_module(
        "export const =",
        None,
        ModuleHostConfig {
            prelude: Some("throw new TypeError('prelude');".into()),
            ..ModuleHostConfig::default()
        },
        &[],
        false,
    )
    .unwrap_err();
    assert!(error.is_entry_syntax_rejection());
    assert!(!error.is_global_prelude_failure());
}

#[test]
fn graph_prelude_shares_global_bindings_and_rejects_harness_errors_before_snapshot() {
    let outcome = observe_script_graph_with_prelude(
        "harnessAnswer;",
        Some("script-prelude.js"),
        ModuleLoadingPolicy::RejectAll,
        Some("var harnessAnswer = 42;"),
        &[],
        false,
        Arc::new(RecordingHooks::default()),
        SnapshotLimits::default(),
    )
    .unwrap();
    assert_eq!(outcome.completion.kind, SnapshotCompletionKind::Normal);
    let SnapshotOutcome::Captured { graph } = outcome.completion.outcome else {
        panic!("entry number snapshot must fit default limits");
    };
    assert_eq!(
        graph.root(),
        &SnapshotValue::Number {
            bits: 42.0f64.to_bits()
        }
    );
    for (prelude, kind) in [
        ("const =", lila_runtime::OracleExceptionType::Syntax),
        (
            "throw new TypeError('prelude');",
            lila_runtime::OracleExceptionType::Type,
        ),
    ] {
        let error = observe_script_graph_with_prelude(
            "42;",
            Some("script-prelude.js"),
            ModuleLoadingPolicy::RejectAll,
            Some(prelude),
            &[],
            false,
            Arc::new(RecordingHooks::default()),
            SnapshotLimits::default(),
        )
        .unwrap_err();
        assert!(error.is_global_prelude_failure());
        assert!(!error.is_entry_syntax_rejection());
        assert_eq!(error.javascript_exception_type(), Some(kind));
    }
}
