//! Function attribution and custom names from the completed function table.

use super::*;
use crate::emitted_function::ModuleFunctionTable;

pub(super) fn append_function_attribution(
    debug_dump: &mut Vec<String>,
    function_table: &ModuleFunctionTable,
) -> Vec<EmittedFunctionSummary> {
    // Emitted-size attribution. `tests/emission/emit_golden.rs` records `debug_dump` per
    // fixture, so these two lines make the largest emitted body a tracked
    // artifact across all 527 CLI fixtures at no extra cost, and give a
    // `Code for function is too large` failure a named suspect.
    //
    // (The `[origin:unknown]` prefix such a failure also carries is a
    // `lila-test262` `FailureOrigin` taxonomy value, not a function name;
    // nothing here changes it, and the engine-side attribution is deliberately
    // index-only so that it cannot change it either.)
    //
    // Both lines use the `key=value` shape `EmittedFunctionSummary::report_lines`
    // uses, with `name=` **last**. Emitted names contain spaces
    // (`get Object.prototype.__proto__`, `Array Iterator.prototype.next`,
    // `get #private`), so a positional layout cannot be parsed back: putting the
    // free-form name last is what makes it unambiguous without quoting, and
    // `tests/emission/emit_golden.rs` parses exactly these keys.
    //
    // One traversal, three consumers. `function_sizes` on the artifact, the two
    // attribution lines below and the opt-in full report are all rendered from
    // `summaries`, so no reader can be told two different things about the same
    // body. The precedent is the `runtime helper functions: 27` line above,
    // which was a second copy of a fact and had drifted by five.
    let function_sizes = function_table.summaries();
    debug_dump.push(EmittedFunctionSummary::attribution_line(
        "largest emitted function",
        EmittedFunctionSummary::largest(&function_sizes),
    ));
    // Reported separately because it is often a different function, and it is
    // the one that predicts Cranelift's virtual-register exhaustion.
    debug_dump.push(EmittedFunctionSummary::attribution_line(
        "most locals in an emitted function",
        EmittedFunctionSummary::most_locals(&function_sizes),
    ));
    debug_dump.push(format!(
        "emitted code bytes: {}",
        function_table.total_body_bytes()
    ));
    if emit_size_report_requested() {
        debug_dump.extend(EmittedFunctionSummary::report_lines(
            &function_sizes,
            usize::MAX,
        ));
    }
    // The same report, written from inside the emitter to a file the caller
    // names. Unlike `debug_dump` this survives every wrapper between here and
    // `main`: the dump has exactly one printer, two crates away, and it was
    // unreachable from `lila build wasm` and from the Test262 wasm-aot backend
    // for the whole of batch 2 — which is how a size claim went a full batch
    // without anyone noticing the measurement was silently empty.
    //
    // The sink truncates rather than appends, so the file always describes the
    // last module emitted.
    write_size_report_file_if_requested(&function_sizes);
    function_sizes
}

pub(super) fn append_function_name_and_check_budget(
    module: &mut Module,
    debug_dump: &mut Vec<String>,
    function_table: &ModuleFunctionTable,
) -> Result<(), EmitError> {
    // The Wasm custom `name` section, built from the same identity table that
    // measured every body. Wasmtime composes its per-function symbol as
    // `wasm[0]::function[N]::<clean_symbol(name)>` from exactly this section
    // (wasmtime/src/compile.rs), so a native-compilation failure that used to
    // read `[origin:unknown] ... Code for function is too large` can now name
    // the function it is about. Custom sections are ignored by validation and
    // do not participate in `largest_wasm_code_body_size`, so this does not
    // move the engine's size-optimized routing threshold.
    module.section(&function_table.name_section());
    debug_dump.push(format!(
        "name section: {} named functions",
        function_table.records().len()
    ));

    // Report-only by default: a budget below the largest body Cranelift accepts
    // today would turn green tests red, and that maximum has not been measured
    // across the fixture corpus yet. Opt in with
    // `LILA_EMIT_FUNCTION_BODY_BUDGET_BYTES` to have the compiler fail in
    // milliseconds with a named function instead of waiting ~37 s for Cranelift
    // to fail anonymously.
    if let Some(budget) = FunctionBodyBudget::from_env() {
        function_table.check_budget(budget)?;
    }

    Ok(())
}
