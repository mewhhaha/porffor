use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, RealmBuilder,
    RunOptions, WasmExecutionFailureKind,
};
use lila_ir::DynamicSourceRuntimeOperation;

fn assert_modes(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(60_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| {
                panic!("ShadowRealm source controls failed: {error}\n{source}")
            });
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{source}"
        );
        assert!(observed.output_events.is_empty(), "{source}");
    }
}

#[test]
fn finite_evaluation_has_fresh_lexicals_and_strictness_owned_variable_lifetimes() {
    assert_modes(
        r#"
        const realm = new ShadowRealm();
        if (realm.evaluate('let temporary = 1; temporary') !== 1) throw 'first-lexical';
        if (realm.evaluate('let temporary = 2; temporary') !== 2) throw 'repeated-lexical';
        if (realm.evaluate('typeof temporary') !== 'undefined') throw 'lexical-leak';
        if (realm.evaluate('var shared = 3; shared') !== 3) throw 'sloppy-var';
        if (realm.evaluate('shared += 1; shared') !== 4) throw 'shared-var';
        if (realm.evaluate("'use strict'; var hidden = 5; hidden") !== 5) throw 'strict-var';
        if (realm.evaluate('typeof hidden') !== 'undefined') throw 'strict-var-leak';
        if (realm.evaluate('delete shared') !== true) throw 'eval-var-configurability';
        if (realm.evaluate('typeof shared') !== 'undefined') throw 'delete-shared';
        if (realm.evaluate('unboundWrite = 7; unboundWrite') !== 7) throw 'inherited-strictness';
        if (realm.evaluate('this === globalThis') !== true) throw 'global-this';
        const retained = realm.evaluate('let retained = 13; () => retained');
        realm.evaluate('let retained = 19; retained');
        if (retained() !== 13) throw 'retained-eval-environment';
        if (typeof unboundWrite !== 'undefined') throw 'outer-global-leak';
        true;
        "#,
    );
}

#[test]
fn finite_candidates_keep_actual_method_receiver_aliases_and_nested_realm_dispatch() {
    assert_modes(include_str!(
        "fixtures/shadow_realm/finite_candidates_receiver_aliases.js"
    ));
}

#[test]
fn source_argument_validation_and_parse_errors_precede_shadow_execution() {
    assert_modes(
        r#"
        const realm = new ShadowRealm();
        let conversions = 0;
        const source = { toString() { conversions++; throw 'coercion'; } };
        try { realm.evaluate(source); throw 'object-source-accepted'; }
        catch (error) { if (!(error instanceof TypeError)) throw error; }
        try { realm.evaluate(new String('1')); throw 'boxed-source-accepted'; }
        catch (error) { if (!(error instanceof TypeError)) throw error; }
        try { realm.evaluate(Symbol('source')); throw 'symbol-source-accepted'; }
        catch (error) { if (!(error instanceof TypeError)) throw error; }
        const evaluate = realm.evaluate;
        try { evaluate?.call({}, '123'); throw 'optional-call-receiver-accepted'; }
        catch (error) { if (!(error instanceof TypeError)) throw error; }
        try { evaluate.apply?.({}, ['124']); throw 'optional-apply-receiver-accepted'; }
        catch (error) { if (!(error instanceof TypeError)) throw error; }
        if (conversions !== 0) throw 'source-coercion';
        try { realm.evaluate('let = ;'); throw 'syntax-accepted'; }
        catch (error) { if (!(error instanceof SyntaxError)) throw error; }
        try { realm.evaluate('new.target'); throw 'new-target-accepted'; }
        catch (error) { if (!(error instanceof SyntaxError)) throw error; }
        try { realm.evaluate('throw new SyntaxError("target")'); throw 'target-throw-accepted'; }
        catch (error) { if (!(error instanceof TypeError)) throw error; }
        if (realm.evaluate('2 + 3') !== 5) throw 'realm-after-throw';
        true;
        "#,
    );
}

#[test]
fn unmatched_shadow_source_retains_uncatchable_typed_runtime_rejection() {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let error = Engine::new(RealmBuilder::new().build())
        .run_script(
            r#"
            const realm = new ShadowRealm();
            let source = '23;';
            realm.evaluate(source);
            source += '/*' + Math.random() + '*/';
            try { realm.evaluate?.call(realm, source); }
            catch (error) { throw 'must not catch capability rejection'; }
            throw 'must not continue';
            "#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .expect_err("an unavailable source cannot execute a different prepared candidate");
    assert_eq!(
        error.runtime_dynamic_source_operations(),
        vec![DynamicSourceRuntimeOperation::ShadowRealmEvaluate],
        "{error}"
    );
    assert_eq!(
        error.wasm_execution_failure_kind(),
        Some(WasmExecutionFailureKind::DynamicSource)
    );
    assert!(error.parse_diagnostic().is_none(), "{error}");
    assert!(error.ir_diagnostic().is_none(), "{error}");
}
