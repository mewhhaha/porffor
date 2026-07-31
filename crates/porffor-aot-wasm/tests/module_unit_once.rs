//! `StatementIr::ModuleUnitOnce` emission.
//!
//! The linker in `porffor-ir::modules::link` fixes evaluation order statically,
//! so the merged script it produces today runs every unit body exactly once
//! without a runtime guard. `ModuleUnitOnce` is what the *dynamic* half needs —
//! `import()` of an already-evaluated module, and a cycle that re-enters a unit
//! mid-evaluation — and neither of those can be written against the current
//! linker. This test therefore drives the emitter directly: it lowers an
//! ordinary script, rewraps its body in a `ModuleUnitOnce`, and proves the
//! emitted module still validates.
//!
//! Validation is the real assertion. A guard emitted with the wrong global
//! index, an unbalanced `if`/`end`, or a control frame the builder failed to
//! push all show up as a Wasm validation failure and nothing else.

use porffor_aot_wasm::emit;
use porffor_front::{parse, ParseOptions};
use porffor_ir::{lower, BlockIr, StatementIr};

/// Matches the worker stack the engine and Test262 runner already use.
const WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

fn on_worker<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(WORKER_STACK_BYTES)
        .spawn(work)
        .expect("worker thread should spawn")
        .join()
        .expect("worker thread should not panic")
}

fn emitted_bytes_wrapping_body_in_module_units(source: &'static str, units: u32) -> Vec<u8> {
    on_worker(move || {
        let unit = parse(source, ParseOptions::script()).expect("fixture should parse");
        let mut program = lower(&unit);
        let script = program.script.as_mut().expect("fixture should lower");

        // Nest one `ModuleUnitOnce` per unit, innermost holding the real body.
        // Nesting also proves the guards do not collide: each level owns its own
        // global, and the innermost body must still run.
        let mut inner = BlockIr {
            statements: std::mem::take(&mut script.body.statements),
            result_kind: script.body.result_kind,
            lexical_environment: None,
        };
        for module in (0..units).rev() {
            inner = BlockIr {
                statements: vec![StatementIr::ModuleUnitOnce {
                    module,
                    block: Box::new(inner),
                }],
                result_kind: script.body.result_kind,
                lexical_environment: None,
            };
        }
        script.body.statements = inner.statements;

        emit(&program)
            .expect("module-unit script should emit")
            .bytes
    })
}

#[test]
fn a_guarded_unit_body_emits_a_valid_module() {
    let bytes = emitted_bytes_wrapping_body_in_module_units("print(1 + 1);", 1);
    wasmparser::Validator::new()
        .validate_all(&bytes)
        .expect("emitted module must validate");
}

/// The guard index is offset past the fixed global registry and the
/// template-object globals, both of which are sized differently depending on
/// whether the script needs the heap. A script that touches neither strings nor
/// objects takes the other branch, so it is validated separately.
#[test]
fn a_guarded_unit_body_validates_without_the_heap() {
    let bytes = emitted_bytes_wrapping_body_in_module_units("1 + 1;", 1);
    wasmparser::Validator::new()
        .validate_all(&bytes)
        .expect("emitted module must validate");
}

#[test]
fn nested_unit_guards_emit_a_valid_module() {
    let bytes = emitted_bytes_wrapping_body_in_module_units("print(1 + 1);", 3);
    wasmparser::Validator::new()
        .validate_all(&bytes)
        .expect("emitted module must validate");
}

#[test]
fn guard_globals_are_only_reserved_for_units_that_exist() {
    let without = emitted_bytes_wrapping_body_in_module_units("print(1 + 1);", 0);
    let with = emitted_bytes_wrapping_body_in_module_units("print(1 + 1);", 1);
    assert!(
        with.len() > without.len(),
        "a guarded body must add both a guard global and the guard code"
    );
}
