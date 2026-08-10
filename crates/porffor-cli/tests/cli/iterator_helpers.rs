//! `Iterator.prototype` helper CLI integration tests, for the one receiver
//! shape that broke every helper but `forEach`.
//!
//! # Why this module exists next to `iterator.rs`
//!
//! `iterator.rs` already carries four helper fixtures — `some`, `every`,
//! `find`, `reduce`. All four failed, and the repair is a single change in
//! `emit_method_call`: nine of the ten `Iterator.prototype` fast paths acquired
//! their callee with `emit_function_value_payload` rather than with the
//! ordinary `[[Get]]` the generic tail and the (working) `forEach` block use.
//! What was **measured** for a `class X extends Iterator` receiver (no explicit
//! constructor) is that the callback never ran, both destination locals were
//! left holding stale scratch, and the arm returned `Ok(())` —
//! `new X().some(cb)` answering `typeof === "object"`. What was **not**
//! identified is the mechanism inside that acquisition, so treat the four
//! `iterator.rs` tests flipping green as necessary evidence, not as attribution:
//! those fixtures also assert IteratorClose obligations that a concurrent round
//! is changing in the same tree, and they discard the thrown label
//! (`assert!(output.status.success())` only), so which check failed is unknown
//! from their output alone.
//!
//! Four fixtures over four helpers is not enough to hold that repair down.
//! `map`, `filter`, `flatMap`, `take` and `drop` carried the identical broken
//! acquisition and had **no** fixture on this receiver shape — they could have
//! stayed broken with `iterator.rs` fully green. So this module covers all
//! ELEVEN helpers (`toArray` included, precisely because it is the fast path's
//! oracle and must never grow one that differs), each asserting both halves of
//! the failure:
//!
//! 1. the returned **value and type** — a fast path that emits nothing fails
//!    this, because the destination locals are never written; and
//! 2. that a **callback throw reaches a user `catch`** — a discarded throw is
//!    one visible face of a call that never happened.
//!
//! It is a new module rather than additions to `iterator.rs` because that file
//! is owned by the concurrent IteratorClose round: the four tests at
//! `iterator.rs:372/388/404/420` must flip green with **zero** edits, which is
//! a strictly stronger claim than editing them.
//!
//! Every fixture is a discriminator, not a boolean: it answers `string(ok)`
//! when every check holds and otherwise answers a `;`-separated list naming the
//! checks that failed, so a red test reports *what* diverged rather than only
//! that something did.

use crate::*;

/// Run one fixture and assert the three invariants every test in this module
/// shares: the run succeeded, it really used the Wasm AOT backend, and the
/// fixture's own discriminator answered `ok`.
fn assert_helper_fixture_is_ok(fixture: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_porf"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path(fixture))
        .output()
        .expect("run command should run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{fixture}: stdout={stdout} stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    // Without this the whole module could pass on a different backend, and the
    // defect under test is a Wasm emission defect.
    assert!(
        stdout.contains("backend_used: WasmAot"),
        "{fixture}: {stdout}"
    );
    assert!(stdout.contains("string(ok)"), "{fixture}: {stdout}");
}

#[test]
fn run_wasm_backend_calls_iterator_prototype_some_on_a_class_receiver() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_some.js");
}

#[test]
fn run_wasm_backend_calls_iterator_prototype_every_on_a_class_receiver() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_every.js");
}

#[test]
fn run_wasm_backend_calls_iterator_prototype_find_on_a_class_receiver() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_find.js");
}

#[test]
fn run_wasm_backend_calls_iterator_prototype_reduce_on_a_class_receiver() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_reduce.js");
}

#[test]
fn run_wasm_backend_calls_iterator_prototype_for_each_on_a_class_receiver() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_for_each.js");
}

#[test]
fn run_wasm_backend_calls_iterator_prototype_map_on_a_class_receiver() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_map.js");
}

#[test]
fn run_wasm_backend_calls_iterator_prototype_filter_on_a_class_receiver() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_filter.js");
}

#[test]
fn run_wasm_backend_calls_iterator_prototype_flat_map_on_a_class_receiver() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_flat_map.js");
}

#[test]
fn run_wasm_backend_calls_iterator_prototype_take_on_a_class_receiver() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_take.js");
}

#[test]
fn run_wasm_backend_calls_iterator_prototype_drop_on_a_class_receiver() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_drop.js");
}

#[test]
fn run_wasm_backend_calls_iterator_prototype_to_array_on_a_class_receiver() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_to_array.js");
}

/// An agreement check across the three key forms — not, despite an earlier
/// claim here, a permanent guard on the fast path.
///
/// `x.some(cb)` takes the static-key fast path; `x["some"](cb)` and `x[k](cb)`
/// route to the generic tail. The tail is the measured-correct oracle for this
/// receiver, so the three forms must be observationally identical — a fast path
/// may only be faster, never different. This is the oracle the defect would
/// have failed on day one.
///
/// # The one condition it cannot see
///
/// It compares runtime output only, so it cannot detect the fast path not being
/// *selected*. Selection needs `receiver_shape_targets_iterator_helper`
/// (`porffor-aot-wasm/src/functions.rs`) to resolve `"some"` on the receiver's
/// `heap_shape` — and `None` there is exactly the state measured for
/// `class A extends Iterator { constructor() { super(); } }`. If a later
/// lowering or heap-shape change makes it `None` here too, all three arms run
/// the generic tail, the records match, this test is green, and the fast path
/// is untested rather than correct.
///
/// Closing that needs an assertion about the *emitted module*, not its output:
/// `porf inspect` / `debug_dump` is the oracle `frontend.rs` and
/// `porffor-aot-wasm/tests/emit_golden.rs` already use. Until such an assertion
/// exists, read a green run here as "the three forms agree", never as "the fast
/// path ran".
#[test]
fn run_wasm_backend_gives_identical_results_for_static_and_computed_helper_keys() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_computed_key_differential.js");
}

/// `new X().take(1).toArray()`, measured to throw `value is not callable`
/// before the repair: `take`'s fast path wrote nothing into the destination
/// locals, so `.toArray()` was applied to stale scratch.
#[test]
fn run_wasm_backend_chains_take_and_to_array_on_a_class_receiver() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_take_to_array_chain.js");
}
