use std::path::PathBuf;

use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};
use lila_ir::HostSurfacePolicy;
use lila_test262::{load_preludes, LocalHarnessSource, PreludeOrigin, SuiteConfig};

fn assert_callable_harness(source: &str, include: &str, strict: bool) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let config = SuiteConfig {
        suite_root: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test262/vendor/test262"),
        local_harness: LocalHarnessSource::EmbeddedWasmAot,
        ..SuiteConfig::default()
    };
    let preludes = load_preludes(&config).expect("pinned callable harness");
    let canonical = preludes.get(include).expect("canonical helper");
    assert_eq!(canonical.origin, PreludeOrigin::VendoredHarness);
    let mut program = if strict {
        "'use strict';\n".to_string()
    } else {
        String::new()
    };
    for name in ["assert.js", "sta-preamble.js", include] {
        program.push_str(&preludes.get(name).expect("complete named section").contents);
        program.push('\n');
    }
    program.push_str(source);
    program.push_str("\ntrue;");
    let result = Engine::new(RealmBuilder::new().build())
        .run_script(
            &program,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("canonical callable harness failed: {error}\n{source}"));
    assert!(result.note.contains("boolean(true)"), "{}", result.note);
}

const CONSTRUCTOR_CASE: &str =
    include_str!("../../../test262/vendor/test262/test/harness/isConstructor.js");
const NATIVE_MATCHER_CASE: &str =
    include_str!("../../../test262/vendor/test262/test/harness/nativeFunctionMatcher.js");

#[test]
fn pinned_constructor_helper_preserves_sloppy_semantics() {
    assert_callable_harness(CONSTRUCTOR_CASE, "isConstructor.js", false);
}

#[test]
fn pinned_constructor_helper_preserves_strict_semantics() {
    assert_callable_harness(CONSTRUCTOR_CASE, "isConstructor.js", true);
}

#[test]
fn pinned_native_function_grammar_preserves_sloppy_semantics() {
    assert_callable_harness(NATIVE_MATCHER_CASE, "nativeFunctionMatcher.js", false);
}

#[test]
fn pinned_native_function_grammar_preserves_strict_semantics() {
    assert_callable_harness(NATIVE_MATCHER_CASE, "nativeFunctionMatcher.js", true);
}

#[test]
fn constructor_helper_observes_prototype_getters_and_revoked_proxies() {
    assert_callable_harness(
        r#"
var calls = 0;
var constructor = new Proxy(function() {}, { get(target, key) {
  if (key === 'prototype') { calls++; throw 1; }
  return target[key];
} });
assert.sameValue(isConstructor(constructor), false);
assert.sameValue(calls, 1);
var revoked = Proxy.revocable(function() {}, {});
revoked.revoke();
assert.sameValue(typeof revoked.proxy, 'function');
assert.sameValue(isConstructor(revoked.proxy), false);
assert.sameValue(isConstructor(new Proxy(function() {}, {})), true);
assert.sameValue(isConstructor(() => {}), false);
assert.throws(Test262Error, () => isConstructor({}));
"#,
        "isConstructor.js",
        false,
    );
}
