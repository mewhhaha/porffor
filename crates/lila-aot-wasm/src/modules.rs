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
//! and resolves or rejects a promise. Prepared direct evals select that same
//! dispatcher through their proved Script Record; no source is parsed at runtime. A Script gets the same
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
use crate::gc_types::*;
use crate::runtime_helpers::*;
mod completion;
mod entry_completion;
mod evaluation;
mod initialization;
mod realm_import;
mod record_state;
mod runtime;
mod synchronous;
mod traversal;
pub(crate) use runtime::ModuleRuntimeOperation;

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
        let guard = self.runtime_schema().module_unit_guard(module)?;
        guard.load(function);
        function.instruction(&Instruction::I32Eqz);
        // The `if` is a Wasm control frame. `open_frame` emits it and records
        // the label it opened in one call, so the branch arithmetic can see it
        // whether or not anyone remembered to say so.
        self.open_frame(ControlFrameKind::If, function);
        guard.mark_started(function);
        self.push_scope();
        let result = self.compile_block_contents(block, function);
        self.pop_scope();
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        result
    }

    /// Prepared direct evals retain a proved Script Record and use that
    /// record's already compiled async dispatcher. The canonical dispatcher
    /// owns the promise, coercion, module jobs, namespace and rejection value.
    /// An absent/ambiguous origin or an undiscovered target remains AOT debt.
    pub(crate) fn emit_dynamic_import(
        &mut self,
        referrer: Option<u32>,
        phase: lila_ir::ImportPhaseIr,
        specifier: &TypedExpr,
        options: Option<&TypedExpr>,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let dispatcher = referrer
            .and_then(|referrer| {
                self.functions
                    .module_graph()
                    .and_then(|graph| graph.script_import_dispatcher(referrer, phase))
            })
            .cloned()
            .ok_or_else(|| {
                unsupported(
                "dynamic import without a compiled graph (the host lowered a source that writes \
                 `import()` without loading its targets)",
            )
            })?;
        let meta = self
            .functions
            .get(dispatcher.function())
            .cloned()
            .ok_or_else(|| {
                unsupported("prepared Script import dispatcher requires its compiled entry")
            })?;
        let schema = self.runtime_schema();
        // ImportCall evaluates both operands before creating its promise.
        let argument = schema.reserve_value_local(function);
        self.compile_expr_to_value(specifier, &argument, function)?;
        let import_options = schema.reserve_value_local(function);
        if let Some(options) = options {
            self.compile_expr_to_value(options, &import_options, function)?;
        } else {
            import_options.set_undefined(function);
        }
        let pending = schema.reserve_completion(function);
        pending.initialize(function);
        let realm = self.load_current_realm(function);
        // Initialization creates canonical cells, without evaluating targets.
        // The same graph is reusable if the direct eval executes in another Realm.
        self.emit_module_initialize_in_realm(&realm, &pending, function)?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        let prototype = schema.reserve_value_local(function);
        self.emit_load_non_array_realm_intrinsic(
            &realm,
            NonArrayRealmIntrinsicSlot::AsyncFunctionPrototype,
            &prototype,
            function,
        );
        let callable = schema.reserve_gc_local(function).initialize(
            self.emit_dynamic_source_function_record(&meta, &realm, &prototype, function)?,
            function,
        );
        let callee = schema.reserve_value_local(function);
        callee.set_reference(&callable, schema, function);
        let receiver = schema.reserve_value_local(function);
        receiver.set_undefined(function);
        let arguments = self.emit_pre_evaluated_arg_vector(&[&argument, &import_options], function);
        self.emit_function_or_proxy_call_with_argv(
            &callee, &receiver, &arguments, &pending, function,
        )?;
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        output.copy_from(pending.value(), function);
        arguments.clear(function);
        receiver.clear(function);
        callee.clear(function);
        callable.clear(function);
        prototype.clear(function);
        realm.clear(function);
        pending.clear(function);
        import_options.clear(function);
        argument.clear(function);
        Ok(())
    }

    /// `ExprIr::ImportMeta`: retains its explicit unsupported source boundary.
    pub(crate) fn emit_import_meta(
        &mut self,
        _module: u32,
        _output: &ValueLocals,
        _function: &mut Function,
    ) -> Result<(), EmitError> {
        Err(unsupported("import.meta"))
    }
}
