//! Wasm emission for ES module graphs.
//!
//! A validated Module-entry graph carries private activation, canonical cell and
//! ordered phase-aware request operations. Allocation publishes every canonical
//! environment before instantiation; runtime DFS owns cycle roots, cached
//! Evaluate promises, async parents and exact rejection values. Source Await
//! resumes the existing async ABI after a separate private instantiation phase.
//!
//! Script-entry and source-phase graphs retain their explicit merged-driver
//! admission boundary.
//!
//! `synchronous` owns allocation/instantiation and canonical cells; `evaluation`
//! and `completion` own runtime DFS and Promise completion; `traversal` owns
//! pure readiness/gather walks. `entry_completion` projects the primary entry.
//! The retained paths below emit `ModuleUnitOnce` and `import.meta` operations.
//!
//! The linker desugars every `import()` call, in either goal, from every
//! graph call site — `import(`, `import.defer(` and `import.source(` alike
//! — into an ordinary call to a generated dispatcher function that `ToString`s
//! the specifier, compares it against the specifiers compiled into the artifact
//! and resolves or rejects a promise, so no `ImportCall` node reaches this
//! backend and no source is parsed at runtime, ever. A Script gets the same
//! treatment as a module: `lila_ir::lower_script_graph` compiles the targets
//! of a Script's `import()` calls into the same artifact and wraps them in one
//! strict function so the Script itself stays Script code. See
//! `lila_ir::modules::dynamic`, and [`emit_dynamic_import`] for the one case
//! that still reaches this file.
//!
//! Namespace constructors carry their private export-reader table directly in
//! `ExprIr::ModuleNamespace`. The canonical runtime object implementation lives
//! in `objects::module_namespace`, alongside the internal methods it dispatches.

use super::*;
mod completion;
mod entry_completion;
mod evaluation;
mod runtime;
mod synchronous;
mod traversal;
pub(crate) use runtime::ModuleRuntimeOperation;
pub(crate) use synchronous::module_execution_record_count;

/// Message every unimplemented module emission reports, so a module compile
/// fails with one recognisable diagnostic rather than a generic backend error.
fn unsupported(feature: &str) -> EmitError {
    EmitError::unsupported(format!(
        "unsupported in lila wasm-aot: module {feature} emission"
    ))
}

/// Number of module-unit guard globals the artifact needs.
///
/// One per distinct unit id reachable through a [`StatementIr::ModuleUnitOnce`]
/// anywhere in the script, so the count is a pure function of the lowered IR
/// and needs no separate plumbing from `ProgramIr::modules`.
#[must_use]
pub(crate) fn module_unit_guard_count(script: &ScriptIr) -> u32 {
    fn scan_block(block: &BlockIr, highest: &mut Option<u32>) {
        for statement in &block.statements {
            scan_statement(statement, highest);
        }
    }

    fn scan_statement(statement: &StatementIr, highest: &mut Option<u32>) {
        if let StatementIr::ModuleUnitOnce { module, block } = statement {
            *highest = Some(highest.map_or(*module, |current: u32| current.max(*module)));
            scan_block(block, highest);
        }
    }

    let mut highest = None;
    for body in script.executable_script_bodies() {
        scan_block(body, &mut highest);
    }
    for function in &script.functions {
        scan_block(&function.body, &mut highest);
    }
    highest.map_or(0, |highest| highest + 1)
}

impl FunctionBuilder<'_> {
    /// Wasm global index of module `unit`'s "already evaluated" guard.
    ///
    /// The guards sit immediately after the template-object globals, which
    /// themselves sit after the fixed registry, so the block stays dense and no
    /// existing index moves.
    pub(crate) fn module_unit_guard_global_index(&self, unit: u32) -> u32 {
        GLOBAL_INDEX_REGISTRY.len() as u32 + self.strings.template_objects.len() as u32 + unit
    }

    /// `StatementIr::ModuleUnitOnce`: run `block` the first time control
    /// reaches it and no-op afterwards.
    ///
    /// The guard is set *before* the body runs, not after, which is what makes
    /// a cyclic graph terminate: a unit that re-enters itself while evaluating
    /// sees its own guard already set and returns instead of recursing.
    pub(crate) fn emit_module_unit_once(
        &mut self,
        module: u32,
        block: &BlockIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let guard = self.module_unit_guard_global_index(module);
        function.instruction(&Instruction::GlobalGet(guard));
        function.instruction(&Instruction::I32Eqz);
        // The `if` is a Wasm control frame. `open_frame` emits it and records
        // the label it opened in one call, so the branch arithmetic can see it
        // whether or not anyone remembered to say so.
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::I32Const(1));
        function.instruction(&Instruction::GlobalSet(guard));
        self.push_scope();
        let result = self.compile_block_contents(block, function);
        self.pop_scope();
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result
    }

    /// `ExprIr::DynamicImport`: leaves a promise payload on the stack.
    ///
    /// Reaching this arm means the *host* compiled a source that writes
    /// `import()` without supplying the graph its specifiers name, so there is
    /// nothing in the artifact to resolve against and nothing this emitter could
    /// invent. Both goals normally supply one — a module through
    /// `lower_module_graph`, a Script through `lower_script_graph` — and
    /// `lila_ir::modules::dynamic` desugars every call site of a graph into
    /// an ordinary call to a generated dispatcher, so no `ImportCall` survives
    /// to this backend.
    ///
    /// What is left here is the case the linker cannot reach at all: a Script
    /// the loader could not read as module code (a sloppy `with`, an octal
    /// literal), whose `import()` specifiers therefore could not be discovered.
    /// Closing it needs the entry's dynamic-import sites read off a *Script*
    /// parse, not a Wasm emitter — an artifact with no target compiled into it
    /// can only reject, and rejecting silently would be a wrong answer rather
    /// than a missing one.
    pub(crate) fn emit_dynamic_import(
        &mut self,
        _referrer: Option<u32>,
        _specifier: &TypedExpr,
        _options: Option<&TypedExpr>,
        _function: &mut Function,
    ) -> Result<(), EmitError> {
        Err(unsupported(
            "dynamic import without a compiled graph (the host lowered a source that writes \
             `import()` without loading its targets)",
        ))
    }

    /// `ExprIr::ImportMeta`: leaves the module's `import.meta` object on the
    /// stack.
    pub(crate) fn emit_import_meta(
        &mut self,
        _module: u32,
        _function: &mut Function,
    ) -> Result<(), EmitError> {
        Err(unsupported("import.meta"))
    }
}
