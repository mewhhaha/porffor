//! Which dispatch `emit_method_call` *selects* for an `Iterator.prototype`
//! helper, asserted on the emitted module rather than on program output.
//!
//! # Why output tests cannot ask this question
//!
//! Every other test of this family — the fourteen in
//! `porffor-cli/tests/cli/iterator_helpers.rs` and the four in `iterator.rs` —
//! runs a fixture and reads its stdout. That is the right shape for "is the
//! answer correct", and it is structurally unable to answer "which arm handled
//! the call", because a correct dispatch and a correct fall-through are
//! supposed to produce the same output. The batch-5 differential fixture says
//! so in its own doc comment.
//!
//! It mattered, and not hypothetically. The defect those fixtures were written
//! for was a selection failure. `new S()`, for a class with heritage and no
//! explicit constructor, is lowered with `kind = Undefined` and a nullish
//! `possible_kinds`; `receiver_shape_targets_iterator_helper` is therefore
//! false, seven of the helper blocks declined the call, and the generic tail
//! took its statically-nullish shortcut and returned having emitted **no call
//! at all**. The runtime value is an ordinary object, so the emitted nullish
//! test never fired and the caller read stale scratch. `drop` and `flatMap`
//! carry an extra `!receiver_is_array` disjunct and were dispatched correctly
//! throughout, which is the only reason their fixtures were green.
//!
//! # The oracle, and the measurement it is calibrated against
//!
//! `drop` and `flatMap`. Each test builds two programs that differ in exactly
//! one identifier and compares the emitted `porffor::main` bodies.
//!
//! Measured on the pre-repair compiler with `PORFFOR_EMIT_SIZE_REPORT_PATH`
//! over the minimal pair (`new Source().drop(1);` against
//! `new Source().take(1);`): `porffor::main` is **557,233** bytes for `drop`
//! and **557,156** for `take` — the failing helper emits **77 bytes fewer**,
//! which is the whole of the call it did not emit. That is the signal, and it
//! is an order of magnitude above [`PAYLOAD_ENCODING_SLACK`].
//!
//! [`PRELUDE`] is what makes the comparison fair. It calls all four helpers
//! under test on a *helper-shaped* receiver, whose static shape does resolve
//! them, so both programs root and emit the same set of builtins and take the
//! same fast path for those calls; only the statement under test differs.
//!
//! # What this does not claim
//!
//! It is a *differential*, not a fingerprint. It cannot name the emitter that
//! ran, only that two calls were emitted alike, and it would stay green if a
//! future change routed both members of a pair into the generic tail. That is
//! an acceptable residual: `drop` and `flatMap` on this receiver are covered
//! end to end by their own CLI fixtures, so "drop is correct" plus "take is
//! emitted like drop" is a real chain. Rung G is the tool for absolute claims
//! about emitted bytes and is deliberately inapplicable to a change that is
//! meant to move them.

use porffor_aot_wasm::emit;
use porffor_front::{parse, ParseOptions};
use porffor_ir::lower;

/// Matches the worker stack the engine and Test262 runner already use.
/// Lowering and emission recurse deeply enough to overflow the 2 MiB default.
const WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

/// The name `FunctionIdentity::Main` renders into the Wasm `name` section and
/// therefore into `WasmArtifact::function_sizes`. It is the script top level
/// plus the runtime bootstrap, which is where every call site under test lives.
const MAIN_FUNCTION_NAME: &str = "porffor::main";

/// Slack for the one legitimate difference between two otherwise identical
/// dispatches: the `i64.const` holding the interned payload of one helper name
/// versus another, whose LEB128 encodings can differ in length. Far below the
/// measured 77-byte signal and far above the cost of a constant.
const PAYLOAD_ENCODING_SLACK: u32 = 8;

/// Shared by both programs of every pair, before the statement under test.
///
/// `Source` has no explicit constructor on purpose: that is the receiver shape
/// the whole family is about, and adding one changes the static typing the
/// guard reads. The four helper calls that follow are applied to a *helper*
/// receiver (`new Source().drop(1)`), whose shape resolves them, so they are
/// dispatched identically in every program and exist only to make both sides
/// root the same builtins.
const PRELUDE: &str = concat!(
    "class Source extends Iterator {\n",
    "  next() { return { done: true, value: undefined }; }\n",
    "}\n",
    "function identity(value) { return value; }\n",
    "new Source().drop(1).take(1);\n",
    "new Source().drop(1).map(identity);\n",
    "new Source().drop(1).flatMap(identity);\n",
);

fn on_worker<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(WORKER_STACK_BYTES)
        .spawn(work)
        .expect("worker thread should spawn")
        .join()
        .expect("worker thread should not panic")
}

/// Emitted byte length of the top-level body, plus the emitted module.
fn main_body_bytes_and_module(source: String) -> (u32, Vec<u8>) {
    on_worker(move || {
        let unit = parse(&source, ParseOptions::script()).expect("probe should parse");
        let program = lower(&unit);
        let artifact = emit(&program).expect("probe should emit");
        let main = artifact
            .function_sizes
            .iter()
            .find(|summary| summary.name == MAIN_FUNCTION_NAME)
            .unwrap_or_else(|| panic!("emitted module must contain a `{MAIN_FUNCTION_NAME}` body"));
        (main.body_bytes.bytes(), artifact.bytes)
    })
}

fn probe(statement: &str) -> String {
    format!("{PRELUDE}{statement}\n")
}

fn assert_dispatched_alike(subject: &str, control: &str) {
    let (control_bytes, _) = main_body_bytes_and_module(probe(control));
    let (subject_bytes, _) = main_body_bytes_and_module(probe(subject));
    let difference = subject_bytes.abs_diff(control_bytes);
    assert!(
        difference <= PAYLOAD_ENCODING_SLACK,
        "`{subject}` and `{control}` must select the same dispatch on a \
         class-instance receiver, so their emitted `{MAIN_FUNCTION_NAME}` \
         bodies may differ only by the helper-name constant: \
         subject={subject_bytes} control={control_bytes} \
         difference={difference} slack={PAYLOAD_ENCODING_SLACK}"
    );
}

#[test]
fn a_class_receiver_dispatches_take_the_same_way_as_drop() {
    assert_dispatched_alike("new Source().take(1);", "new Source().drop(1);");
}

/// The callback-taking half of the family. `flatMap` is the control here for
/// the same reason `drop` is above: its block routes every non-array receiver
/// to the shared dispatch, so it was correct throughout while `map` was not.
#[test]
fn a_class_receiver_dispatches_map_the_same_way_as_flat_map() {
    assert_dispatched_alike(
        "new Source().map(identity);",
        "new Source().flatMap(identity);",
    );
}

/// Validation is a separate assertion from size. The repair adds a
/// RequireObjectCoercible `if`/`end` and three throw-propagation `if`/`end`
/// pairs to the shared dispatch, and an unbalanced control frame is invisible
/// to a byte count.
#[test]
fn a_class_receiver_helper_call_emits_a_valid_module() {
    for statement in [
        "new Source().take(1);",
        "new Source().drop(1);",
        "new Source().some(identity);",
        "new Source().find(identity);",
        "new Source().filter(identity);",
        "new Source().every(identity);",
        "new Source().reduce(function (a, b) { return a + b; }, 0);",
    ] {
        let (_, bytes) = main_body_bytes_and_module(probe(statement));
        wasmparser::Validator::new()
            .validate_all(&bytes)
            .unwrap_or_else(|err| panic!("emitted module must validate for `{statement}`: {err}"));
    }
}
