//! Throw propagation out of raw Wasm control frames.
//!
//! These are the two cases that no cheap check could adjudicate. A `br`
//! immediate is the same width whether it names the right label or the wrong
//! one, so the golden-byte gate (rung G) cannot see the difference; and both
//! the right and the wrong index are in range, so wasm validation accepts
//! either. The only instrument that distinguishes them is running the program,
//! which is what this module does.
//!
//! Both fixtures throw from a getter installed on `Object.prototype`, so the
//! read takes the prototype-walk path in `lila-aot-wasm/src/objects.rs`.
//! That path opens raw `Instruction::If`/`Block` frames, which the branch
//! arithmetic used to be blind to — see the module docs in
//! `lila-aot-wasm/src/code_sink.rs`.

use crate::*;

/// The loop case: the throw must reach the `catch`, not the loop's back edge.
///
/// The `iteration` count is the assertion that matters. Before the label-depth
/// repair the propagating `br` landed one label short, on the `loop` rather
/// than on the handler's `block`, so the body re-ran and threw again forever.
/// `for (var j = 0; j < 2; j++)` cannot bound that — the back edge skips the
/// update expression — so a regression shows up as an enormous `iteration`
/// count followed by a trap, and `count == 1` is what proves the branch landed
/// where it should.
#[test]
fn run_wasm_backend_propagates_a_throwing_property_read_out_of_a_loop() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_throw_propagation_in_loop.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("backend_used: WasmAot"),
        "the fixture must actually run on the Wasm-AOT backend: {stdout}"
    );
    assert_eq!(
        stdout.matches("iteration\n").count(),
        1,
        "the loop body must run exactly once: a throw that branches to the loop's \
         back edge instead of the handler re-runs it without bound. {stdout}"
    );
    assert!(
        !stdout.contains("after read"),
        "execution must not continue past the throwing read: {stdout}"
    );
    assert!(
        stdout.contains("caught TypeError: thrown from a prototype accessor"),
        "the getter's own TypeError must reach the enclosing catch. Matching on the \
         name alone would also accept any other TypeError this read path can raise \
         (`value is not callable`, the proxy paths), so the message is part of the \
         assertion: {stdout}"
    );
    assert!(
        stdout.contains("end\n"),
        "the program must run to completion after the catch: {stdout}"
    );
}

/// The switch case: the throw must reach the `catch`, not be discarded.
///
/// A `switch` lowers to a `block` per case inside a breakable `block`, so a
/// branch one label too shallow lands on a block that just ends. Nothing traps
/// and nothing spins; the `TypeError` is dropped and the program prints `end`.
/// So `end` alone proves nothing here, and `caught TypeError` is the whole
/// assertion.
#[test]
fn run_wasm_backend_propagates_a_throwing_property_read_out_of_a_switch() {
    let output = Command::new(env!("CARGO_BIN_EXE_lila"))
        .arg("run")
        .arg("--execution-backend")
        .arg("wasm")
        .arg(fixture_path("wasm_throw_propagation_in_switch.js"))
        .output()
        .expect("run command should run");

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("backend_used: WasmAot"),
        "the fixture must actually run on the Wasm-AOT backend: {stdout}"
    );
    assert!(
        !stdout.contains("after read"),
        "execution must not continue past the throwing read: {stdout}"
    );
    assert!(
        !stdout.contains("default\n"),
        "the `default` arm must not be reached: {stdout}"
    );
    assert!(
        stdout.contains("caught TypeError: thrown from a prototype accessor"),
        "the getter's own TypeError must reach the enclosing catch rather than being \
         discarded by the switch's block. The message is part of the assertion: the \
         name alone is satisfied by any other TypeError this read path can raise: \
         {stdout}"
    );
}
