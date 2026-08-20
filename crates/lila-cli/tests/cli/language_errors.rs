//! `language` CLI integration tests: errors, abrupt completions, realms and
//! `new.target`, host hooks, async generators.
//!
//! Split out of [`crate::language`] because that module OOMed as one libtest
//! process on this container — three SIGKILLs at t+1200 s with `avail` falling
//! monotonically to 1.14 GiB — and the only remaining lever is fewer tests per
//! process. See the header of `language.rs` for the measurements, the two
//! refuted runner knobs, and why the split has to be by module file rather than
//! by libtest filter.
//!
//! 33 tests, all heavy. Its chunk is `run_chunk language_errors
//! language_errors::` in `scripts/rung1c-chunks.sh`, and it needs BOTH that line
//! and `mod language_errors;` in `main.rs`: a module with a chunk but no `mod`
//! line is not compiled, its filter selects nothing, libtest exits 0 on
//! `0 passed`, and the done-file banks a chunk that measured nothing. That is
//! the `iterator_helpers` incident, and
//! `known_failures::rung_1c_chunks_cover_every_cli_area_module` is what catches
//! it now.
//!
//! Do NOT rename this module to anything ending in `language` — the overlap rule
//! keys on `"{other}::".ends_with("{chunk}::")`, and a stem that is a `::`-suffix
//! of another needs a `--skip` or the two chunks double-run each other's tests.

use crate::*;

#[test]
fn run_wasm_backend_succeeds_for_abstract_module_source_host_hook_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_abstract_module_source_host_hook.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_htmldda_host_hook_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_htmldda_host_hook.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262"));
}

#[test]
fn run_wasm_backend_succeeds_for_aggregateerror_cross_realm_newtarget_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_aggregateerror_cross_realm_newtarget.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(123"));
}

#[test]
fn run_wasm_backend_uses_async_function_intrinsic_prototype_topology() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_async_function_intrinsics.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(123"), "{stdout}");
}

#[test]
fn run_wasm_backend_succeeds_for_boolean_cross_realm_newtarget_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_boolean_cross_realm_newtarget.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(123"));
}

#[test]
fn run_wasm_backend_routes_abrupt_completions_through_finally() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_abrupt_finally_routing.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_preserves_catch_finally_derived_return_completions() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_catch_finally_derived_return.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_non_callable_catchability_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_non_callable_catchability.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("string(call,construct,error-call,error-construct)"));
}

#[test]
fn run_wasm_backend_succeeds_for_error_tostring_toprimitive_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_error_tostring_toprimitive.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("string(message,name)"));
}

#[test]
fn run_wasm_backend_orders_error_tostring_and_reads_every_object_representation() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_error_tostring_order_and_receivers.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

#[test]
fn run_wasm_backend_succeeds_for_error_iserror_other_realm_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_error_iserror_other_realm.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains(
        "string(Error,EvalError,RangeError,ReferenceError,SyntaxError,TypeError,URIError,AggregateError)"
    ));
}

#[test]
fn run_wasm_backend_succeeds_for_error_constructor_properties_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_error_constructor_properties.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_uses_new_target_realm_for_error_prototype() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_error_constructor_realm_prototype.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262)"), "{stdout}");
}

#[test]
fn run_wasm_backend_uses_new_target_realms_for_native_error_prototypes() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_native_error_constructor_realm_prototypes.js",
        ))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(262)"), "{stdout}");
}

#[test]
fn run_wasm_backend_succeeds_for_throwtypeerror_intrinsic_properties_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_throwtypeerror_intrinsic_properties.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_reports_uncaught_throw_fixture_error() {
    let output = ProcessCommand::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_uncaught_throw.js"))
        .output()
        .expect("run command should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("uncaught throw: wasm-aot completion: number(1)"));
}

#[test]
fn run_wasm_backend_reports_gc_requires_real_collector() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_gc_requires_real_collector.js"))
        .output()
        .expect("run command should run");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("uncaught throw"));
    assert!(stderr.contains("gc requires a real collector in wasm-aot"));
}

#[test]
fn run_wasm_backend_succeeds_for_supported_abrupt_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_abrupt_core.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_aggregateerror_newtarget_prototype_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_aggregateerror_newtarget_prototype.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(123"));
}

#[test]
fn run_wasm_backend_succeeds_for_aggregateerror_iterable_to_list_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_aggregateerror_iterable_to_list.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(123"));
}

#[test]
fn run_wasm_backend_succeeds_for_aggregateerror_constructor_properties_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_aggregateerror_constructor_properties.js",
        ))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_suppressederror_constructor_properties_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_suppressederror_constructor_properties.js",
        ))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_bound_construct_new_target_identity_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_bound_construct_new_target_identity.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_succeeds_for_construct_function_realm_fallback_fixture() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_construct_function_realm_fallback.js"))
        .output()
        .expect("run command should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"));
}

#[test]
fn run_wasm_backend_preserves_async_generator_yield_star_acquisition_across_method_wrappers() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_async_generator_yield_star_wrapper_acquisition.js",
        ))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("number(5"), "{stdout}");
}

#[test]
fn run_wasm_backend_preserves_async_generator_yield_star_lexical_initialization() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_async_generator_yield_star_lexical_initialization.js",
        ))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(
        stdout.contains("async-generator-yield-star-lexical-initialization:11:13"),
        "{stdout}"
    );
}

#[test]
fn run_wasm_backend_validates_async_generator_yield_star_next_across_method_wrappers() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_async_generator_yield_star_next_wrapper_validation.js",
        ))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(
        stdout.contains("async-generator-next-wrapper-validation:12:false"),
        "{stdout}"
    );
}

#[test]
fn run_wasm_backend_validates_async_generator_yield_star_return_across_method_wrappers() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_async_generator_yield_star_return_wrapper_validation.js",
        ))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(
        stdout.contains("async-generator-return-wrapper-validation:4"),
        "{stdout}"
    );
}

#[test]
fn run_wasm_backend_validates_async_generator_yield_star_throw_across_method_wrappers() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_async_generator_yield_star_throw_wrapper_validation.js",
        ))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(
        stdout.contains("async-generator-throw-wrapper-validation:4"),
        "{stdout}"
    );
}

/// PutValue 6.2.5.6 step 3.d, consumed by a Reference whose `[[Strict]]` is
/// carried on the IR node rather than read from the ambient strictness of the
/// function being emitted.
///
/// The pair is the whole test: the two fixtures differ only in the directive
/// prologue, so a strict-only pass would mean the throw was hardcoded and a
/// sloppy-only pass would mean the carried flag never reaches the guard.
///
/// The `try` is not decoration. Inside `main`, the strict guard's throw
/// branches to the active handler by Wasm label depth, and the runtime form of
/// the guard opens one block the compile-time form does not; forwarding the
/// caller's depth unchanged into it either targets the wrong label or fails
/// validation. Nothing outside a top-level `try` observes that — an emitted
/// function returns a completion instead of branching.
#[test]
fn run_wasm_backend_throws_for_strict_reference_property_write_inside_top_level_try() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_reference_strictness_putvalue_strict.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

/// The sloppy half of the pair above. See its doc comment.
#[test]
fn run_wasm_backend_ignores_sloppy_reference_property_write_inside_top_level_try() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_reference_strictness_putvalue_sloppy.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

/// A generator evaluates a property Reference before suspending and consumes
/// that same Reference only after a normal resume. This pins base/key order,
/// the no-re-evaluation rule, carried `[[Strict]]`, abrupt suppression and the
/// shared plain/delegated resume consumer.
#[test]
fn run_wasm_backend_preserves_property_reference_across_generator_suspension() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_generator_suspended_property_reference.js",
        ))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"));
    assert!(stdout.contains("boolean(true)"), "{stdout}");
}

/// A runtime-thrown error's `message` must not be its `name`.
///
/// **Was the T24 declared failure; repaired, so the declaration is gone.**
/// `emit_runtime_error_object` (`crates/lila-aot-wasm/src/builtins/errors.rs`)
/// used to define `message` from the error's *name* payload and ignore the
/// message argument entirely — its parameter was spelled `_message`, so not even
/// an unused-parameter warning mentioned it. The one-token repair could not land
/// alone: `StringPool::payload` panics for a string that was never interned, and
/// the message literals reaching that function were not in the pool precisely
/// because it never asked for them. It landed together with
/// `data.rs`'s `RUNTIME_ERROR_MESSAGE_LITERALS`.
///
/// The ledger row, this test's `#[should_panic]` and the `const _` line in
/// `known_failures.rs` were retired in that same patch, because a declared
/// failure that starts passing is a rung-1c red by design
/// (`test did not panic as expected`).
///
/// The test stays. Nothing else observes this: no Test262 case reads a
/// runtime-thrown error's message (audited — the nine files that assert on a
/// caught error's `.message` all throw it themselves), and no other CLI fixture
/// does either, so deleting the observer would let the defect come back with
/// every suite green. That is exactly the shape it took the first time.
#[test]
fn run_wasm_backend_gives_a_runtime_error_a_message_distinct_from_its_name() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(
            "wasm_runtime_error_message_is_not_its_name.js",
        ))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("backend_used: WasmAot"), "{stdout}");
    assert!(
        stdout.contains("string(message-differs)"),
        "runtime error message equals its name (emit_runtime_error_object must define \
         `message` from its message argument, not from the name payload): {stdout}"
    );
}
