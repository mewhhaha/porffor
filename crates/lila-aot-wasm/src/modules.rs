//! Wasm emission for ES module graphs.
//!
//! A module graph is linked at compile time into the single `ScriptIr` the rest
//! of the backend emits, so most module semantics need no code here at all: a
//! cross-module binding read is an ordinary environment-slot read of the
//! exporter's cell, and evaluation order is fixed statically.
//!
//! What does need emission is the part that is genuinely dynamic:
//!
//! * [`emit_module_unit_once`] — run a module's hoist or body block exactly
//!   once, which is what makes cycles and repeated `import()` behave;
//! * [`emit_import_meta`] — read the module's `import.meta` object.
//!
//! `import()` is *not* on that list, in either goal. The linker desugars every
//! call site of a graph — `import(`, `import.defer(` and `import.source(` alike
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
//! Module namespace objects are *not* on that list either. `import * as ns` is
//! materialized by the linker as generated Script text — one `Object.create`,
//! one `Object.defineProperty` per export whose getter names the exporter's own
//! binding, and one `Object.preventExtensions` — so it reaches this backend as
//! ordinary object code and needs no emitter. See
//! `lila_ir::modules::namespace`, which owns that source and documents the
//! single invariant the translation gives up (the properties are accessors, so
//! `Object.getOwnPropertyDescriptor` reports `get` rather than `value`).
//! [`emit_module_namespace`] is the seam where a real 10.4.6 exotic object would
//! close that gap, and it stays a stub until there is one.

use super::*;

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
    scan_block(&script.body, &mut highest);
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

    /// `ExprIr::ModuleNamespace`: leaves the identity-cached namespace exotic
    /// object on the stack.
    ///
    /// Deliberately still a stub. Two things would have to change before a real
    /// implementation could be anything but dead code:
    ///
    /// * nothing constructs `ExprIr::ModuleNamespace`. The linker materializes
    ///   `import * as ns` as generated Script text (see the module docs), so a
    ///   namespace object reaches this backend as ordinary object code and this
    ///   arm is never taken;
    /// * `emit` hands `emit_script` a bare [`ScriptIr`], never the `ProgramIr`,
    ///   so `module` here cannot be turned into an export table at all —
    ///   `ProgramIr::modules` is where the sorted export names and their target
    ///   bindings live, and it does not reach this far.
    ///
    /// So the honest thing this arm can do is say which invariant a caller would
    /// be reaching for, rather than emit an object that only looks like one.
    pub(crate) fn emit_module_namespace(
        &mut self,
        _module: u32,
        _function: &mut Function,
    ) -> Result<(), EmitError> {
        Err(unsupported(
            "namespace object (10.4.6 exotic object; the linker emits an ordinary \
             accessor object instead, so this arm means an IR producer got ahead of it)",
        ))
    }
}
