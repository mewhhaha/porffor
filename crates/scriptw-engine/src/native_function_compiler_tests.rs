//! Production Engine controls; observation is scoped to this Engine instance.

use super::*;
use wasm_native_function_compiler::{
    NativeFunctionCompilationObserver, NativeFunctionCompilationSnapshot,
};

mod program_ownership;
mod realm_and_order;

fn converted_text(text: &str) -> String {
    let units = text
        .encode_utf16()
        .map(|unit| unit.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("String.fromCharCode({units})")
}

fn assert_native_trace(
    source: &str,
    expected: &[&str],
    unmatched: &[&[&str]],
    for_of: bool,
    counts: NativeFunctionCompilationSnapshot,
) {
    configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observer = Arc::new(NativeFunctionCompilationObserver::default());
    let mut engine = Engine::new(RealmBuilder::new().build());
    engine.native_function_compilation_observer = Some(Arc::clone(&observer));
    let unit = engine
        .compile_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
        )
        .expect("outer source keeps normal compiler admission");
    let script = unit.ir.script.as_ref().expect("Script IR");
    assert!(script.prepared_dynamic_functions.len() <= 256);
    for tuple in unmatched {
        assert!(
            !script.prepared_dynamic_functions.iter().any(|entry| entry
                .arguments
                .iter()
                .map(String::as_str)
                .eq(tuple.iter().copied())),
            "actual tuple must not be admitted by the finite prepared guard: {tuple:?}"
        );
    }
    if for_of {
        assert!(
            script.body.statements.iter().any(|statement| matches!(
                statement,
                scriptw_ir::StatementIr::ForOfIterator { .. }
            )),
            "source discovery must retain the real generic iterator"
        );
    }
    let outcome = engine
        .observe_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("production native Function path must compile and execute");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        outcome.completion
    );
    let expected = expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_owned()))
        .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected, "source:\n{source}");
    assert_eq!(observer.snapshot(), counts, "actual native callback phases");
}

#[test]
fn historical_apply_to_string_tuples_use_native_factories_and_original_arguments() {
    // Original S15.3.4.3_A7_T3/T4 converted tuples. Runtime code-unit production
    // prevents a literal body from proving finite admission before ToString.
    let first_body = converted_text("this.shifted=a1;");
    let third_body = converted_text("this.shifted=a3;");
    let source = format!(
        r#"
var i = 0, trace = '';
var p = {{toString: function() {{ var name = 'a' + (++i); trace += name + ';'; return name; }}}};
var obj = {{}};
Function(p, "a2,a3", {first_body}).apply(obj, new Array("nine", "inch", "nails"));
print(obj.shifted);
print(i);
print(trace);
print(typeof this.shifted);
i = 0; trace = '';
new Function(p, p, p, {third_body}).apply(obj, (function() {{ return arguments; }})("a", "b", "c"));
print(obj.shifted);
print(i);
print(trace);
print(typeof this.shifted);
"#
    );
    for directive in ["", "'use strict';"] {
        assert_native_trace(
            &format!("{directive}{source}"),
            &[
                "nine",
                "1",
                "a1;",
                "undefined",
                "c",
                "3",
                "a1;a2;a3;",
                "undefined",
            ],
            &[
                &["a1", "a2,a3", "this.shifted=a1;"],
                &["a1", "a2", "a3", "this.shifted=a3;"],
            ],
            false,
            NativeFunctionCompilationSnapshot {
                requests: 2,
                completed_factories: 2,
                syntax_rejections: 0,
            },
        );
    }
}

#[test]
fn unmatched_body_reference_formals_use_actual_source_and_global_environment() {
    let source = r#"
var key1 = 17;
var counter = 3;
var p = { toString() { var name = 'key' + (++counter); print(name); return name; } };
var created = Function(p, 'return key1;');
print(created(42));
print(counter);
print(created.length);
print(created.toString().includes('key4'));
"#;
    for directive in ["", "'use strict';"] {
        assert_native_trace(
            &format!("{directive}{source}"),
            &["key4", "17", "4", "1", "true"],
            &[&["key4", "return key1;"]],
            false,
            NativeFunctionCompilationSnapshot {
                requests: 1,
                completed_factories: 1,
                syntax_rejections: 0,
            },
        );
    }
}

#[test]
fn finite_overflow_keeps_iteration_effects_and_both_endpoint_native_factories() {
    let literals = (0..257)
        .map(|index| format!("'return {index};'"))
        .collect::<Vec<_>>()
        .join(",");
    for directive in ["", "'use strict';"] {
        for head in ["var body", "let body", "const body"] {
            let source = format!(
                r#"
{directive}
var visits = 0, conversions = 0, trace = '';
for ({head} of [{literals}]) {{
  var index = visits++;
  if (index === 0 || index === 256) {{
    print(Function({{toString() {{ conversions++; trace += index + ';'; return body; }}}})());
  }}
}}
print(visits);
print(conversions);
print(trace);
"#
            );
            assert_native_trace(
                &source,
                &["0", "256", "257", "2", "0;256;"],
                &[&["return 0;"], &["return 256;"]],
                true,
                NativeFunctionCompilationSnapshot {
                    requests: 2,
                    completed_factories: 2,
                    syntax_rejections: 0,
                },
            );
        }
    }
}
