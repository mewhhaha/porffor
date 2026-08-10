//! `Iterator.prototype` helper CLI integration tests, for the one receiver
//! shape that broke every helper but `forEach`.
//!
//! # Why this module exists next to `iterator.rs`
//!
//! `iterator.rs` already carries four helper fixtures — `some`, `every`,
//! `find`, `reduce`. All four failed. The mechanism, identified in batch 6 and
//! **not** what an earlier version of this comment said, is that no code was
//! emitted for the call.
//!
//! `new S()`, for a class with heritage and no explicit constructor — the
//! receiver every fixture here uses — is lowered with `kind = Undefined` and a
//! nullish `possible_kinds` (`porf inspect` on `class D extends Iterator {}
//! new D();` reports `result=undefined`, while the same class without heritage,
//! or with an explicit constructor, reports `result=object`). The static shape
//! guard `receiver_shape_targets_iterator_helper` is therefore false, the seven
//! blocks that carry only that guard — `find`, `reduce`, `take`, `map`,
//! `every`, `some`, `filter` — declined the call, and `emit_method_call`'s
//! generic tail took its statically-nullish shortcut: it emitted a nullish
//! throw that never fires at run time and returned without writing either
//! destination local. Everything the fixtures below measured is that one hole
//! read back — a stale-typed value with the callback never invoked, a
//! `TypeError: value is not callable` when the stale pair is used as the next
//! receiver, and a trap inside `helper::value_to_string`.
//!
//! Two blocks did not decline: `drop` and `flatMap` carry an extra
//! `!receiver_is_array` disjunct and reach the shared dispatch for any
//! non-array receiver. That is the whole reason those two fixtures were green
//! while `take`'s — the same program modulo one identifier, with a
//! byte-identical result shape in `lowering.rs` — was red. Emitted-size
//! attribution puts a number on it: `porffor::main` is 69 bytes larger for
//! `new Source().flatMap(identity)` than for `.some(identity)`, `.map(identity)`
//! or `.find(identity)`, and those three are byte-identical to each other.
//!
//! The root cause one level up is the `undefined` typing, which belongs to
//! `porffor-ir` and is recorded as an exact patch in
//! `target/lane-notes/iterator-helper-static-key-call-on-a-class-receiver-b6-integration.md`.
//! Fixing it there would make the static shape guard true and is the better
//! long-term answer; the emitter change here is what makes the dispatch correct
//! whether or not that typing is ever right.
//!
//! The four `iterator.rs` tests flipping green is necessary evidence, not
//! attribution: those fixtures also assert IteratorClose obligations that a
//! concurrent round is changing in the same tree, and they discard the thrown
//! label (`assert!(output.status.success())` only), so which check failed is
//! unknown from their output alone.
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
/// *selected* — and that turned out to be the defect itself rather than a
/// hypothetical. `receiver_shape_targets_iterator_helper`
/// (`porffor-aot-wasm/src/functions.rs`) does not resolve `"some"` on a class
/// instance's `heap_shape`, so before batch 6 all three arms of this fixture
/// were meant to disagree and did; after it, all three reach a dynamic dispatch
/// and agree. Agreement alone can therefore be reached from either end, and a
/// future change that made the static-key form fall back to the generic tail
/// again would leave this test green.
///
/// That gap is closed by an assertion about the *emitted module* rather than
/// its output, in `crates/porffor-aot-wasm/tests/iterator_helper_dispatch.rs`:
/// it compares the emitted `porffor::main` body of a `take` program against a
/// `drop` program that differs by one identifier, so a `take` that stops being
/// dispatched like `drop` shows up as a size divergence with no runtime
/// component at all. Read a green run *here* as "the three forms agree", and
/// that test as "the same dispatch was selected".
#[test]
fn run_wasm_backend_gives_identical_results_for_static_and_computed_helper_keys() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_computed_key_differential.js");
}

/// `new X().take(1).toArray()`, measured to throw `value is not callable`
/// before the repair.
///
/// Note which `take` is which, because the fixture contains both and they took
/// opposite paths. `new Source().take(1)` has a *class instance* receiver,
/// whose static shape does not resolve `take` to `Iterator.prototype.take`, so
/// the fast-path guard was false and the call fell through to the generic tail.
/// `new Source().drop(1).take(2)` in the same fixture has a *helper* receiver,
/// whose shape does resolve it, so that one always took the fast path and was
/// always correct. The pair is the sharpest evidence available that the guard,
/// not the emission, was the defect — same helper, same argument, opposite
/// outcome, one static shape apart.
#[test]
fn run_wasm_backend_chains_take_and_to_array_on_a_class_receiver() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_take_to_array_chain.js");
}

/// Abrupt completions raised by the dispatch itself rather than by a callback.
///
/// The other thirteen fixtures only ever throw from a callback or from `next`,
/// so they exercise the call and nothing before it. This one throws from the
/// receiver expression and from the `[[Get]]` of the helper property, and pins
/// that an abrupt `[[Get]]` leaves the argument list unevaluated (7.3.11
/// GetMethod precedes ArgumentListEvaluation).
///
/// It is a **regression** test rather than a repair witness, and that is
/// measured: it already answered `string(ok)` before the repair, because its
/// receivers are ordinary object literals and a class with an explicit
/// constructor — receivers the generic tail handled correctly. Batch 6 moves
/// exactly those receivers onto the shared helper dispatch, which had none of
/// these checks, so this is the test that would catch the routing change
/// trading one defect for another.
#[test]
fn run_wasm_backend_propagates_abrupt_completions_from_helper_dispatch() {
    assert_helper_fixture_is_ok("wasm_iterator_helper_class_receiver_abrupt_dispatch.js");
}
