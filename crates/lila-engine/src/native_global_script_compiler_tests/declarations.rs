use super::*;
#[test]
fn native_indirect_eval_keeps_strictness_global_vars_lexicals_and_runtime_conflicts() {
    let sloppy = "var nativeValue=17; let nativeLexical=23; nativeValue+nativeLexical;";
    let strict = "'use strict'; var strictOnly=42; let strictLexical=1; strictOnly;";
    let conflict = "var collision=99;";
    for directive in ["", "'use strict';"] {
        let source = format!(
            r#"
{directive}
var indirect=eval;
let collision=1;
print(indirect({sloppy}));
print(nativeValue);
print(typeof nativeLexical);
print(indirect({strict}));
print(typeof strictOnly);
try {{ indirect({conflict}); }} catch(error) {{ print(error instanceof SyntaxError); }}
print(collision);
"#,
            sloppy = converted_text(sloppy),
            strict = converted_text(strict),
            conflict = converted_text(conflict)
        );
        assert_native_trace(
            &source,
            &["40", "17", "undefined", "42", "undefined", "true", "1"],
            &[
                (
                    PreparedScriptKind::IndirectEval,
                    sloppy.encode_utf16().collect(),
                ),
                (
                    PreparedScriptKind::IndirectEval,
                    strict.encode_utf16().collect(),
                ),
                (
                    PreparedScriptKind::IndirectEval,
                    conflict.encode_utf16().collect(),
                ),
            ],
            false,
            global_counts(3, 3, 0),
            0,
        );
    }
}

#[test]
fn native_global_support_keeps_non_strings_prepared_direct_and_shadow_entries() {
    let source = r#"
var token={toString(){throw 'must not convert';}};
print((0,eval)(token)===token);
print(eval('17;'));
var realm=new ShadowRealm();
print(realm.evaluate('23;'));
"#;
    assert_native_trace(
        source,
        &["true", "17", "23"],
        &[],
        false,
        global_counts(0, 0, 0),
        0,
    );
}

#[test]
fn unmatched_direct_and_shadow_sources_keep_their_separate_typed_capability_errors() {
    configure_compilation_jobs(1).unwrap();
    // Retain this historical qualified selector while direct eval gains its
    // own caller-context native route. The global compiler still sees no call.
    let direct = format!("print(eval({}));", converted_text("1+2;"));
    assert_native_trace(&direct, &["3"], &[], false, global_counts(0, 0, 0), 0);
    for (callee, expected) in [(
        "new ShadowRealm().evaluate",
        lila_ir::DynamicSourceRuntimeOperation::ShadowRealmEvaluate,
    )] {
        let source = format!("{callee}({});", converted_text("1+2;"));
        let observer = Arc::new(NativeGlobalScriptCompilationObserver::default());
        let mut engine = Engine::new(RealmBuilder::new().build());
        engine.native_global_script_compilation_observer = Some(Arc::clone(&observer));
        let error = engine
            .observe_script(
                &source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_err();
        assert_eq!(error.runtime_dynamic_source_operations(), vec![expected]);
        assert_eq!(observer.snapshot(), global_counts(0, 0, 0));
    }
}
