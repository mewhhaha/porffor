use std::borrow::Cow;

use porffor_ir::DerivedConstructorActivationIr;

use crate::functions::{
    emit_array_alloc_helper_function, emit_function_object_alloc_helper_function,
};
use crate::objects::{
    emit_object_append_accessor_property_helper_function,
    emit_object_append_data_property_helper_function, emit_plain_object_alloc_helper_function,
};
use porffor_ir::{
    FunctionExecutionKind, HostBuiltinId, ProgramIr, ScriptIr, StandardBuiltinId, ValueKind,
};
use wasm_encoder::{
    CodeSection, ConstExpr, DataSection, ElementSection, Elements, ExportKind, ExportSection,
    Function, FunctionSection, GlobalSection, GlobalType, ImportSection, Instruction,
    MemorySection, MemoryType, Module, RefType, TableSection, TableType, TypeSection, ValType,
};

use super::*;

#[derive(Debug, Clone, Copy)]
pub(crate) enum ControlFrameKind {
    If,
    Block,
    Loop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ControlTarget {
    pub(crate) frame: usize,
    pub(crate) environment_depth: u32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct IteratorCloseOnThrowLocals {
    pub(crate) iterator_payload_local: u32,
    pub(crate) iterator_tag_local: u32,
    pub(crate) key_local: u32,
    pub(crate) return_payload_local: u32,
    pub(crate) return_tag_local: u32,
    pub(crate) result_payload_local: u32,
    pub(crate) result_tag_local: u32,
    pub(crate) saved_payload_local: u32,
    pub(crate) saved_tag_local: u32,
    pub(crate) saved_completion_local: u32,
    pub(crate) saved_aux_local: u32,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct LoopTargets {
    pub(crate) continue_frame: ControlTarget,
}

#[derive(Debug, Clone)]
pub(crate) struct LabelTargets {
    pub(crate) name: String,
    pub(crate) break_frame: ControlTarget,
    pub(crate) continue_frame: Option<ControlTarget>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BindingStorage {
    Fixed { payload_local: u32, kind: ValueKind },
    Dynamic { tag_local: u32, payload_local: u32 },
    EnvSlot { slot: u32, hops: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReturnAbi {
    MainExport,
    MultiValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OrdinarySetDataOnReceiverEmission {
    Inline,
    Outlined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CompletionKind {
    Normal,
    Throw,
    Return,
    Break,
    Continue,
}

impl CompletionKind {
    pub(crate) const fn code(self) -> i64 {
        match self {
            Self::Normal => COMPLETION_KIND_NORMAL,
            Self::Throw => COMPLETION_KIND_THROW,
            Self::Return => COMPLETION_KIND_RETURN,
            Self::Break => COMPLETION_KIND_BREAK,
            Self::Continue => COMPLETION_KIND_CONTINUE,
        }
    }
}

pub(crate) struct FunctionBuilder<'a> {
    pub(crate) body: &'a BlockIr,
    pub(crate) params: &'a [FunctionParamIr],
    pub(crate) owned_env_bindings: &'a [OwnedEnvBindingIr],
    pub(crate) captured_bindings: &'a [porffor_ir::CapturedBindingIr],
    pub(crate) strings: &'a StringPool,
    pub(crate) functions: &'a FunctionMetaRegistry,
    pub(crate) function_id: Option<FunctionId>,
    pub(crate) function_flavor: FunctionFlavor,
    pub(crate) lexical_derived_activation: Option<&'a DerivedConstructorActivationIr>,
    pub(crate) is_derived_constructor: bool,
    pub(crate) strict: bool,
    pub(crate) self_binding_name: Option<String>,
    pub(crate) script_global_bindings: BTreeMap<String, ScriptGlobalBindingKind>,
    pub(crate) uses_heap: bool,
    pub(crate) return_abi: ReturnAbi,
    pub(crate) binding_scopes: Vec<BTreeMap<String, BindingStorage>>,
    pub(crate) hoisted_vars: Vec<String>,
    pub(crate) next_binding_local: u32,
    pub(crate) total_binding_local_count: u32,
    pub(crate) temp_local_count: u32,
    pub(crate) current_env_local: u32,
    pub(crate) class_function_context_local: u32,
    pub(crate) active_private_environment_locals: Vec<u32>,
    pub(crate) named_function_context_local: u32,
    pub(crate) result_local: u32,
    pub(crate) result_tag_local: u32,
    pub(crate) completion_local: u32,
    pub(crate) completion_aux_local: u32,
    pub(crate) scratch_local: u32,
    pub(crate) temp_local_base: u32,
    pub(crate) temp_stack_depth: u32,
    pub(crate) max_temp_stack_depth: u32,
    pub(crate) environment_depth: u32,
    pub(crate) this_payload_local: Option<u32>,
    pub(crate) this_tag_local: Option<u32>,
    pub(crate) control_stack: Vec<ControlFrameKind>,
    pub(crate) breakable_stack: Vec<ControlTarget>,
    pub(crate) loop_stack: Vec<LoopTargets>,
    pub(crate) label_stack: Vec<LabelTargets>,
    pub(crate) throw_handler_stack: Vec<ControlTarget>,
    pub(crate) finally_stack: Vec<ControlTarget>,
    pub(crate) generator_finalizer_depth: u32,
    pub(crate) stub_standard_builtin_body: bool,
    pub(crate) runtime_bootstrap_plan: RuntimeBootstrapPlan,
    pub(crate) heap_alloc_function_index: Option<u32>,
    pub(crate) object_append_data_property_function_index: Option<u32>,
    pub(crate) object_append_accessor_property_function_index: Option<u32>,
    pub(crate) function_object_alloc_function_index: Option<u32>,
    pub(crate) plain_object_alloc_function_index: Option<u32>,
    pub(crate) array_alloc_function_index: Option<u32>,
    /// When false, `emit_object_read_ordinary` inlines its body instead of
    /// emitting a call to the shared object-read runtime helper. Set false only
    /// while compiling the object-read helper itself (to avoid self-recursion).
    pub(crate) outline_object_read: bool,
    /// When false, `emit_object_write` inlines its body instead of emitting a
    /// call to the shared object-write runtime helper. Set false only while
    /// compiling the object-write helper itself.
    pub(crate) outline_object_write: bool,
    /// When false, `emit_object_define_data_with_flag_locals` inlines its body
    /// instead of emitting a call to the shared object-define-data helper. Set
    /// false only while compiling that helper itself. Realm/global bootstrap
    /// defines hundreds of data properties, so outlining this keeps the
    /// bootstrap-style functions well under Cranelift's per-function limit.
    pub(crate) outline_object_define_data: bool,
    /// When false, `emit_function_handle_call_with_argv_inner` inlines the
    /// plain function-call dispatcher instead of calling the shared helper.
    /// Set false only while compiling that helper itself.
    pub(crate) outline_function_call: bool,
    /// When false, `emit_function_or_proxy_call_with_argv_inner` inlines the
    /// proxy-aware call-dispatch state machine instead of calling the shared
    /// helper. Set false only while compiling that helper itself.
    pub(crate) outline_proxy_call: bool,
    /// When false, `emit_function_or_proxy_construct_with_argv` inlines the
    /// proxy-aware construct-dispatch state machine instead of calling the
    /// shared helper. Set false only while compiling that helper itself.
    pub(crate) outline_proxy_construct: bool,
    /// When false, `emit_string_payload_equality_i32` inlines its byte-compare
    /// loop instead of calling the shared string-equality helper. Set false only
    /// while compiling that helper itself. Builtin bodies compare interned
    /// string payloads at thousands of sites (property-name matching, key
    /// switches), and the inline loop is ~65 instructions per site, so outlining
    /// it keeps the largest builtin bodies under Cranelift's per-function
    /// virtual-register limit.
    pub(crate) outline_string_equality: bool,
    /// When false, `emit_number_to_string_payload` inlines its digit-emission
    /// state machine instead of calling the shared helper. Set false only while
    /// compiling that helper itself. Number formatting appears in nearly every
    /// builtin (ToString of numeric results, join/serialize paths), and the
    /// inline expansion is several KB per site.
    pub(crate) outline_number_to_string: bool,
    /// When false, `emit_string_to_number_payload` inlines its parse state
    /// machine instead of calling the shared helper. Set false only while
    /// compiling that helper itself. String-to-number parsing (ToNumber of
    /// string operands) is similarly several KB per inline site.
    pub(crate) outline_string_to_number: bool,
    /// When false, `emit_value_to_string_payload` inlines the full dynamic
    /// ToString composite (per-kind dispatch, ToPrimitive on objects, array
    /// join, function source text) instead of calling the shared helper. Set
    /// false only while compiling that helper itself. Every dynamic string
    /// concatenation and ToString site otherwise pays tens of KB inline.
    pub(crate) outline_value_to_string: bool,
    /// When false, `emit_value_to_number_payload` inlines the full ToNumber
    /// composite (per-kind dispatch, ToPrimitive on objects, array→string,
    /// BigInt/Symbol throw sites) instead of calling the shared helper. Set
    /// false only while compiling that helper itself. ToNumber appears at ~130
    /// builtin sites, each otherwise several KB inline.
    pub(crate) outline_value_to_number: bool,
    /// When false, `emit_value_to_numeric_locals` inlines the full dynamic
    /// ToNumeric composite instead of calling the shared helper. Set false only
    /// while compiling that helper itself. Object coercion dominates repeated
    /// arithmetic expressions, so outlining it keeps user functions below
    /// Cranelift's per-function virtual-register limit.
    pub(crate) outline_value_to_numeric: bool,
    /// When false, `emit_object_get_prototype_of_with_depth` inlines the
    /// proxy-aware `[[GetPrototypeOf]]` state machine instead of emitting a call
    /// to the shared helper. Set false only while compiling that helper itself.
    /// The proxy get-prototype-of expansion (which mutually inlines the
    /// proxy-aware `[[IsExtensible]]` walk to a fixed depth) is ~356KB per
    /// `instanceof` site under a realm/proxy-enabled module, so outlining it is
    /// what keeps `instanceof other.X` reading functions from blowing past
    /// Cranelift's per-function code-size limit.
    pub(crate) outline_object_get_prototype_of: bool,
    /// When false, `emit_object_is_extensible_i32_with_depth` inlines the
    /// proxy-aware `[[IsExtensible]]` state machine instead of emitting a call to
    /// the shared helper. Set false only while compiling that helper itself.
    pub(crate) outline_object_is_extensible: bool,
    /// When false, `emit_object_read_with_key_tag` inlines the proxy-aware
    /// `[[Get]]` wrapper (proxy-handler check, `get` trap invoke, invariant
    /// validation, one-level nested-proxy unroll) instead of emitting a call to
    /// the shared helper. Set false only while compiling that helper itself. The
    /// proxy read wrapper is ~21KB per read site under a realm/proxy-enabled
    /// module, and dynamic reads are the single most common operation, so
    /// outlining it is the dominant code-size win for realm modules.
    pub(crate) outline_object_read_proxy: bool,
    pub(crate) outline_array_write: bool,
    /// Controls whether the shared object-write helper emits receiver-side
    /// ordinary data writes as calls to their dedicated runtime helper. Other
    /// builders keep these writes inline so only the repeated copies inside the
    /// already-outlined object-write state machine are extracted.
    pub(crate) ordinary_set_data_on_receiver_emission: OrdinarySetDataOnReceiverEmission,
    /// When `Some(local)`, `emit_object_write` is being emitted as the shared
    /// outlined write helper and must decide sloppy/strict `[[Set]]` failure
    /// behavior from the runtime value of `local` (a helper parameter carrying
    /// the calling function's strictness) rather than from the compile-time
    /// `is_current_function_strict()` of the helper body itself (which is a
    /// fixed, mode-less runtime helper). `None` for inline emission, where the
    /// compile-time strictness of the enclosing function is authoritative.
    pub(crate) object_write_strict_flag_local: Option<u32>,
}

pub fn emit(program: &ProgramIr) -> Result<WasmArtifact, EmitError> {
    // Diagnostics are scanned *before* `program.script`: a stage that reports a
    // reason for failing also declines to produce a script, so checking the
    // script first replaces every honest diagnostic with the generic "no
    // lowered script ir". Module linking is the case that made this visible —
    // an unresolved specifier is a `LinkError`, not `Unsupported`, and used to
    // reach the backend as nothing at all.
    if let Some(diagnostic) = program.diagnostics.iter().find(|diagnostic| {
        matches!(
            diagnostic.kind,
            porffor_ir::IrDiagnosticKind::Unsupported
                | porffor_ir::IrDiagnosticKind::LinkError
                | porffor_ir::IrDiagnosticKind::EarlyError
        )
    }) {
        return Err(EmitError::unsupported(diagnostic.message.clone()));
    }
    let script = program.script.as_ref().ok_or_else(|| {
        EmitError::unsupported("unsupported in porffor wasm-aot first slice: no lowered script ir")
    })?;
    emit_script(script)
}

fn emit_script(script: &ScriptIr) -> Result<WasmArtifact, EmitError> {
    for function in script
        .functions
        .iter()
        .filter(|function| function.execution_kind == FunctionExecutionKind::AsyncGenerator)
    {
        if let Some(feature) = function
            .body
            .statements
            .iter()
            .find_map(async_generator_dispatcher_unsupported_feature)
        {
            return Err(EmitError::unsupported(format!(
                "unsupported in porffor wasm-aot first slice: async-generator body dispatcher for `{}` does not yet support {feature}",
                function.name
            )));
        }
    }

    // Emission fixpoint over the builtin stub partitions (standard and host).
    // The seed partitions come from the script text
    // (`should_stub_standard_builtin`; host builtins the script references).
    // Each pass records, via `FunctionMetaRegistry`, every builtin whose
    // function value was materialized somewhere (its funcref-table slot became
    // runtime-reachable) or whose body was direct-called. Any such builtin
    // that was stubbed this pass is force-compiled on the next pass, so no
    // reachable function value can ever resolve to the shared "not emitted"
    // stub. The forced sets grow monotonically and are bounded by the builtin
    // counts, so the loop terminates; in practice it converges in 2-4 passes.
    let mut forced = ForcedBuiltins::default();
    loop {
        let (artifact, touched_stubbed) = emit_script_with_forced_builtins(script, &forced)?;
        if touched_stubbed.standard.is_empty()
            && touched_stubbed.host.is_empty()
            && !touched_stubbed.number_pow_import
        {
            return Ok(artifact);
        }
        forced.standard.extend(touched_stubbed.standard);
        forced.host.extend(touched_stubbed.host);
        forced.number_pow_import |= touched_stubbed.number_pow_import;
    }
}

#[derive(Clone, Copy)]
enum AsyncGeneratorSuspension {
    Await,
    Yield,
}

fn async_generator_contains_suspension(
    statement: &StatementIr,
    suspension: AsyncGeneratorSuspension,
) -> bool {
    match statement {
        StatementIr::AsyncAwait { .. } => matches!(suspension, AsyncGeneratorSuspension::Await),
        StatementIr::GeneratorYield { .. } => {
            matches!(suspension, AsyncGeneratorSuspension::Yield)
        }
        StatementIr::GeneratorLoop {
            before_suspension,
            suspension_statement,
            after_suspension,
            ..
        } => before_suspension
            .iter()
            .chain(std::iter::once(suspension_statement.as_ref()))
            .chain(after_suspension)
            .any(|statement| async_generator_contains_suspension(statement, suspension)),
        StatementIr::GeneratorIf {
            then_before_yield,
            then_yield_statement,
            then_after_yield,
            else_before_yield,
            else_yield_statement,
            else_after_yield,
            ..
        } => then_before_yield
            .iter()
            .chain(then_yield_statement.as_deref())
            .chain(then_after_yield)
            .chain(else_before_yield)
            .chain(else_yield_statement.as_deref())
            .chain(else_after_yield)
            .any(|statement| async_generator_contains_suspension(statement, suspension)),
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => statements
            .iter()
            .any(|statement| async_generator_contains_suspension(statement, suspension)),
        StatementIr::Block(block) => block
            .statements
            .iter()
            .any(|statement| async_generator_contains_suspension(statement, suspension)),
        StatementIr::If {
            then_branch,
            else_branch,
            ..
        } => {
            async_generator_contains_suspension(then_branch, suspension)
                || else_branch.as_ref().is_some_and(|else_branch| {
                    async_generator_contains_suspension(else_branch, suspension)
                })
        }
        // A `for await` loop is itself a suspension: it awaits `next()` once per
        // iteration and awaits the iterator close on exit. Only recursing into
        // the body would report a nested for-await as suspension-free and let it
        // through a guard that exists precisely to keep second suspensions out.
        StatementIr::ForOfArray {
            async_plan: Some(_),
            ..
        }
        | StatementIr::ForOfIterator {
            async_plan: Some(_),
            ..
        } => matches!(suspension, AsyncGeneratorSuspension::Await),
        StatementIr::While { body, .. }
        | StatementIr::DoWhile { body, .. }
        | StatementIr::For { body, .. }
        | StatementIr::ForOfArray { body, .. }
        | StatementIr::ForOfString { body, .. }
        | StatementIr::ForOfIterator { body, .. }
        | StatementIr::ForInArray { body, .. }
        | StatementIr::ForInString { body, .. }
        | StatementIr::ForInObject { body, .. }
        | StatementIr::Labelled {
            statement: body, ..
        } => async_generator_contains_suspension(body, suspension),
        StatementIr::Switch {
            lexical_declarations,
            cases,
            ..
        } => lexical_declarations
            .iter()
            .chain(cases.iter().flat_map(|case| case.body.statements.iter()))
            .any(|statement| async_generator_contains_suspension(statement, suspension)),
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => try_block
            .statements
            .iter()
            .chain(&catch_block.statements)
            .any(|statement| async_generator_contains_suspension(statement, suspension)),
        StatementIr::TryFinally {
            try_block,
            finally_block,
            ..
        } => try_block
            .statements
            .iter()
            .chain(&finally_block.statements)
            .any(|statement| async_generator_contains_suspension(statement, suspension)),
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => try_block
            .statements
            .iter()
            .chain(&catch_block.statements)
            .chain(&finally_block.statements)
            .any(|statement| async_generator_contains_suspension(statement, suspension)),
        _ => false,
    }
}

fn async_generator_dispatcher_unsupported_feature(statement: &StatementIr) -> Option<&'static str> {
    match statement {
        StatementIr::ModuleUnitOnce { .. } => Some("module unit evaluation"),
        StatementIr::Empty
        | StatementIr::Lexical { .. }
        | StatementIr::AnnexBFunctionCopy { .. }
        | StatementIr::Var(_)
        | StatementIr::Expression(_)
        | StatementIr::Debugger
        | StatementIr::Throw(_)
        | StatementIr::Return(_) => None,
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => statements
            .iter()
            .find_map(async_generator_dispatcher_unsupported_feature),
        StatementIr::Block(block) => block
            .statements
            .iter()
            .find_map(async_generator_dispatcher_unsupported_feature),
        StatementIr::GeneratorYield {
            resume_mode: GeneratorResumeModeIr::AssignProperty { .. },
            ..
        } => Some("property-assignment yield resumption"),
        StatementIr::GeneratorYield { .. } | StatementIr::AsyncAwait { .. } => None,
        StatementIr::GeneratorLoop {
            before_suspension,
            suspension_statement,
            after_suspension,
            entry_state,
            resume_state,
            exit_state,
            ..
        } => {
            let (suspend_state, suspension_resume_state) = match suspension_statement.as_ref() {
                StatementIr::GeneratorYield {
                    delegate: false,
                    suspend_state,
                    resume_state,
                    ..
                }
                | StatementIr::AsyncAwait {
                    suspend_state,
                    resume_state,
                    ..
                } => (suspend_state, resume_state),
                StatementIr::GeneratorYield { delegate: true, .. } => {
                    return Some("resumable loops with delegated yield");
                }
                _ => return Some("resumable loops without one direct suspension"),
            };
            if suspend_state != entry_state || suspension_resume_state != resume_state {
                return Some("resumable loops with non-linear suspension states");
            };
            if exit_state != resume_state {
                return Some("resumable loops with an unplanned exit state");
            }
            if before_suspension
                .iter()
                .chain(after_suspension)
                .any(|statement| {
                    async_generator_contains_suspension(statement, AsyncGeneratorSuspension::Await)
                        || async_generator_contains_suspension(
                            statement,
                            AsyncGeneratorSuspension::Yield,
                        )
                })
            {
                return Some("resumable loops containing multiple suspensions");
            }
            std::iter::once(suspension_statement.as_ref())
                .chain(before_suspension)
                .chain(after_suspension)
                .find_map(async_generator_dispatcher_unsupported_feature)
        }
        StatementIr::GeneratorIf {
            then_before_yield,
            then_yield_statement,
            then_after_yield,
            else_before_yield,
            else_yield_statement,
            else_after_yield,
            ..
        } => {
            let surrounding_statements = then_before_yield
                .iter()
                .chain(then_after_yield)
                .chain(else_before_yield)
                .chain(else_after_yield);
            if surrounding_statements.clone().any(|statement| {
                async_generator_contains_suspension(statement, AsyncGeneratorSuspension::Await)
                    || async_generator_contains_suspension(
                        statement,
                        AsyncGeneratorSuspension::Yield,
                    )
            }) {
                return Some("resumable branches containing multiple suspensions");
            }
            surrounding_statements
                .chain(then_yield_statement.as_deref())
                .chain(else_yield_statement.as_deref())
                .find_map(async_generator_dispatcher_unsupported_feature)
        }
        StatementIr::ForOfIterator {
            name,
            body,
            async_plan: Some(_),
            ..
        } if async_generator_for_await_is_transparent_yield(name, body) => None,
        // A for-await loop owns four states of its own (`entry`,
        // `value_resume`, `close_resume`, `exit`) and re-enters at whichever of
        // them the activation carries. That replay is sound as long as the loop
        // is the only thing suspending: a suspension in the body would need a
        // back edge from its resume state to the loop's entry state, which the
        // linear state chain does not have, and resuming at a body state would
        // fail the loop's entry test and skip the loop entirely. So a
        // suspension-free body compiles like any ordinary loop body, and a
        // suspending one is still refused rather than miscompiled.
        StatementIr::ForOfArray {
            body,
            async_plan: Some(_),
            ..
        } => {
            // `compile_async_for_of_array` still gates the loop on the three
            // states the loop itself owns, so a body suspension would resume
            // outside that test and skip the loop entirely.
            if async_generator_contains_suspension(body, AsyncGeneratorSuspension::Await)
                || async_generator_contains_suspension(body, AsyncGeneratorSuspension::Yield)
            {
                return Some("for-await iteration with a suspension in the loop body");
            }
            None
        }
        StatementIr::ForOfIterator {
            body,
            lexical_environment,
            async_plan: Some(_),
            ..
        } => {
            // A nested `for await` allocates its own four states inside this
            // loop's span, so this loop's per-iteration gate would enter the
            // inner loop's head instead of the inner loop entering it.
            if async_generator_contains_suspension(body, AsyncGeneratorSuspension::Await) {
                return Some("for-await iteration with a nested for-await in the loop body");
            }
            if async_generator_contains_suspension(body, AsyncGeneratorSuspension::Yield)
                && (lexical_environment
                    .as_ref()
                    .and_then(|environment| environment.iteration_environment.as_ref())
                    .is_some()
                    || matches!(body.as_ref(), StatementIr::Block(block) if block.lexical_environment.is_some()))
            {
                return Some(
                    "for-await-of with a per-iteration lexical environment and a body suspension",
                );
            }
            None
        }
        StatementIr::If {
            then_branch,
            else_branch,
            ..
        } => {
            if async_generator_contains_suspension(statement, AsyncGeneratorSuspension::Await)
                || async_generator_contains_suspension(statement, AsyncGeneratorSuspension::Yield)
            {
                return Some("branches containing suspension");
            }
            std::iter::once(then_branch.as_ref())
                .chain(else_branch.as_deref())
                .find_map(async_generator_dispatcher_unsupported_feature)
        }
        StatementIr::While { .. }
        | StatementIr::DoWhile { .. }
        | StatementIr::For { .. }
        | StatementIr::ForOfArray { .. }
        | StatementIr::ForOfString { .. }
        | StatementIr::ForOfIterator { .. }
        | StatementIr::ForInArray { .. }
        | StatementIr::ForInString { .. }
        | StatementIr::ForInObject { .. } => Some("loops"),
        StatementIr::Switch { .. } => Some("switch statements"),
        StatementIr::Labelled { .. } => Some("labelled statements"),
        StatementIr::TryCatch {
            try_block,
            catch_block,
            async_plan: Some(_),
            ..
        } => try_block
            .statements
            .iter()
            .chain(&catch_block.statements)
            .find_map(async_generator_dispatcher_unsupported_feature),
        StatementIr::TryFinally {
            try_block,
            finally_block,
            async_plan: Some(_),
            ..
        } => try_block
            .statements
            .iter()
            .chain(&finally_block.statements)
            .find_map(async_generator_dispatcher_unsupported_feature),
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            async_plan: Some(_),
            ..
        } => try_block
            .statements
            .iter()
            .chain(&catch_block.statements)
            .chain(&finally_block.statements)
            .find_map(async_generator_dispatcher_unsupported_feature),
        StatementIr::TryCatch { .. }
        | StatementIr::TryFinally { .. }
        | StatementIr::TryCatchFinally { .. } => Some("try statements without a resume plan"),
        StatementIr::Break { .. } | StatementIr::Continue { .. } => {
            Some("loop control completions")
        }
    }
}

pub(crate) fn async_generator_for_await_is_transparent_yield(
    binding: &str,
    body: &StatementIr,
) -> bool {
    match body {
        StatementIr::GeneratorYield {
            value:
                TypedExpr {
                    expr: ExprIr::Identifier(yielded_binding),
                    ..
                },
            delegate: false,
            resume_mode: GeneratorResumeModeIr::Ignore,
            ..
        } => yielded_binding == binding,
        StatementIr::LexicalBlock(statements) => {
            matches!(statements.as_slice(), [statement]
                if async_generator_for_await_is_transparent_yield(binding, statement))
        }
        StatementIr::Block(block) => {
            matches!(block.statements.as_slice(), [statement]
                if async_generator_for_await_is_transparent_yield(binding, statement))
        }
        _ => false,
    }
}

/// Builtins whose real bodies must be emitted regardless of what the script
/// text references, because a previous emission pass proved they are
/// dynamically reachable (see `emit_script`).
#[derive(Default)]
struct ForcedBuiltins {
    standard: BTreeSet<StandardBuiltinId>,
    host: BTreeSet<HostBuiltinId>,
    number_pow_import: bool,
}

fn emit_script_with_forced_builtins(
    script: &ScriptIr,
    forced: &ForcedBuiltins,
) -> Result<(WasmArtifact, ForcedBuiltins), EmitError> {
    let uses_heap = true;
    let references_agent_host = script
        .host_builtins
        .iter()
        .chain(&forced.host)
        .any(|builtin| {
            matches!(
                builtin,
                HostBuiltinId::AgentStart
                    | HostBuiltinId::AgentBroadcast
                    | HostBuiltinId::AgentReceiveBroadcast
                    | HostBuiltinId::AgentReport
                    | HostBuiltinId::AgentGetReport
                    | HostBuiltinId::AgentSleep
                    | HostBuiltinId::AgentMonotonicNow
                    | HostBuiltinId::AgentLeaving
            )
        });
    let uses_shared_memory = references_agent_host
        || script_references_memory_atomics(script)
        || forced
            .standard
            .iter()
            .copied()
            .any(standard_builtin_uses_memory_atomics);
    let uses_atomics_wait_async =
        script_references_standard_builtin(script, StandardBuiltinId::AtomicsWaitAsync)
            || forced
                .standard
                .contains(&StandardBuiltinId::AtomicsWaitAsync);
    let mut compiled_host_builtins = script.host_builtins.clone();
    if compiled_host_builtins.contains(&HostBuiltinId::CreateHTMLDDA)
        && !compiled_host_builtins.contains(&HostBuiltinId::HTMLDDA)
    {
        compiled_host_builtins.push(HostBuiltinId::HTMLDDA);
    }
    for builtin in all_host_builtins() {
        if forced.host.contains(builtin) && !compiled_host_builtins.contains(builtin) {
            compiled_host_builtins.push(*builtin);
        }
    }
    let stubbed_host_builtins = all_host_builtins()
        .iter()
        .copied()
        .filter(|builtin| !compiled_host_builtins.contains(builtin))
        .collect::<Vec<_>>();
    let uses_host_print = compiled_host_builtins.contains(&HostBuiltinId::Print);
    let uses_agent_host = compiled_host_builtins.iter().any(|builtin| {
        matches!(
            builtin,
            HostBuiltinId::AgentStart
                | HostBuiltinId::AgentBroadcast
                | HostBuiltinId::AgentReceiveBroadcast
                | HostBuiltinId::AgentReport
                | HostBuiltinId::AgentGetReport
                | HostBuiltinId::AgentSleep
                | HostBuiltinId::AgentMonotonicNow
                | HostBuiltinId::AgentLeaving
        )
    });
    let uses_number_pow_import = forced.number_pow_import;
    let mut compiled_standard_builtins = Vec::new();
    let mut stubbed_standard_builtins = Vec::new();
    for builtin in StandardBuiltinId::all_functions() {
        if !forced.standard.contains(builtin) && should_stub_standard_builtin(script, *builtin) {
            stubbed_standard_builtins.push(*builtin);
        } else {
            compiled_standard_builtins.push(*builtin);
        }
    }
    let uses_wall_clock_millis = compiled_standard_builtins
        .iter()
        .any(|builtin| builtin.requires_wall_clock());
    let number_pow_import_function_index =
        uses_number_pow_import.then_some(1 + u32::from(uses_host_print));
    let wall_clock_millis_import_function_index = uses_wall_clock_millis
        .then_some(1 + u32::from(uses_host_print) + u32::from(uses_number_pow_import));
    let shared_memory_alloc_function_index = uses_shared_memory.then_some(
        1 + u32::from(uses_host_print)
            + u32::from(uses_number_pow_import)
            + u32::from(uses_wall_clock_millis),
    );
    let monotonic_clock_nanos_import_function_index =
        uses_atomics_wait_async.then(|| shared_memory_alloc_function_index.unwrap() + 1);
    let sleep_nanos_import_function_index =
        monotonic_clock_nanos_import_function_index.map(|index| index + 1);
    let agent_call_import_function_index = uses_agent_host.then_some(
        1 + u32::from(uses_host_print)
            + u32::from(uses_number_pow_import)
            + u32::from(uses_wall_clock_millis)
            + u32::from(uses_shared_memory)
            + 2 * u32::from(uses_atomics_wait_async),
    );
    let imported_function_count = 1
        + u32::from(uses_host_print)
        + u32::from(uses_number_pow_import)
        + u32::from(uses_wall_clock_millis)
        + u32::from(uses_shared_memory)
        + 2 * u32::from(uses_atomics_wait_async)
        + u32::from(uses_agent_host);
    let uses_json_stringify =
        compiled_standard_builtins.contains(&StandardBuiltinId::JsonStringify);
    // The Temporal calendar helpers are only *called* from the five types that
    // carry a [[Calendar]] slot; nothing else can reach them.
    let uses_temporal_calendar = compiled_standard_builtins.iter().any(|builtin| {
        let name = builtin.debug_name();
        name.contains("Temporal.PlainDate")
            || name.contains("Temporal.PlainYearMonth")
            || name.contains("Temporal.PlainMonthDay")
            || name.contains("Temporal.ZonedDateTime")
    });
    let runtime_bootstrap_plan =
        RuntimeBootstrapPlan::from_script(script, &compiled_standard_builtins);
    let has_shared_stub =
        !stubbed_standard_builtins.is_empty() || !stubbed_host_builtins.is_empty();
    let function_metas = FunctionMetaRegistry::new(
        build_function_metas(
            script.functions.as_slice(),
            &compiled_standard_builtins,
            &stubbed_standard_builtins,
            &compiled_host_builtins,
            &stubbed_host_builtins,
            imported_function_count,
        ),
        number_pow_import_function_index,
        wall_clock_millis_import_function_index,
        shared_memory_alloc_function_index,
        monotonic_clock_nanos_import_function_index,
        sleep_nanos_import_function_index,
        agent_call_import_function_index,
    );
    let emitted_standard_builtins = emitted_compiled_standard_builtins(&compiled_standard_builtins);
    let string_pool =
        StringPool::collect(script, function_metas.metas(), &compiled_standard_builtins);
    let uses_function_table = true;
    let callable_function_count = script.functions.len()
        + emitted_standard_builtins.len()
        + usize::from(has_shared_stub)
        + compiled_host_builtins.len();
    let heap_alloc_function_index =
        uses_heap.then_some(imported_function_count + 1 + callable_function_count as u32);
    let object_append_data_property_function_index =
        heap_alloc_function_index.map(|heap_alloc_function_index| heap_alloc_function_index + 1);
    let object_append_accessor_property_function_index = object_append_data_property_function_index
        .map(|append_function_index| append_function_index + 1);
    let function_object_alloc_function_index = object_append_accessor_property_function_index
        .map(|append_function_index| append_function_index + 1);
    let plain_object_alloc_function_index = function_object_alloc_function_index
        .map(|function_object_alloc_function_index| function_object_alloc_function_index + 1);
    let array_alloc_function_index = plain_object_alloc_function_index
        .map(|plain_object_alloc_function_index| plain_object_alloc_function_index + 1);
    let mut main_builder = FunctionBuilder::new_main(
        script,
        &string_pool,
        &function_metas,
        uses_heap,
        runtime_bootstrap_plan.clone(),
        heap_alloc_function_index,
        object_append_data_property_function_index,
        object_append_accessor_property_function_index,
        function_object_alloc_function_index,
        plain_object_alloc_function_index,
        array_alloc_function_index,
    );
    let main_function = main_builder.compile()?;
    let mut compiled_functions = Vec::with_capacity(callable_function_count);
    for function in &script.functions {
        let mut builder = FunctionBuilder::new_function(
            function,
            &script.global_bindings,
            &string_pool,
            &function_metas,
            uses_heap,
            runtime_bootstrap_plan.clone(),
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        );
        compiled_functions.push(builder.compile()?);
    }
    for builtin in &emitted_standard_builtins {
        let mut builder = FunctionBuilder::new_standard_builtin(
            *builtin,
            &string_pool,
            &function_metas,
            uses_heap,
            false,
            runtime_bootstrap_plan.clone(),
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        );
        compiled_functions.push(builder.compile_builtin()?);
    }
    if has_shared_stub {
        let stub_builtin = stubbed_standard_builtins
            .first()
            .copied()
            .unwrap_or(StandardBuiltinId::FunctionConstructor);
        let mut builder = FunctionBuilder::new_standard_builtin(
            stub_builtin,
            &string_pool,
            &function_metas,
            uses_heap,
            true,
            runtime_bootstrap_plan.clone(),
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        );
        compiled_functions.push(builder.compile_builtin()?);
    }
    for builtin in &compiled_host_builtins {
        let mut builder = FunctionBuilder::new_host_builtin(
            *builtin,
            &string_pool,
            &function_metas,
            uses_heap,
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        );
        compiled_functions.push(builder.compile_builtin()?);
    }

    // Shared object-read / object-write runtime helpers. These carry the large
    // property-access state machines that would otherwise be inlined at every
    // read/write site, blowing single functions past Cranelift's per-function
    // code-size limit. They are emitted once, directly after the heap helpers,
    // and are reached with plain `call`s (never through the funcref table).
    let object_read_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_object_read_helper()
        })
        .transpose()?;
    let object_write_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_object_write_helper()
        })
        .transpose()?;
    let object_define_data_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_object_define_data_helper()
        })
        .transpose()?;
    let proxy_call_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_proxy_call_helper()
        })
        .transpose()?;
    let proxy_construct_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_proxy_construct_helper()
        })
        .transpose()?;
    let string_equality_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_string_equality_helper()
        })
        .transpose()?;
    let number_to_string_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_number_to_string_helper()
        })
        .transpose()?;
    let string_to_number_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_string_to_number_helper()
        })
        .transpose()?;
    let value_to_string_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_value_to_string_helper()
        })
        .transpose()?;
    let value_to_number_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_value_to_number_helper()
        })
        .transpose()?;
    let value_to_numeric_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_value_to_numeric_helper()
        })
        .transpose()?;
    let object_get_prototype_of_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_object_get_prototype_of_helper()
        })
        .transpose()?;
    let object_is_extensible_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_object_is_extensible_helper()
        })
        .transpose()?;
    let object_read_proxy_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_object_read_proxy_helper()
        })
        .transpose()?;
    let regexp_matcher_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_regexp_matcher_helper()
        })
        .transpose()?;
    let function_call_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_function_call_helper()
        })
        .transpose()?;
    let dynamic_property_read_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_dynamic_property_read_helper()
        })
        .transpose()?;
    let ordinary_set_data_on_receiver_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_ordinary_set_data_on_receiver_helper()
        })
        .transpose()?;
    let ordinary_set_data_on_receiver_with_fallback_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_ordinary_set_data_on_receiver_with_fallback_helper()
        })
        .transpose()?;
    let array_write_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_array_write_helper()
        })
        .transpose()?;
    let ordinary_set_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_ordinary_set_helper(true)
        })
        .transpose()?;
    let ordinary_set_without_receiver_fallback_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_ordinary_set_helper(false)
        })
        .transpose()?;
    let decimal_to_binary64_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_decimal_to_binary64_helper()
        })
        .transpose()?;
    let bigint_arithmetic_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_bigint_arithmetic_helper()
        })
        .transpose()?;
    let json_stringify_value_helper_function = (uses_heap && uses_json_stringify)
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_json_stringify_value_helper()
        })
        .transpose()?;
    // Both Temporal calendar helpers are emitted whenever the heap is enabled,
    // with a stub body when no calendar-bearing Temporal builtin is compiled,
    // so their fixed function offsets never shift. Order matters: the probe
    // helper takes the lower index.
    let temporal_calendar_iso_date_probe_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_temporal_calendar_iso_date_probe_helper(uses_temporal_calendar)
        })
        .transpose()?;
    let temporal_calendar_identifier_helper_function = uses_heap
        .then(|| {
            let mut builder = FunctionBuilder::new_runtime_operation_helper(
                &string_pool,
                &function_metas,
                uses_heap,
                runtime_bootstrap_plan.clone(),
                heap_alloc_function_index,
                object_append_data_property_function_index,
                object_append_accessor_property_function_index,
                function_object_alloc_function_index,
                plain_object_alloc_function_index,
                array_alloc_function_index,
            );
            builder.compile_temporal_calendar_identifier_helper(uses_temporal_calendar)
        })
        .transpose()?;

    let mut types = TypeSection::new();
    types.ty().function([], [ValType::I64]);
    if uses_function_table {
        types.ty().function(
            function_param_types(),
            [ValType::I64, ValType::I64, ValType::I64, ValType::I64],
        );
    }
    types.ty().function([ValType::I64], [ValType::I64]);
    types.ty().function(
        [
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
        ],
        [],
    );
    types.ty().function(
        [
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
        ],
        [],
    );
    types.ty().function(
        [
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
            ValType::I64,
        ],
        [ValType::I64],
    );
    types
        .ty()
        .function([ValType::I64, ValType::I64], [ValType::I64]);
    types
        .ty()
        .function([ValType::I64], [ValType::I64, ValType::I64]);
    types.ty().function([ValType::I32, ValType::I32], []);
    types
        .ty()
        .function([ValType::F64, ValType::F64], [ValType::F64]);
    types.ty().function([], [ValType::I32]);
    types.ty().function([], [ValType::I64]);
    types.ty().function([ValType::I64], []);
    types
        .ty()
        .function([ValType::I64, ValType::I64, ValType::I64], [ValType::I64]);
    types.ty().function([], [ValType::F64]);

    let main_wasm_index = imported_function_count;

    let mut functions = FunctionSection::new();
    functions.function(0);
    for _ in 0..callable_function_count {
        functions.function(JS_FUNCTION_TYPE_INDEX);
    }
    if uses_heap {
        functions.function(HEAP_ALLOC_TYPE_INDEX);
        functions.function(OBJECT_APPEND_DATA_PROPERTY_TYPE_INDEX);
        functions.function(OBJECT_APPEND_ACCESSOR_PROPERTY_TYPE_INDEX);
        functions.function(FUNCTION_OBJECT_ALLOC_TYPE_INDEX);
        functions.function(PLAIN_OBJECT_ALLOC_TYPE_INDEX);
        functions.function(ARRAY_ALLOC_TYPE_INDEX);
        // object-read + object-write runtime helpers share the JS function type.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // object-define-data helper: seven i64 params, no results.
        functions.function(OBJECT_APPEND_ACCESSOR_PROPERTY_TYPE_INDEX);
        // proxy call + construct dispatch helpers share the JS function type.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // string-payload-equality helper (also the JS function type).
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // number-to-string + string-to-number conversion helpers.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // dynamic ToString (value-to-string) helper.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // dynamic ToNumber (value-to-number) helper.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // dynamic ToNumeric (value-to-numeric) helper.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // proxy-aware [[GetPrototypeOf]] + [[IsExtensible]] helpers.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // proxy-aware [[Get]] helper.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // sequence-only RegExp matcher helper.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // plain function-call dispatcher helper.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // runtime-kind dynamic property-read helper.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // receiver-side OrdinarySet data-property helper.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // receiver-side OrdinarySet helper with generic write fallback.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // dense/sparse Array element-write helper.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // OrdinarySet helper with an explicit receiver.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // OrdinarySet helper without generic receiver write fallback.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // Exact decimal source text to binary64 conversion helper.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // Arbitrary-precision BigInt arithmetic helper.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // Temporal ParseTemporalCalendarString ISO-date probe helper.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // Temporal ToTemporalCalendarIdentifier string-resolution helper.
        // Both carry stub bodies unless a calendar-bearing Temporal builtin is
        // compiled, so their fixed offsets never shift.
        functions.function(JS_FUNCTION_TYPE_INDEX);
        // JSON.stringify value helper (only when JSON.stringify is compiled).
        if uses_json_stringify {
            functions.function(JS_FUNCTION_TYPE_INDEX);
        }
    }

    let mut exports = ExportSection::new();
    exports.export("main", ExportKind::Func, main_wasm_index);
    exports.export(
        RESULT_TAG_EXPORT,
        ExportKind::Global,
        RESULT_TAG_GLOBAL_INDEX,
    );
    exports.export(
        COMPLETION_KIND_EXPORT,
        ExportKind::Global,
        COMPLETION_KIND_GLOBAL_INDEX,
    );
    exports.export(
        COMPLETION_AUX_EXPORT,
        ExportKind::Global,
        COMPLETION_AUX_GLOBAL_INDEX,
    );
    exports.export(
        THROW_ERROR_NAME_EXPORT,
        ExportKind::Global,
        throw_error_name_global_index(uses_heap),
    );

    let mut globals = GlobalSection::new();
    globals.global(
        GlobalType {
            val_type: ValType::I32,
            mutable: true,
            shared: false,
        },
        &ConstExpr::i32_const(ValueKind::Undefined.tag()),
    );
    globals.global(
        GlobalType {
            val_type: ValType::I32,
            mutable: true,
            shared: false,
        },
        &ConstExpr::i32_const(COMPLETION_KIND_NORMAL as i32),
    );
    globals.global(
        GlobalType {
            val_type: ValType::I32,
            mutable: true,
            shared: false,
        },
        &ConstExpr::i32_const(0),
    );
    if uses_heap {
        globals.global(
            GlobalType {
                val_type: ValType::I64,
                mutable: true,
                shared: false,
            },
            &ConstExpr::i64_const(align_heap_start(string_pool.bytes.len()) as i64),
        );
        globals.global(
            GlobalType {
                val_type: ValType::I64,
                mutable: true,
                shared: false,
            },
            &ConstExpr::i64_const(0),
        );
        for _ in 0..59 {
            globals.global(
                GlobalType {
                    val_type: ValType::I64,
                    mutable: true,
                    shared: false,
                },
                &ConstExpr::i64_const(0),
            );
        }
    }
    globals.global(
        GlobalType {
            val_type: ValType::I64,
            mutable: true,
            shared: false,
        },
        &ConstExpr::i64_const(0),
    );
    if uses_heap {
        for _ in THROW_ERROR_NAME_HEAP_GLOBAL_INDEX + 1..GLOBAL_INDEX_REGISTRY.len() as u32 {
            globals.global(
                GlobalType {
                    val_type: ValType::I64,
                    mutable: true,
                    shared: false,
                },
                &ConstExpr::i64_const(0),
            );
        }
    }
    for _ in &string_pool.template_objects {
        globals.global(
            GlobalType {
                val_type: ValType::I64,
                mutable: true,
                shared: false,
            },
            &ConstExpr::i64_const(0),
        );
    }
    // One "already evaluated" guard per module unit, immediately after the
    // template-object globals so no existing index moves. Zero means "not yet
    // evaluated"; `FunctionBuilder::emit_module_unit_once` sets it.
    for _ in 0..module_unit_guard_count(script) {
        globals.global(
            GlobalType {
                val_type: ValType::I32,
                mutable: true,
                shared: false,
            },
            &ConstExpr::i32_const(0),
        );
    }

    let mut code = CodeSection::new();
    code.function(&main_function);
    for function in &compiled_functions {
        code.function(function);
    }
    if uses_heap {
        code.function(&emit_heap_alloc_helper_function());
        code.function(&emit_object_append_data_property_helper_function(
            heap_alloc_function_index.expect("heap helper index must exist when heap is enabled"),
        ));
        code.function(&emit_object_append_accessor_property_helper_function(
            heap_alloc_function_index.expect("heap helper index must exist when heap is enabled"),
        ));
        code.function(&emit_function_object_alloc_helper_function(
            heap_alloc_function_index.expect("heap helper index must exist when heap is enabled"),
            object_append_data_property_function_index
                .expect("object append helper index must exist when heap is enabled"),
        ));
        code.function(&emit_plain_object_alloc_helper_function(
            heap_alloc_function_index.expect("heap helper index must exist when heap is enabled"),
        ));
        code.function(&emit_array_alloc_helper_function(
            heap_alloc_function_index.expect("heap helper index must exist when heap is enabled"),
        ));
        code.function(
            object_read_helper_function
                .as_ref()
                .expect("object-read helper must exist when heap is enabled"),
        );
        code.function(
            object_write_helper_function
                .as_ref()
                .expect("object-write helper must exist when heap is enabled"),
        );
        code.function(
            object_define_data_helper_function
                .as_ref()
                .expect("object-define-data helper must exist when heap is enabled"),
        );
        code.function(
            proxy_call_helper_function
                .as_ref()
                .expect("proxy-call helper must exist when heap is enabled"),
        );
        code.function(
            proxy_construct_helper_function
                .as_ref()
                .expect("proxy-construct helper must exist when heap is enabled"),
        );
        code.function(
            string_equality_helper_function
                .as_ref()
                .expect("string-equality helper must exist when heap is enabled"),
        );
        code.function(
            number_to_string_helper_function
                .as_ref()
                .expect("number-to-string helper must exist when heap is enabled"),
        );
        code.function(
            string_to_number_helper_function
                .as_ref()
                .expect("string-to-number helper must exist when heap is enabled"),
        );
        code.function(
            value_to_string_helper_function
                .as_ref()
                .expect("value-to-string helper must exist when heap is enabled"),
        );
        code.function(
            value_to_number_helper_function
                .as_ref()
                .expect("value-to-number helper must exist when heap is enabled"),
        );
        code.function(
            value_to_numeric_helper_function
                .as_ref()
                .expect("value-to-numeric helper must exist when heap is enabled"),
        );
        code.function(
            object_get_prototype_of_helper_function
                .as_ref()
                .expect("get-prototype-of helper must exist when heap is enabled"),
        );
        code.function(
            object_is_extensible_helper_function
                .as_ref()
                .expect("is-extensible helper must exist when heap is enabled"),
        );
        code.function(
            object_read_proxy_helper_function
                .as_ref()
                .expect("object-read-proxy helper must exist when heap is enabled"),
        );
        code.function(
            regexp_matcher_helper_function
                .as_ref()
                .expect("regexp matcher helper must exist when heap is enabled"),
        );
        code.function(
            function_call_helper_function
                .as_ref()
                .expect("function-call helper must exist when heap is enabled"),
        );
        code.function(
            dynamic_property_read_helper_function
                .as_ref()
                .expect("dynamic property-read helper must exist when heap is enabled"),
        );
        code.function(
            ordinary_set_data_on_receiver_helper_function
                .as_ref()
                .expect("ordinary receiver-set helper must exist when heap is enabled"),
        );
        code.function(
            ordinary_set_data_on_receiver_with_fallback_helper_function
                .as_ref()
                .expect("ordinary receiver-set fallback helper must exist when heap is enabled"),
        );
        code.function(
            array_write_helper_function
                .as_ref()
                .expect("array-write helper must exist when heap is enabled"),
        );
        code.function(
            ordinary_set_helper_function
                .as_ref()
                .expect("ordinary-set helper must exist when heap is enabled"),
        );
        code.function(
            ordinary_set_without_receiver_fallback_helper_function
                .as_ref()
                .expect("ordinary-set no-fallback helper must exist when heap is enabled"),
        );
        code.function(
            decimal_to_binary64_helper_function
                .as_ref()
                .expect("decimal converter helper must exist when heap is enabled"),
        );
        code.function(
            bigint_arithmetic_helper_function
                .as_ref()
                .expect("BigInt arithmetic helper must exist when heap is enabled"),
        );
        code.function(
            temporal_calendar_iso_date_probe_helper_function
                .as_ref()
                .expect("temporal calendar date-probe helper must exist when heap is enabled"),
        );
        code.function(
            temporal_calendar_identifier_helper_function
                .as_ref()
                .expect("temporal calendar identifier helper must exist when heap is enabled"),
        );
        if let Some(json_stringify_value_helper_function) =
            json_stringify_value_helper_function.as_ref()
        {
            code.function(json_stringify_value_helper_function);
        }
    }

    let mut module = Module::new();
    module.section(&types);
    {
        let mut imports = ImportSection::new();
        imports.import(
            HOST_IMPORT_MODULE,
            HOST_IMPORT_AGENT_CAN_SUSPEND,
            wasm_encoder::EntityType::Function(HOST_AGENT_CAN_SUSPEND_IMPORT_TYPE_INDEX),
        );
        if uses_host_print {
            imports.import(
                HOST_IMPORT_MODULE,
                HOST_IMPORT_PRINT_LINE_UTF8,
                wasm_encoder::EntityType::Function(HOST_PRINT_IMPORT_TYPE_INDEX),
            );
        }
        if uses_number_pow_import {
            imports.import(
                HOST_IMPORT_MODULE,
                HOST_IMPORT_NUMBER_POW,
                wasm_encoder::EntityType::Function(HOST_NUMBER_POW_IMPORT_TYPE_INDEX),
            );
        }
        if uses_wall_clock_millis {
            imports.import(
                HOST_IMPORT_MODULE,
                HOST_IMPORT_WALL_CLOCK_MILLIS,
                wasm_encoder::EntityType::Function(HOST_WALL_CLOCK_MILLIS_IMPORT_TYPE_INDEX),
            );
        }
        if uses_shared_memory {
            imports.import(
                HOST_IMPORT_MODULE,
                HOST_IMPORT_SHARED_MEMORY_ALLOC,
                wasm_encoder::EntityType::Function(HEAP_ALLOC_TYPE_INDEX),
            );
            if uses_atomics_wait_async {
                imports.import(
                    HOST_IMPORT_MODULE,
                    HOST_IMPORT_MONOTONIC_CLOCK_NANOS,
                    wasm_encoder::EntityType::Function(
                        HOST_MONOTONIC_CLOCK_NANOS_IMPORT_TYPE_INDEX,
                    ),
                );
                imports.import(
                    HOST_IMPORT_MODULE,
                    HOST_IMPORT_SLEEP_NANOS,
                    wasm_encoder::EntityType::Function(HOST_SLEEP_NANOS_IMPORT_TYPE_INDEX),
                );
            }
            let initial_pages = initial_memory_pages(string_pool.bytes.len(), uses_heap);
            imports.import(
                HOST_IMPORT_MODULE,
                HOST_IMPORT_PRIVATE_MEMORY,
                wasm_encoder::EntityType::Memory(MemoryType {
                    minimum: initial_pages,
                    maximum: None,
                    memory64: false,
                    shared: false,
                    page_size_log2: None,
                }),
            );
            imports.import(
                HOST_IMPORT_MODULE,
                HOST_IMPORT_SHARED_MEMORY,
                wasm_encoder::EntityType::Memory(MemoryType {
                    minimum: 1,
                    maximum: Some(16_384),
                    memory64: false,
                    shared: true,
                    page_size_log2: None,
                }),
            );
        }
        if uses_agent_host {
            imports.import(
                HOST_IMPORT_MODULE,
                HOST_IMPORT_AGENT_CALL,
                wasm_encoder::EntityType::Function(HOST_AGENT_CALL_IMPORT_TYPE_INDEX),
            );
        }
        module.section(&imports);
    }
    module.section(&functions);
    if uses_function_table {
        let mut tables = TableSection::new();
        tables.table(TableType {
            element_type: RefType::FUNCREF,
            minimum: callable_function_count as u64,
            maximum: Some(callable_function_count as u64),
            table64: false,
            shared: false,
        });
        module.section(&tables);
    }

    let mut debug_dump = vec![
        "module: js-aot".to_string(),
        "export func: main -> i64".to_string(),
        format!("static result kind: {}", script.result_kind().as_str()),
        format!("locals: {}", main_builder.emitted_local_count()),
        format!("internal functions: {}", callable_function_count),
        format!(
            "runtime helper functions: {}",
            if uses_heap {
                27 + usize::from(uses_json_stringify)
            } else {
                0
            }
        ),
        format!(
            "standard builtin bodies: {} real, {} shared-stubbed",
            emitted_standard_builtins.len(),
            stubbed_standard_builtins.len()
        ),
        format!(
            "runtime bootstrap: {} standard roots, full globals={}",
            runtime_bootstrap_plan.standard_roots.len(),
            runtime_bootstrap_plan.full_standard_globals
        ),
        format!(
            "standard builtin real names: {}",
            emitted_standard_builtins
                .iter()
                .map(|builtin| builtin.debug_name())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        format!(
            "host builtin bodies: {} real, {} shared-stubbed",
            compiled_host_builtins.len(),
            stubbed_host_builtins.len()
        ),
        format!(
            "host builtin real names: {}",
            compiled_host_builtins
                .iter()
                .map(|builtin| builtin.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        format!("global registry slots: {}", GLOBAL_INDEX_REGISTRY.len()),
        format!("completion kind slots: {}", COMPLETION_KIND_REGISTRY.len()),
        format!("export global: {RESULT_TAG_EXPORT}"),
        format!("export global: {COMPLETION_KIND_EXPORT}"),
        format!("export global: {COMPLETION_AUX_EXPORT}"),
        format!("export global: {THROW_ERROR_NAME_EXPORT}"),
        format!("import func: {HOST_IMPORT_MODULE}.{HOST_IMPORT_AGENT_CAN_SUSPEND}"),
    ];
    if uses_host_print {
        debug_dump.push(format!(
            "import func: {HOST_IMPORT_MODULE}.{HOST_IMPORT_PRINT_LINE_UTF8}"
        ));
    }
    if uses_number_pow_import {
        debug_dump.push(format!(
            "import func: {HOST_IMPORT_MODULE}.{HOST_IMPORT_NUMBER_POW}"
        ));
    }
    if uses_wall_clock_millis {
        debug_dump.push(format!(
            "import func: {HOST_IMPORT_MODULE}.{HOST_IMPORT_WALL_CLOCK_MILLIS}"
        ));
    }
    if uses_shared_memory {
        debug_dump.push(format!(
            "import func: {HOST_IMPORT_MODULE}.{HOST_IMPORT_SHARED_MEMORY_ALLOC}"
        ));
        debug_dump.push(format!(
            "import memory: {HOST_IMPORT_MODULE}.{HOST_IMPORT_PRIVATE_MEMORY}"
        ));
        debug_dump.push(format!(
            "import memory: {HOST_IMPORT_MODULE}.{HOST_IMPORT_SHARED_MEMORY}"
        ));
    }
    if uses_atomics_wait_async {
        debug_dump.push(format!(
            "import func: {HOST_IMPORT_MODULE}.{HOST_IMPORT_MONOTONIC_CLOCK_NANOS}"
        ));
        debug_dump.push(format!(
            "import func: {HOST_IMPORT_MODULE}.{HOST_IMPORT_SLEEP_NANOS}"
        ));
    }
    if uses_agent_host {
        debug_dump.push(format!(
            "import func: {HOST_IMPORT_MODULE}.{HOST_IMPORT_AGENT_CALL}"
        ));
    }

    if !string_pool.bytes.is_empty() || uses_heap {
        if !uses_shared_memory {
            let mut memories = MemorySection::new();
            memories.memory(MemoryType {
                minimum: initial_memory_pages(string_pool.bytes.len(), uses_heap),
                maximum: None,
                memory64: false,
                shared: false,
                page_size_log2: None,
            });
            module.section(&memories);
        }
        exports.export("memory", ExportKind::Memory, 0);
        if uses_shared_memory {
            debug_dump.push("memory: exported private linear memory".to_string());
        } else {
            debug_dump.push("memory: exported linear memory".to_string());
        }

        let mut data = DataSection::new();
        data.active(
            0,
            &ConstExpr::i32_const(STATIC_DATA_OFFSET as i32),
            string_pool.bytes.iter().copied(),
        );
        module.section(&globals);
        module.section(&exports);
        if uses_function_table {
            let mut elements = ElementSection::new();
            let first_callable_wasm_index = imported_function_count + 1;
            let function_indexes = (first_callable_wasm_index
                ..first_callable_wasm_index + callable_function_count as u32)
                .collect::<Vec<_>>();
            elements.active(
                Some(0),
                &ConstExpr::i32_const(0),
                Elements::Functions(Cow::Owned(function_indexes)),
            );
            module.section(&elements);
        }
        module.section(&code);
        if !string_pool.bytes.is_empty() {
            module.section(&data);
            debug_dump.push("data segments: 1".to_string());
        } else {
            debug_dump.push("data segments: 0".to_string());
        }
        if uses_heap {
            debug_dump.push("heap: enabled".to_string());
        }
    } else {
        module.section(&globals);
        module.section(&exports);
        if uses_function_table {
            let mut elements = ElementSection::new();
            let first_callable_wasm_index = imported_function_count + 1;
            let function_indexes = (first_callable_wasm_index
                ..first_callable_wasm_index + callable_function_count as u32)
                .collect::<Vec<_>>();
            elements.active(
                Some(0),
                &ConstExpr::i32_const(0),
                Elements::Functions(Cow::Owned(function_indexes)),
            );
            module.section(&elements);
        }
        module.section(&code);
        debug_dump.push("memory: none".to_string());
        debug_dump.push("data segments: 0".to_string());
    }

    // Builtins codegen proved reachable (materialized or direct-called) while
    // their body was stubbed this pass: the caller force-compiles these and
    // re-emits (see `emit_script`).
    let touched_stubbed = ForcedBuiltins {
        standard: function_metas
            .touched_standard_builtins()
            .into_iter()
            .filter(|builtin| stubbed_standard_builtins.contains(builtin))
            .collect(),
        host: function_metas
            .touched_host_builtins()
            .into_iter()
            .filter(|builtin| stubbed_host_builtins.contains(builtin))
            .collect(),
        number_pow_import: function_metas.touched_number_pow_import() && !uses_number_pow_import,
    };

    Ok((
        WasmArtifact {
            bytes: module.finish(),
            invariant_note: "direct-js-to-wasm module",
            debug_dump: debug_dump.join("\n"),
        },
        touched_stubbed,
    ))
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn template_object_global_index(&self, site_id: u64) -> u32 {
        let site_offset = self
            .strings
            .template_objects
            .keys()
            .position(|candidate| *candidate == site_id)
            .expect("template object site must be collected") as u32;
        GLOBAL_INDEX_REGISTRY.len() as u32 + site_offset
    }

    fn new_main(
        script: &'a ScriptIr,
        strings: &'a StringPool,
        functions: &'a FunctionMetaRegistry,
        uses_heap: bool,
        runtime_bootstrap_plan: RuntimeBootstrapPlan,
        heap_alloc_function_index: Option<u32>,
        object_append_data_property_function_index: Option<u32>,
        object_append_accessor_property_function_index: Option<u32>,
        function_object_alloc_function_index: Option<u32>,
        plain_object_alloc_function_index: Option<u32>,
        array_alloc_function_index: Option<u32>,
    ) -> Self {
        Self::new(
            &script.body,
            &[],
            script.owned_env_bindings.as_slice(),
            &[],
            strings,
            functions,
            None,
            FunctionFlavor::Ordinary,
            None,
            script.strict,
            None,
            script
                .global_bindings
                .iter()
                .map(|binding| (binding.name.clone(), binding.kind))
                .collect(),
            uses_heap,
            ReturnAbi::MainExport,
            false,
            runtime_bootstrap_plan,
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        )
    }

    fn new_function(
        function: &'a FunctionIr,
        global_bindings: &'a [ScriptGlobalBindingIr],
        strings: &'a StringPool,
        functions: &'a FunctionMetaRegistry,
        uses_heap: bool,
        runtime_bootstrap_plan: RuntimeBootstrapPlan,
        heap_alloc_function_index: Option<u32>,
        object_append_data_property_function_index: Option<u32>,
        object_append_accessor_property_function_index: Option<u32>,
        function_object_alloc_function_index: Option<u32>,
        plain_object_alloc_function_index: Option<u32>,
        array_alloc_function_index: Option<u32>,
    ) -> Self {
        Self::new(
            &function.body,
            function.params.as_slice(),
            function.owned_env_bindings.as_slice(),
            function.captured_bindings.as_slice(),
            strings,
            functions,
            Some(function.id.clone()),
            function.flavor,
            function.lexical_derived_activation.as_ref(),
            function.strict,
            function.is_named_expression.then(|| function.name.clone()),
            global_bindings
                .iter()
                .map(|binding| (binding.name.clone(), binding.kind))
                .collect(),
            uses_heap,
            ReturnAbi::MultiValue,
            function.is_derived_constructor,
            runtime_bootstrap_plan,
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        )
    }

    fn new_host_builtin(
        builtin: HostBuiltinId,
        strings: &'a StringPool,
        functions: &'a FunctionMetaRegistry,
        uses_heap: bool,
        heap_alloc_function_index: Option<u32>,
        object_append_data_property_function_index: Option<u32>,
        object_append_accessor_property_function_index: Option<u32>,
        function_object_alloc_function_index: Option<u32>,
        plain_object_alloc_function_index: Option<u32>,
        array_alloc_function_index: Option<u32>,
    ) -> Self {
        let function_id = builtin.function_id();
        Self::new(
            &EMPTY_BLOCK,
            &[],
            &[],
            &[],
            strings,
            functions,
            Some(function_id),
            FunctionFlavor::Ordinary,
            None,
            true,
            None,
            BTreeMap::new(),
            uses_heap,
            ReturnAbi::MultiValue,
            false,
            RuntimeBootstrapPlan::default(),
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn new_runtime_operation_helper(
        strings: &'a StringPool,
        functions: &'a FunctionMetaRegistry,
        uses_heap: bool,
        runtime_bootstrap_plan: RuntimeBootstrapPlan,
        heap_alloc_function_index: Option<u32>,
        object_append_data_property_function_index: Option<u32>,
        object_append_accessor_property_function_index: Option<u32>,
        function_object_alloc_function_index: Option<u32>,
        plain_object_alloc_function_index: Option<u32>,
        array_alloc_function_index: Option<u32>,
    ) -> Self {
        Self::new(
            &EMPTY_BLOCK,
            &[],
            &[],
            &[],
            strings,
            functions,
            None,
            FunctionFlavor::Ordinary,
            None,
            true,
            None,
            BTreeMap::new(),
            uses_heap,
            ReturnAbi::MultiValue,
            false,
            runtime_bootstrap_plan,
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        )
    }

    fn new_standard_builtin(
        builtin: StandardBuiltinId,
        strings: &'a StringPool,
        functions: &'a FunctionMetaRegistry,
        uses_heap: bool,
        stub_body: bool,
        runtime_bootstrap_plan: RuntimeBootstrapPlan,
        heap_alloc_function_index: Option<u32>,
        object_append_data_property_function_index: Option<u32>,
        object_append_accessor_property_function_index: Option<u32>,
        function_object_alloc_function_index: Option<u32>,
        plain_object_alloc_function_index: Option<u32>,
        array_alloc_function_index: Option<u32>,
    ) -> Self {
        let mut builder = Self::new(
            &EMPTY_BLOCK,
            &[],
            &[],
            &[],
            strings,
            functions,
            Some(builtin.function_id()),
            FunctionFlavor::Ordinary,
            None,
            true,
            None,
            BTreeMap::new(),
            uses_heap,
            ReturnAbi::MultiValue,
            false,
            runtime_bootstrap_plan,
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
        );
        builder.stub_standard_builtin_body = stub_body;
        builder
    }

    fn new(
        body: &'a BlockIr,
        params: &'a [FunctionParamIr],
        owned_env_bindings: &'a [OwnedEnvBindingIr],
        captured_bindings: &'a [porffor_ir::CapturedBindingIr],
        strings: &'a StringPool,
        functions: &'a FunctionMetaRegistry,
        function_id: Option<FunctionId>,
        function_flavor: FunctionFlavor,
        lexical_derived_activation: Option<&'a DerivedConstructorActivationIr>,
        strict: bool,
        self_binding_name: Option<String>,
        script_global_bindings: BTreeMap<String, ScriptGlobalBindingKind>,
        uses_heap: bool,
        return_abi: ReturnAbi,
        is_derived_constructor: bool,
        runtime_bootstrap_plan: RuntimeBootstrapPlan,
        heap_alloc_function_index: Option<u32>,
        object_append_data_property_function_index: Option<u32>,
        object_append_accessor_property_function_index: Option<u32>,
        function_object_alloc_function_index: Option<u32>,
        plain_object_alloc_function_index: Option<u32>,
        array_alloc_function_index: Option<u32>,
    ) -> Self {
        let hoisted_vars = collect_hoisted_vars_block_root(body);
        let self_binding_local_count = usize::from(self_binding_name.is_some());
        let param_local_count = count_param_locals(return_abi) as u32;
        let needs_arguments_binding_locals = matches!(return_abi, ReturnAbi::MultiValue)
            && function_flavor == FunctionFlavor::Ordinary;
        let captured_arguments_local_count = if captured_bindings
            .iter()
            .any(|binding| binding.name == LEXICAL_ARGUMENTS_NAME)
        {
            2
        } else {
            0
        };
        let total_binding_local_count = (count_block_lexicals(body)
            + self_binding_local_count
            + count_param_binding_locals(params, owned_env_bindings)
            + if needs_arguments_binding_locals { 2 } else { 0 }
            + captured_arguments_local_count) as u32
            + (hoisted_vars.len() as u32 * 2);
        let temp_local_count = count_block_temp_locals(body).max(2048) as u32;
        let current_env_local = param_local_count + total_binding_local_count;
        let class_function_context_local = current_env_local + 5;
        let named_function_context_local = class_function_context_local + 1;
        let scratch_local = named_function_context_local + 1;
        Self {
            body,
            params,
            owned_env_bindings,
            captured_bindings,
            strings,
            functions,
            function_id,
            function_flavor,
            lexical_derived_activation,
            is_derived_constructor,
            strict,
            self_binding_name,
            script_global_bindings,
            uses_heap,
            return_abi,
            hoisted_vars,
            binding_scopes: Vec::new(),
            next_binding_local: param_local_count,
            total_binding_local_count,
            temp_local_count,
            current_env_local,
            class_function_context_local,
            active_private_environment_locals: Vec::new(),
            named_function_context_local,
            result_local: current_env_local + 1,
            result_tag_local: current_env_local + 2,
            completion_local: current_env_local + 3,
            completion_aux_local: current_env_local + 4,
            scratch_local,
            temp_local_base: scratch_local + 1,
            temp_stack_depth: 0,
            max_temp_stack_depth: 0,
            environment_depth: 0,
            this_payload_local: matches!(return_abi, ReturnAbi::MultiValue).then_some(1),
            this_tag_local: matches!(return_abi, ReturnAbi::MultiValue).then_some(2),
            control_stack: Vec::new(),
            breakable_stack: Vec::new(),
            loop_stack: Vec::new(),
            label_stack: Vec::new(),
            throw_handler_stack: Vec::new(),
            finally_stack: Vec::new(),
            generator_finalizer_depth: 0,
            stub_standard_builtin_body: false,
            runtime_bootstrap_plan,
            heap_alloc_function_index,
            object_append_data_property_function_index,
            object_append_accessor_property_function_index,
            function_object_alloc_function_index,
            plain_object_alloc_function_index,
            array_alloc_function_index,
            outline_object_read: true,
            outline_object_write: true,
            outline_object_define_data: true,
            outline_function_call: true,
            outline_proxy_call: true,
            outline_proxy_construct: true,
            outline_string_equality: true,
            outline_number_to_string: true,
            outline_string_to_number: true,
            outline_value_to_string: true,
            outline_value_to_number: true,
            outline_value_to_numeric: true,
            outline_object_get_prototype_of: true,
            outline_object_is_extensible: true,
            outline_object_read_proxy: true,
            outline_array_write: true,
            ordinary_set_data_on_receiver_emission: OrdinarySetDataOnReceiverEmission::Inline,
            object_write_strict_flag_local: None,
        }
    }

    /// Wasm function index of the shared object-read runtime helper. It is
    /// emitted immediately after the six heap/object allocation helpers.
    pub(crate) fn object_read_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 6)
    }

    /// Wasm function index of the shared object-write runtime helper.
    pub(crate) fn object_write_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 7)
    }

    /// Wasm function index of the shared object-define-data runtime helper.
    pub(crate) fn object_define_data_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 8)
    }

    /// Wasm function index of the shared proxy-aware call-dispatch helper.
    pub(crate) fn proxy_call_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 9)
    }

    /// Wasm function index of the shared proxy-aware construct-dispatch helper.
    pub(crate) fn proxy_construct_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 10)
    }

    /// Wasm function index of the shared string-equality helper.
    pub(crate) fn string_equality_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 11)
    }

    /// Wasm function index of the shared number-to-string helper.
    pub(crate) fn number_to_string_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 12)
    }

    /// Wasm function index of the shared string-to-number helper.
    pub(crate) fn string_to_number_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 13)
    }

    /// Wasm function index of the shared dynamic ToString (value-to-string)
    /// helper.
    pub(crate) fn value_to_string_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 14)
    }

    /// Wasm function index of the shared dynamic ToNumber (value-to-number)
    /// helper. Unconditional (like value-to-string) so its fixed offset never
    /// shifts.
    pub(crate) fn value_to_number_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 15)
    }

    /// Wasm function index of the shared dynamic ToNumeric helper.
    pub(crate) fn value_to_numeric_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 16)
    }

    /// Wasm function index of the shared proxy-aware `[[GetPrototypeOf]]` helper.
    /// Unconditional (emitted whenever heap is used) so its fixed offset never
    /// shifts.
    pub(crate) fn object_get_prototype_of_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 17)
    }

    /// Wasm function index of the shared proxy-aware `[[IsExtensible]]` helper.
    /// Unconditional (emitted whenever heap is used) so its fixed offset never
    /// shifts.
    pub(crate) fn object_is_extensible_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 18)
    }

    /// Wasm function index of the shared proxy-aware `[[Get]]` helper.
    /// Unconditional (emitted whenever heap is used) so its fixed offset never
    /// shifts.
    pub(crate) fn object_read_proxy_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 19)
    }

    /// Wasm function index of the sequence-only RegExp matcher helper.
    /// Unconditional (emitted whenever heap is used) so its fixed offset never
    /// shifts.
    pub(crate) fn regexp_matcher_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 20)
    }

    /// Wasm function index of the shared plain function-call dispatcher.
    /// Unconditional (emitted whenever heap is used) so its fixed offset never
    /// shifts.
    pub(crate) fn function_call_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 21)
    }

    /// Wasm function index of the runtime-kind dynamic property-read helper.
    /// Unconditional (emitted whenever heap is used) so its fixed offset never
    /// shifts.
    pub(crate) fn dynamic_property_read_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 22)
    }

    /// Wasm function index of the receiver-side OrdinarySet data-property
    /// helper. Unconditional (emitted whenever heap is used) so its fixed
    /// offset never shifts.
    pub(crate) fn ordinary_set_data_on_receiver_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 23)
    }

    pub(crate) fn ordinary_set_data_on_receiver_with_fallback_helper_function_index(
        &self,
    ) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 24)
    }

    /// Wasm function index of the shared dense/sparse Array element-write helper.
    pub(crate) fn array_write_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 25)
    }

    /// Wasm function index of the shared OrdinarySet helper with an explicit receiver.
    pub(crate) fn ordinary_set_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 26)
    }

    pub(crate) fn ordinary_set_without_receiver_fallback_helper_function_index(
        &self,
    ) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 27)
    }

    /// Wasm function index of the exact decimal source-text to binary64 helper.
    pub(crate) fn decimal_to_binary64_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 28)
    }

    /// Wasm function index of the shared arbitrary-precision BigInt arithmetic
    /// helper.
    pub(crate) fn bigint_arithmetic_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 29)
    }

    /// Wasm function index of the Temporal ISO-date calendar-probe helper.
    /// Always emitted (stubbed when no calendar-bearing Temporal builtin is
    /// compiled) so its fixed offset never shifts.
    pub(crate) fn temporal_calendar_iso_date_probe_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 30)
    }

    /// Wasm function index of the shared `ToTemporalCalendarIdentifier`
    /// string-resolution helper. Always emitted (stubbed when unused).
    pub(crate) fn temporal_calendar_identifier_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 31)
    }

    /// Wasm function index of the shared JSON.stringify value helper. Emitted
    /// only when `JSON.stringify` is compiled, immediately after the last
    /// unconditional runtime helper, so its index never shifts the preceding
    /// fixed-offset helpers.
    pub(crate) fn json_stringify_value_helper_function_index(&self) -> Option<u32> {
        self.heap_alloc_function_index.map(|base| base + 32)
    }

    pub(crate) fn local_count(&self) -> usize {
        self.total_binding_local_count as usize + 8 + self.temp_local_count as usize
    }

    pub(crate) fn emitted_local_count(&self) -> u32 {
        self.total_binding_local_count + 8 + self.max_temp_stack_depth
    }

    pub(crate) const fn is_main(&self) -> bool {
        matches!(self.return_abi, ReturnAbi::MainExport)
    }

    pub(crate) fn is_script_global_binding(&self, name: &str) -> bool {
        self.script_global_bindings
            .get(name)
            .is_some_and(|kind| *kind != ScriptGlobalBindingKind::Intrinsic)
    }

    pub(crate) fn should_read_script_global_property(&self, name: &str) -> bool {
        !self.is_main()
            && name != LEXICAL_THIS_NAME
            && name != LEXICAL_ARGUMENTS_NAME
            && self.lookup_binding(name).is_none()
    }

    pub(crate) fn reserve_temp_local(&mut self) -> u32 {
        assert!(self.temp_stack_depth < self.temp_local_count);
        let local = self.temp_local_base + self.temp_stack_depth;
        self.temp_stack_depth += 1;
        self.max_temp_stack_depth = self.max_temp_stack_depth.max(self.temp_stack_depth);
        local
    }

    pub(crate) fn release_temp_local(&mut self, local: u32) {
        assert!(self.temp_stack_depth > 0);
        self.temp_stack_depth -= 1;
        let expected = self.temp_local_base + self.temp_stack_depth;
        assert_eq!(local, expected);
    }

    pub(crate) fn finish_function(&self, function: Function) -> Function {
        let planned_local_count = self.local_count() as u32;
        let emitted_local_count = self.emitted_local_count();
        if emitted_local_count == planned_local_count {
            return function;
        }

        let local_declaration =
            Function::new([(planned_local_count, ValType::I64)]).into_raw_body();
        let mut body_bytes = function.into_raw_body();
        assert!(
            body_bytes.starts_with(&local_declaration),
            "function local declaration does not match planned local count {planned_local_count}"
        );
        let instruction_bytes = body_bytes.split_off(local_declaration.len());
        let mut function = Function::new([(emitted_local_count, ValType::I64)]);
        function.raw(instruction_bytes);
        function
    }

    fn normalize_base_class_constructor_result(&mut self, function: &mut Function) {
        let is_base_class_constructor = self.current_function_meta().is_some_and(|meta| {
            meta.class_kind == ClassFunctionKind::Constructor && !meta.is_derived_constructor
        });
        if !is_base_class_constructor {
            return;
        }
        let (Some(this_payload_local), Some(this_tag_local)) =
            (self.this_payload_local, self.this_tag_local)
        else {
            return;
        };

        // Only a fall-through Normal completion is normalized here. Explicit
        // Return and Throw completions remain visible to OrdinaryConstruct,
        // which preserves object returns and selects the preallocated receiver
        // for primitive returns.
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(this_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
    }

    fn compile(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));

        self.push_scope();
        self.ensure_heap_ptr_after_static_data(&mut function);
        self.init_current_realm(&mut function)?;
        self.init_current_env(&mut function)?;
        self.init_runtime_roots(&mut function)?;
        self.init_script_global_object(&mut function)?;
        self.init_template_objects(&mut function)?;
        self.bind_captured_bindings(&mut function);
        let suspended_initialization =
            self.current_function_meta()
                .and_then(|meta| match meta.execution_kind {
                    FunctionExecutionKind::Generator => Some((
                        HEAP_GENERATOR_RESUME_STATE_OFFSET,
                        GENERATOR_RESUME_STATE_INITIALIZING,
                    )),
                    FunctionExecutionKind::AsyncGenerator => Some((
                        HEAP_ASYNC_GENERATOR_RESUME_STATE_OFFSET,
                        ASYNC_GENERATOR_RESUME_STATE_INITIALIZING,
                    )),
                    FunctionExecutionKind::Ordinary | FunctionExecutionKind::Async => None,
                });
        let resumable_initialized_offset = self.current_function_meta().and_then(|meta| match meta
            .execution_kind
        {
            FunctionExecutionKind::Generator => Some(HEAP_GENERATOR_INITIALIZED_OFFSET),
            FunctionExecutionKind::Async => Some(HEAP_ASYNC_INITIALIZED_OFFSET),
            FunctionExecutionKind::AsyncGenerator => Some(HEAP_ASYNC_GENERATOR_INITIALIZED_OFFSET),
            FunctionExecutionKind::Ordinary => None,
        });
        if let Some(initialized_offset) = resumable_initialized_offset {
            let activation_local = self
                .new_target_payload_local()
                .expect("resumable body must use the function call ABI");
            self.load_i64_to_local_from_offset(
                activation_local,
                initialized_offset,
                self.scratch_local,
                &mut function,
            );
            function.instruction(&Instruction::LocalGet(self.scratch_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
        }
        self.bind_self_function(&mut function)?;
        if let Some(constructor_meta) = self
            .current_function_meta()
            .filter(|meta| {
                meta.class_kind == ClassFunctionKind::Constructor && !meta.is_derived_constructor
            })
            .cloned()
        {
            let this_payload_local = self
                .this_payload_local
                .expect("base class constructor must receive a this payload");
            let this_tag_local = self
                .this_tag_local
                .expect("base class constructor must receive a this tag");
            self.emit_initialize_instance_elements(
                &constructor_meta,
                self.class_function_context_local,
                this_payload_local,
                this_tag_local,
                &mut function,
            )?;
        }
        self.bind_parameters(&mut function)?;
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        for name in self.hoisted_vars.clone() {
            let reuses_parameter_binding = self.params.iter().any(|param| param.name == name);
            let reuses_arguments_binding =
                name == LEXICAL_ARGUMENTS_NAME && self.function_flavor == FunctionFlavor::Ordinary;
            if reuses_parameter_binding || reuses_arguments_binding {
                continue;
            }
            let storage = if let Some(slot) = self.owned_env_slot(&name) {
                BindingStorage::EnvSlot { slot, hops: 0 }
            } else {
                let tag_local = self.next_binding_local;
                let payload_local = self.next_binding_local + 1;
                self.next_binding_local += 2;
                BindingStorage::Dynamic {
                    tag_local,
                    payload_local,
                }
            };
            self.binding_scopes
                .last_mut()
                .expect("binding scope stack must exist")
                .insert(name, storage);
            self.initialize_binding_undefined(storage, &mut function);
        }
        if let Some(initialized_offset) = resumable_initialized_offset {
            self.initialize_direct_lexical_bindings(&self.body.statements, &mut function);
            let activation_local = self
                .new_target_payload_local()
                .expect("resumable body must use the function call ABI");
            self.store_i64_const_at_offset(activation_local, initialized_offset, 1, &mut function);
            function.instruction(&Instruction::End);
        }
        if let Some((resume_state_offset, initializing_state)) = suspended_initialization {
            let activation_local = self
                .new_target_payload_local()
                .expect("suspended function body must use the function call ABI");
            self.load_i64_to_local_from_offset(
                activation_local,
                resume_state_offset,
                self.scratch_local,
                &mut function,
            );
            function.instruction(&Instruction::LocalGet(self.scratch_local));
            function.instruction(&Instruction::I64Const(initializing_state as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.set_completion_kind(CompletionKind::Normal, &mut function);
            self.emit_statement_result(&mut function, ValueKind::Undefined);
            self.emit_return_current_completion(&mut function);
            function.instruction(&Instruction::End);
        }
        if self
            .current_function_meta()
            .is_some_and(|meta| meta.is_synthetic_default_derived_constructor)
        {
            self.emit_super_construct_with_arg_vector(
                self.argc_param_local(),
                self.argv_param_local(),
                self.result_local,
                self.result_tag_local,
                &mut function,
            )?;
            self.emit_propagate_throw_from_locals_if_needed_with_extra_depth(
                self.result_local,
                self.result_tag_local,
                0,
                &mut function,
            )?;
        }
        self.compile_block_contents(self.body, &mut function)?;
        if matches!(self.return_abi, ReturnAbi::MultiValue)
            && !self
                .current_function_meta()
                .is_some_and(|meta| meta.class_kind == ClassFunctionKind::Constructor)
        {
            self.emit_statement_result(&mut function, ValueKind::Undefined);
        }
        self.normalize_base_class_constructor_result(&mut function);
        self.normalize_derived_constructor_result(&mut function)?;
        if self.is_main() && self.uses_heap {
            if self
                .functions
                .monotonic_clock_nanos_import_function_index()
                .is_some()
            {
                function.instruction(&Instruction::Block(BlockType::Empty));
                function.instruction(&Instruction::Loop(BlockType::Empty));
                self.emit_drain_promise_jobs(&mut function)?;
                self.emit_drain_atomics_wait_async_timeouts(&mut function)?;
                function.instruction(&Instruction::BrIf(0));
                function.instruction(&Instruction::End);
                function.instruction(&Instruction::End);
            } else {
                self.emit_drain_promise_jobs(&mut function)?;
            }
            // Every job that could still attach a handler has now run, so a
            // promise still marked unhandled really is an unhandled rejection.
            self.emit_report_unhandled_rejection(&mut function)?;
        }
        assert!(
            self.next_binding_local <= self.current_env_local,
            "binding local planner boundary {} exceeded by next local {}",
            self.current_env_local,
            self.next_binding_local
        );
        self.pop_scope();

        match self.return_abi {
            ReturnAbi::MainExport => {
                function.instruction(&Instruction::LocalGet(self.result_tag_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::GlobalSet(RESULT_TAG_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::GlobalSet(COMPLETION_KIND_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalGet(self.completion_aux_local));
                function.instruction(&Instruction::I32WrapI64);
                function.instruction(&Instruction::GlobalSet(COMPLETION_AUX_GLOBAL_INDEX));
                function.instruction(&Instruction::LocalGet(self.result_local));
            }
            ReturnAbi::MultiValue => {
                function.instruction(&Instruction::LocalGet(self.result_local));
                function.instruction(&Instruction::LocalGet(self.result_tag_local));
                function.instruction(&Instruction::LocalGet(self.completion_local));
                function.instruction(&Instruction::LocalGet(self.completion_aux_local));
            }
        }
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    fn init_template_objects(&mut self, function: &mut Function) -> Result<(), EmitError> {
        if !self.is_main() {
            return Ok(());
        }

        let templates = self
            .strings
            .template_objects
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for template in templates {
            let raw_elements = template
                .raw
                .iter()
                .cloned()
                .map(|raw| {
                    TypedExpr::from_info(ValueInfo::new(ValueKind::String), ExprIr::String(raw))
                })
                .collect::<Vec<_>>();
            let cooked_elements = template
                .cooked
                .iter()
                .map(|cooked| match cooked {
                    Some(cooked) => TypedExpr::from_info(
                        ValueInfo::new(ValueKind::String),
                        ExprIr::String(cooked.clone()),
                    ),
                    None => TypedExpr::undefined(),
                })
                .collect::<Vec<_>>();

            let raw_local = self.reserve_temp_local();
            self.compile_array_literal_payload(&raw_elements, function)?;
            function.instruction(&Instruction::LocalSet(raw_local));
            self.freeze_template_array(raw_local, raw_elements.len(), function);

            let cooked_local = self.reserve_temp_local();
            self.compile_array_literal_payload(&cooked_elements, function)?;
            function.instruction(&Instruction::LocalSet(cooked_local));

            let key_local = self.reserve_temp_local();
            let raw_tag_local = self.reserve_temp_local();
            let false_local = self.reserve_temp_local();
            function.instruction(&Instruction::I64Const(self.strings.payload("raw")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
            function.instruction(&Instruction::LocalSet(raw_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(false_local));
            self.emit_array_define_named_data_descriptor(
                cooked_local,
                key_local,
                raw_local,
                raw_tag_local,
                false_local,
                false_local,
                false_local,
                None,
                None,
                None,
                None,
                None,
                function,
            )?;
            self.freeze_template_array(cooked_local, cooked_elements.len(), function);
            function.instruction(&Instruction::LocalGet(cooked_local));
            function.instruction(&Instruction::GlobalSet(
                self.template_object_global_index(template.site_id),
            ));

            self.release_temp_local(false_local);
            self.release_temp_local(raw_tag_local);
            self.release_temp_local(key_local);
            self.release_temp_local(cooked_local);
            self.release_temp_local(raw_local);
        }
        Ok(())
    }

    fn freeze_template_array(
        &mut self,
        array_local: u32,
        element_count: usize,
        function: &mut Function,
    ) {
        let buffer_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let descriptor_kind_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(array_local, HEAP_PTR_OFFSET, buffer_local, function);
        for index in 0..element_count {
            function.instruction(&Instruction::LocalGet(buffer_local));
            function.instruction(&Instruction::I64Const(
                (index as u64 * HEAP_ARRAY_ENTRY_SIZE) as i64,
            ));
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(entry_local));
            self.load_i64_to_local_from_offset(
                entry_local,
                HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
                descriptor_kind_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(descriptor_kind_local));
            function.instruction(&Instruction::I64Const(
                !((OBJECT_DESCRIPTOR_CONFIGURABLE | OBJECT_DESCRIPTOR_WRITABLE) as i64),
            ));
            function.instruction(&Instruction::I64And);
            function.instruction(&Instruction::LocalSet(descriptor_kind_local));
            self.store_i64_local_at_offset(
                entry_local,
                HEAP_ARRAY_DESCRIPTOR_KIND_OFFSET,
                descriptor_kind_local,
                function,
            );
        }
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(descriptor_kind_local));
        self.emit_array_store_length_writable_descriptor(
            array_local,
            descriptor_kind_local,
            function,
        );
        self.store_i64_const_at_offset(array_local, HEAP_ARRAY_NON_EXTENSIBLE_OFFSET, 1, function);
        self.release_temp_local(descriptor_kind_local);
        self.release_temp_local(entry_local);
        self.release_temp_local(buffer_local);
    }

    fn ensure_heap_ptr_after_static_data(&self, function: &mut Function) {
        if !self.is_main() || !self.uses_heap {
            return;
        }
        let heap_start = align_heap_start(self.strings.bytes.len()) as i64;
        function.instruction(&Instruction::I64Const(heap_start));
        function.instruction(&Instruction::GlobalSet(HEAP_PTR_GLOBAL_INDEX));
    }

    fn init_current_realm(&mut self, function: &mut Function) -> Result<(), EmitError> {
        if !self.is_main() || !self.uses_heap {
            return Ok(());
        }
        let realm_local = self.reserve_temp_local();
        self.emit_alloc_realm_record(1, 1, realm_local, function)?;
        function.instruction(&Instruction::LocalGet(realm_local));
        function.instruction(&Instruction::GlobalSet(CURRENT_REALM_GLOBAL_INDEX));
        self.release_temp_local(realm_local);
        Ok(())
    }

    fn compile_builtin(&mut self) -> Result<Function, EmitError> {
        let Some(function_id) = self.function_id.clone() else {
            return Err(EmitError::unsupported(
                "unsupported in porffor wasm-aot first slice: missing builtin id",
            ));
        };
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.push_scope();
        self.init_current_env(&mut function)?;
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        if let Some(builtin) = StandardBuiltinId::from_function_id(&function_id) {
            if self.stub_standard_builtin_body {
                self.emit_throw_runtime_error(
                    TYPE_ERROR_NAME,
                    &format!(
                        "standard builtin body is not emitted unless referenced directly: {}",
                        builtin.debug_name()
                    ),
                    self.result_local,
                    self.result_tag_local,
                    &mut function,
                )?;
                self.emit_return_current_completion(&mut function);
            } else {
                self.compile_standard_builtin(builtin, &mut function)?;
            }
        } else {
            match HostBuiltinId::from_function_id(&function_id) {
                Some(HostBuiltinId::Print) => self.compile_host_print_builtin(&mut function)?,
                Some(HostBuiltinId::Gc) => self.compile_host_gc_builtin(&mut function)?,
                Some(HostBuiltinId::AssertThrows) => {
                    self.compile_host_assert_throws_builtin(&mut function)?
                }
                Some(HostBuiltinId::IsConstructor) => {
                    self.compile_host_is_constructor_builtin(&mut function)?
                }
                Some(HostBuiltinId::CreateRealm) => {
                    self.compile_host_create_realm_builtin(&mut function)?
                }
                Some(HostBuiltinId::CreateHTMLDDA) => {
                    self.compile_host_create_html_dda_builtin(&mut function)?
                }
                Some(HostBuiltinId::HTMLDDA) => {
                    self.compile_host_html_dda_builtin(&mut function)?
                }
                Some(HostBuiltinId::ParseInt) => {
                    self.compile_host_parse_int_builtin(&mut function)?
                }
                Some(HostBuiltinId::ParseFloat) => {
                    self.compile_host_parse_float_builtin(&mut function)?
                }
                Some(HostBuiltinId::DetachArrayBuffer) => {
                    self.compile_host_detach_array_buffer_builtin(&mut function)?
                }
                Some(HostBuiltinId::AgentStart) => {
                    self.compile_host_agent_start_builtin(&mut function)?
                }
                Some(HostBuiltinId::AgentBroadcast) => {
                    self.compile_host_agent_broadcast_builtin(&mut function)?
                }
                Some(HostBuiltinId::AgentReceiveBroadcast) => {
                    self.compile_host_agent_receive_broadcast_builtin(&mut function)?
                }
                Some(HostBuiltinId::AgentReport) => {
                    self.compile_host_agent_report_builtin(&mut function)?
                }
                Some(HostBuiltinId::AgentGetReport) => {
                    self.compile_host_agent_get_report_builtin(&mut function)?
                }
                Some(HostBuiltinId::AgentSleep) => {
                    self.compile_host_agent_sleep_builtin(&mut function)?
                }
                Some(HostBuiltinId::AgentMonotonicNow) => {
                    self.compile_host_agent_monotonic_now_builtin(&mut function)?
                }
                Some(HostBuiltinId::AgentLeaving) => {
                    self.compile_host_agent_leaving_builtin(&mut function)?
                }
                None => {
                    return Err(EmitError::unsupported(format!(
                        "unsupported in porffor wasm-aot first slice: unknown builtin `{function_id}`"
                    )));
                }
            }
        }
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared object-read runtime helper. Rather than inlining the
    /// large ordinary/proxy property-read state machine at every read site, that
    /// sequence is emitted exactly once here and reached with a plain `call`.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`] (seven i64 params, four i64
    /// results). Params: 0=object payload, 1=object tag, 2=receiver payload,
    /// 3=receiver tag, 4=key payload. Params 5/6 are unused. Results are the
    /// standard `(result, result_tag, completion, completion_aux)` tuple.
    fn compile_object_read_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_object_read = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_object_read_ordinary_inner(
            0,
            1,
            2,
            3,
            4,
            self.result_local,
            self.result_tag_local,
            None,
            &mut function,
        )?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared object-write runtime helper. The large ordinary/proxy
    /// property-write state machine is emitted once here and reached with a
    /// plain `call`.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=object payload,
    /// 1=object tag, 2=key payload, 3=value payload, 4=value tag, 5=calling
    /// function strictness, 6=calling standard builtin's realm environment (or
    /// zero). On a setter/proxy throw the thrown value is surfaced through the
    /// `(result, result_tag, completion, completion_aux)` result tuple.
    fn compile_object_write_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_object_write = false;
        self.ordinary_set_data_on_receiver_emission = OrdinarySetDataOnReceiverEmission::Outlined;
        // Helper parameter 5 carries the calling function's strictness (0 sloppy,
        // nonzero strict). Parameter 6 carries a standard builtin's self-backed
        // realm environment; other callers pass zero. This lets ArraySetLength
        // create a RangeError in the calling builtin's Realm.
        self.object_write_strict_flag_local = Some(5);
        self.push_scope();
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_object_write(0, 1, 2, 3, 4, &mut function)?;
        self.pop_scope();
        self.object_write_strict_flag_local = None;
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the receiver-side data-property step used by OrdinarySet.
    /// This state machine is repeated several times inside the shared
    /// object-write helper, so that helper calls this single outlined copy.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=receiver
    /// payload, 1=receiver tag, 2=key payload, 3=key tag, 4=value payload,
    /// 5=value tag, 6=calling realm environment. Results are the standard
    /// `(result, result_tag, completion, aux)` tuple; on normal completion the
    /// first result is the boolean success value.
    fn compile_ordinary_set_data_on_receiver_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.push_scope();
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_ordinary_set_data_on_receiver_result_with_depth(
            0,
            1,
            2,
            3,
            4,
            5,
            self.result_local,
            4,
            false,
            &mut function,
        )?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the receiver-side OrdinarySet step used when an exotic
    /// receiver must fall back to its generic `[[Set]]` behavior.
    fn compile_ordinary_set_data_on_receiver_with_fallback_helper(
        &mut self,
    ) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.push_scope();
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_ordinary_set_data_on_receiver_result_with_depth(
            0,
            1,
            2,
            3,
            4,
            5,
            self.result_local,
            4,
            true,
            &mut function,
        )?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared Array element-write state machine. Argument-vector
    /// construction and builtin internals use this path often enough that
    /// inlining it can exceed Cranelift's per-function code-size limit.
    /// Params: 0=array payload, 1=index, 2=value payload, 3=value tag,
    /// 4=calling realm environment. Params 5/6 are unused.
    fn compile_array_write_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_array_write = false;
        self.push_scope();
        function.instruction(&Instruction::LocalGet(4));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_array_write(0, 1, 2, 3, &mut function)?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles OrdinarySet with an explicit receiver once for callers such as
    /// `Reflect.set`. The five tagged inputs are passed through the standard
    /// argument vector in params 5/6.
    fn compile_ordinary_set_helper(
        &mut self,
        allow_receiver_generic_write_fallback: bool,
    ) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.ordinary_set_data_on_receiver_emission = OrdinarySetDataOnReceiverEmission::Outlined;
        let target_payload_local = self.reserve_temp_local();
        let target_tag_local = self.reserve_temp_local();
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let key_payload_local = self.reserve_temp_local();
        let key_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let realm_environment_local = self.reserve_temp_local();
        let realm_environment_tag_local = self.reserve_temp_local();
        let strict_local = self.reserve_temp_local();

        self.push_scope();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(strict_local));
        self.object_write_strict_flag_local = Some(strict_local);
        self.emit_builtin_arg_to_locals(0, target_payload_local, target_tag_local, &mut function);
        self.emit_builtin_arg_to_locals(
            1,
            receiver_payload_local,
            receiver_tag_local,
            &mut function,
        );
        self.emit_builtin_arg_to_locals(2, key_payload_local, key_tag_local, &mut function);
        self.emit_builtin_arg_to_locals(3, value_payload_local, value_tag_local, &mut function);
        self.emit_builtin_arg_to_locals(
            4,
            realm_environment_local,
            realm_environment_tag_local,
            &mut function,
        );
        function.instruction(&Instruction::LocalGet(realm_environment_local));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_ordinary_set_result_with_receiver_fallback(
            target_payload_local,
            target_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            key_payload_local,
            key_tag_local,
            value_payload_local,
            value_tag_local,
            self.result_local,
            allow_receiver_generic_write_fallback,
            &mut function,
        )?;
        self.object_write_strict_flag_local = None;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        function.instruction(&Instruction::End);
        self.pop_scope();

        self.release_temp_local(strict_local);
        self.release_temp_local(realm_environment_tag_local);
        self.release_temp_local(realm_environment_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_tag_local);
        self.release_temp_local(key_payload_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        self.release_temp_local(target_tag_local);
        self.release_temp_local(target_payload_local);
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared object-define-data runtime helper. Bootstrap-style
    /// code (realm setup, `$262.createRealm`) defines hundreds of data
    /// properties; emitting the define state machine once here keeps those
    /// functions under Cranelift's per-function limit.
    ///
    /// Wasm signature is [`OBJECT_APPEND_ACCESSOR_PROPERTY_TYPE_INDEX`] (seven
    /// i64 params, no results). Params: 0=object payload, 1=key payload,
    /// 2=value payload, 3=value tag, 4=writable, 5=enumerable, 6=configurable.
    fn compile_object_define_data_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_object_define_data = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_object_define_data_with_flag_locals(0, 1, 2, 3, 4, 5, 6, &mut function)?;
        self.pop_scope();
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared plain function-call dispatcher.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=callee payload,
    /// 1=callee tag, 2=this payload, 3=this tag, 4=argc, 5=argv. Param 6 is
    /// unused. Results are the `(result, result_tag, completion, aux)` tuple;
    /// throws are surfaced through the completion rather than propagated.
    fn compile_function_call_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_function_call = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_function_handle_call_with_argv_inner(
            0,
            1,
            Some((2, Some(3))),
            4,
            5,
            self.result_local,
            self.result_tag_local,
            None,
            &mut function,
        )?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared runtime-kind dynamic property-read dispatcher.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=target payload,
    /// 1=target tag, 2=receiver payload, 3=receiver tag, 4=property-key payload,
    /// 5=property-key tag. Param 6 is unused. Results are the standard
    /// `(result, result_tag, completion, aux)` tuple.
    fn compile_dynamic_property_read_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_dynamic_property_read_with_key_locals(
            0,
            1,
            2,
            3,
            4,
            5,
            self.result_local,
            self.result_tag_local,
            &mut function,
        )?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared proxy-aware call-dispatch helper. The proxy call
    /// state machine (walk the proxy chain, invoke the `apply` trap, otherwise
    /// `call_indirect`) is emitted once here and reached with a plain `call`.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=callee payload,
    /// 1=callee tag, 2=this payload, 3=this tag, 4=argc, 5=argv. Params 6 is
    /// unused. Results are the `(result, result_tag, completion, aux)` tuple;
    /// throws are surfaced through the completion rather than propagated.
    fn compile_proxy_call_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_proxy_call = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_function_or_proxy_call_with_argv_inner(
            0,
            1,
            2,
            3,
            4,
            5,
            self.result_local,
            self.result_tag_local,
            false,
            &mut function,
        )?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared proxy-aware construct-dispatch helper.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=callee payload,
    /// 1=callee tag, 2=new.target payload, 3=new.target tag, 4=argc, 5=argv.
    /// Param 6 is unused. Results are the `(result, result_tag, completion,
    /// aux)` tuple.
    fn compile_proxy_construct_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_proxy_construct = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_function_or_proxy_construct_with_argv(
            0,
            1,
            2,
            3,
            4,
            5,
            self.result_local,
            self.result_tag_local,
            &mut function,
        )?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared string-payload-equality helper. Builtin bodies
    /// compare interned string payloads at thousands of sites (property-name
    /// matching, key switches); the ~65-instruction byte-compare loop is emitted
    /// once here and reached with a plain `call`, keeping the largest builtin
    /// bodies under Cranelift's per-function virtual-register limit.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=lhs string
    /// payload, 1=rhs string payload, 2=ASCII-case-fold mode. Params 3-6 are
    /// unused. Results are the
    /// standard four-i64 tuple with the comparison result (0 or 1) in the first
    /// slot; the other three are always zero.
    fn compile_string_equality_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_string_equality = false;
        self.push_scope();
        self.emit_string_payload_equality_i32_with_ascii_case_folding(0, 1, Some(2), &mut function);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared number-to-string helper (the ECMAScript Number→
    /// String digit-emission state machine, several KB per inline copy).
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=number payload
    /// (f64 bits). Params 1-6 are unused. Results are the standard four-i64
    /// tuple with the string payload in the first slot; the rest are zero.
    fn compile_number_to_string_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_number_to_string = false;
        self.push_scope();
        self.emit_number_to_string_payload(0, &mut function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared string-to-number helper (the ECMAScript String→
    /// Number parse state machine, several KB per inline copy).
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=string payload.
    /// Params 1-6 are unused. Results are the standard four-i64 tuple with the
    /// number payload (f64 bits) in the first slot; the rest are zero.
    fn compile_string_to_number_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_string_to_number = false;
        self.push_scope();
        self.emit_string_to_number_payload(0, &mut function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared dynamic ToString helper (per-kind dispatch,
    /// ToPrimitive on objects, array join, function source text — tens of KB
    /// per inline copy, and dynamic string concatenation hits it constantly).
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=value payload,
    /// 1=value tag. Params 2-6 are unused. Results are the standard four-i64
    /// tuple: on normal completion the string payload is in the first slot; a
    /// ToPrimitive/Symbol throw is surfaced through the completion slots.
    fn compile_value_to_string_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_value_to_string = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_value_to_string_payload(0, 1, &mut function)?;
        // A Symbol/ToPrimitive throw deep inside (see
        // `emit_object_to_primitive_locals_locals_inner`) leaves completion=THROW
        // with the real error already in `self.result_local` without branching;
        // only commit the computed string payload over `self.result_local` on
        // the normal-completion path, else the throw would be silently replaced
        // by whatever placeholder value the dispatch produced.
        let computed_value_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalSet(computed_value_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(computed_value_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(computed_value_local);
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        // The result tag is String on normal completion, but on a throw
        // `self.result_local` holds the real thrown error (typically an Object),
        // not a string — report its actual tag instead of hardcoding String.
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared dynamic ToNumber helper (per-kind dispatch,
    /// ToPrimitive on objects, array→string coercion, BigInt/Symbol throws,
    /// string parse — several KB per inline copy across ~130 builtin sites).
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=value payload,
    /// 1=value tag. Params 2-6 are unused. Results are the standard four-i64
    /// tuple: on normal completion the number payload (f64 bits) is in the
    /// first slot with a Number tag; a BigInt/Symbol/ToPrimitive throw is
    /// surfaced through the completion slots.
    fn compile_value_to_number_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_value_to_number = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_value_to_number_payload(1, 0, &mut function)?;
        // Same discipline as `compile_value_to_string_helper`: a BigInt/Symbol/
        // ToPrimitive throw leaves completion=THROW with the real error already
        // in `self.result_local` without branching, so only commit the computed
        // number payload on the normal-completion path.
        let computed_value_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalSet(computed_value_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(computed_value_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);
        self.release_temp_local(computed_value_local);
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        // Same reasoning as `compile_value_to_string_helper`: report the real
        // thrown error's tag on a throw instead of hardcoding Number.
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Result(ValType::I64)));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared dynamic ToNumeric helper.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params 0 and 1 contain
    /// the input payload and tag, and param 6 contains the calling function's
    /// realm environment. The standard four-i64 result tuple preserves the
    /// resulting Number-or-BigInt tag and any abrupt completion produced by
    /// ToPrimitive.
    fn compile_value_to_numeric_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_value_to_numeric = false;
        self.push_scope();
        function.instruction(&Instruction::LocalGet(6));
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_value_to_numeric_locals(0, 1, &mut function)?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalGet(1));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared proxy-aware `[[GetPrototypeOf]]` helper. The proxy
    /// get-prototype-of state machine (walk the proxy chain, invoke the
    /// `getPrototypeOf` trap, validate against a non-extensible target) is
    /// emitted once here and reached with a plain `call`, instead of being
    /// inlined (~356KB) at every `instanceof`/prototype-walk site in a
    /// realm/proxy-enabled module.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=object payload,
    /// 1=object tag. Params 2-6 are unused. Results are the standard four-i64
    /// tuple: on normal completion the prototype `(payload, tag)` is in the first
    /// two slots; a proxy-trap throw is surfaced through the completion slots.
    fn compile_object_get_prototype_of_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_object_get_prototype_of = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_object_get_prototype_of_with_depth(
            0,
            1,
            self.result_local,
            self.result_tag_local,
            0,
            &mut function,
        )?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared proxy-aware `[[IsExtensible]]` helper, called by the
    /// get-prototype-of helper (and the `Object`/`Reflect` extensibility
    /// builtins) instead of inlining its proxy-trap walk.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=object payload,
    /// 1=object tag. Params 2-6 are unused. Results are the standard four-i64
    /// tuple: on normal completion the boolean result (0/1) is in the first slot;
    /// a proxy-trap throw is surfaced through the completion slots.
    fn compile_object_is_extensible_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_object_is_extensible = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_object_is_extensible_i32_with_depth(0, 1, self.result_local, 0, &mut function)?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    /// Compiles the shared proxy-aware `[[Get]]` helper. The proxy read wrapper
    /// (proxy-handler check, `get` trap invoke, invariant validation, one-level
    /// nested-proxy target unroll) is emitted once here and reached with a plain
    /// `call`, instead of being inlined (~21KB) at every dynamic property read in
    /// a realm/proxy-enabled module.
    ///
    /// Wasm signature is [`JS_FUNCTION_TYPE_INDEX`]. Params: 0=object payload,
    /// 1=object tag, 2=receiver payload, 3=receiver tag, 4=key payload, 5=key
    /// tag. Param 6 is unused. Results are the standard four-i64 tuple: on normal
    /// completion the value `(payload, tag)` is in the first two slots; a
    /// proxy-trap throw is surfaced through the completion slots.
    fn compile_object_read_proxy_helper(&mut self) -> Result<Function, EmitError> {
        let mut function =
            Function::new_with_locals_types(std::iter::repeat_n(ValType::I64, self.local_count()));
        self.outline_object_read_proxy = false;
        self.push_scope();
        self.set_completion_kind(CompletionKind::Normal, &mut function);
        self.emit_statement_result(&mut function, ValueKind::Undefined);
        self.emit_object_read_with_key_tag(
            0,
            1,
            2,
            3,
            4,
            Some(5),
            self.result_local,
            self.result_tag_local,
            &mut function,
        )?;
        self.pop_scope();
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::LocalGet(self.completion_aux_local));
        function.instruction(&Instruction::End);
        Ok(self.finish_function(function))
    }

    fn init_current_env(&mut self, function: &mut Function) -> Result<(), EmitError> {
        match self.return_abi {
            ReturnAbi::MainExport => {
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.current_env_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.class_function_context_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.named_function_context_local));
            }
            ReturnAbi::MultiValue => {
                function.instruction(&Instruction::LocalGet(0));
                function.instruction(&Instruction::LocalSet(self.current_env_local));
                if self
                    .current_function_meta()
                    .is_some_and(WasmFunctionMeta::has_function_context)
                {
                    // Functions that need execution context state receive an
                    // immutable context in their env parameter. Lexical lookup
                    // resumes from the environment captured inside it.
                    function.instruction(&Instruction::LocalGet(self.current_env_local));
                    function.instruction(&Instruction::LocalSet(self.class_function_context_local));
                    self.load_i64_to_local_from_offset(
                        self.current_env_local,
                        HEAP_CLASS_FUNCTION_CONTEXT_LEXICAL_ENV_OFFSET,
                        self.current_env_local,
                        function,
                    );
                } else {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(self.class_function_context_local));
                }
                if self
                    .current_function_meta()
                    .is_some_and(|meta| meta.is_named_expression)
                {
                    function.instruction(&Instruction::LocalGet(self.current_env_local));
                    function.instruction(&Instruction::LocalSet(self.named_function_context_local));
                    self.load_i64_to_local_from_offset(
                        self.current_env_local,
                        ENV_PARENT_OFFSET,
                        self.current_env_local,
                        function,
                    );
                } else {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(self.named_function_context_local));
                }
            }
        }

        let resumable_activation = self.current_function_meta().and_then(|meta| {
            let environment_offset = match meta.execution_kind {
                FunctionExecutionKind::Generator => HEAP_GENERATOR_ENV_OFFSET,
                FunctionExecutionKind::Async => HEAP_ASYNC_ENV_OFFSET,
                FunctionExecutionKind::AsyncGenerator => HEAP_ASYNC_GENERATOR_LEXICAL_ENV_OFFSET,
                FunctionExecutionKind::Ordinary => return None,
            };
            self.new_target_payload_local()
                .map(|activation_local| (activation_local, environment_offset))
        });
        if self.owned_env_bindings.is_empty() {
            if let Some((activation_local, environment_offset)) = resumable_activation {
                self.load_i64_to_local_from_offset(
                    activation_local,
                    environment_offset,
                    self.scratch_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(self.scratch_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::LocalGet(self.scratch_local));
                function.instruction(&Instruction::LocalSet(self.current_env_local));
                function.instruction(&Instruction::Else);
                self.store_i64_local_at_offset(
                    activation_local,
                    environment_offset,
                    self.current_env_local,
                    function,
                );
                function.instruction(&Instruction::End);
            }
            return Ok(());
        }

        if let Some((activation_local, environment_offset)) = resumable_activation {
            self.load_i64_to_local_from_offset(
                activation_local,
                environment_offset,
                self.scratch_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(self.scratch_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::I64Ne);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(self.scratch_local));
            function.instruction(&Instruction::LocalSet(self.current_env_local));
            function.instruction(&Instruction::Else);
        }

        let parent_env_local = self.reserve_temp_local();
        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalSet(parent_env_local));
        self.emit_heap_alloc_const(
            ENV_SLOT_BASE_OFFSET + self.owned_env_bindings.len() as u64 * ENV_SLOT_SIZE,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(self.current_env_local));
        self.store_i64_local_at_offset(
            self.current_env_local,
            ENV_PARENT_OFFSET,
            parent_env_local,
            function,
        );
        for binding in self.owned_env_bindings {
            self.store_i64_const_at_offset(
                self.current_env_local,
                ENV_SLOT_BASE_OFFSET + binding.slot as u64 * ENV_SLOT_SIZE + ENV_SLOT_TAG_OFFSET,
                ENV_SLOT_UNINITIALIZED_TAG as u64,
                function,
            );
            self.store_i64_const_at_offset(
                self.current_env_local,
                ENV_SLOT_BASE_OFFSET
                    + binding.slot as u64 * ENV_SLOT_SIZE
                    + ENV_SLOT_PAYLOAD_OFFSET,
                0,
                function,
            );
        }
        if self.function_flavor == FunctionFlavor::Ordinary {
            if self.is_derived_constructor {
                let activation = self
                    .lexical_derived_activation
                    .expect("derived constructor must have activation metadata");
                self.initialize_derived_activation(activation, function)?;
            }
            if let Some(slot) = self.owned_env_slot(LEXICAL_THIS_NAME) {
                if self.is_main() {
                    self.release_temp_local(parent_env_local);
                    return Ok(());
                }
                let Some(this_payload_local) = self.this_payload_local else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: top-level `this`",
                    ));
                };
                let Some(this_tag_local) = self.this_tag_local else {
                    return Err(EmitError::unsupported(
                        "unsupported in porffor wasm-aot first slice: missing `this` tag local",
                    ));
                };
                self.write_binding_from_locals(
                    BindingStorage::EnvSlot { slot, hops: 0 },
                    this_payload_local,
                    this_tag_local,
                    function,
                );
            }
            if let Some(slot) = self.owned_env_slot(LEXICAL_NEW_TARGET_NAME) {
                if self.current_function_meta().is_some_and(|meta| {
                    matches!(
                        meta.execution_kind,
                        FunctionExecutionKind::Generator
                            | FunctionExecutionKind::Async
                            | FunctionExecutionKind::AsyncGenerator
                    )
                }) {
                    function.instruction(&Instruction::I64Const(0));
                    function.instruction(&Instruction::LocalSet(self.scratch_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::LocalSet(self.result_tag_local));
                    self.write_binding_from_locals(
                        BindingStorage::EnvSlot { slot, hops: 0 },
                        self.scratch_local,
                        self.result_tag_local,
                        function,
                    );
                } else {
                    let Some(new_target_payload_local) = self.new_target_payload_local() else {
                        return Err(EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing `new.target` payload local",
                        ));
                    };
                    let Some(new_target_tag_local) = self.new_target_tag_local() else {
                        return Err(EmitError::unsupported(
                            "unsupported in porffor wasm-aot first slice: missing `new.target` tag local",
                        ));
                    };
                    self.write_binding_from_locals(
                        BindingStorage::EnvSlot { slot, hops: 0 },
                        new_target_payload_local,
                        new_target_tag_local,
                        function,
                    );
                }
            }
            if let Some(slot) = self.owned_env_slot(LEXICAL_HOME_OBJECT_NAME) {
                self.load_i64_to_local_from_offset(
                    self.class_function_context_local,
                    HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_PAYLOAD_OFFSET,
                    self.scratch_local,
                    function,
                );
                self.load_i64_to_local_from_offset(
                    self.class_function_context_local,
                    HEAP_CLASS_FUNCTION_CONTEXT_HOME_OBJECT_TAG_OFFSET,
                    self.result_tag_local,
                    function,
                );
                self.write_binding_from_locals(
                    BindingStorage::EnvSlot { slot, hops: 0 },
                    self.scratch_local,
                    self.result_tag_local,
                    function,
                );
            }
        }
        if let Some((activation_local, environment_offset)) = resumable_activation {
            self.store_i64_local_at_offset(
                activation_local,
                environment_offset,
                self.current_env_local,
                function,
            );
            function.instruction(&Instruction::End);
        }
        self.release_temp_local(parent_env_local);
        Ok(())
    }

    /// Seeds compiler-private derived-constructor activation slots in the
    /// freshly allocated invocation environment.  This state must never live
    /// in the function object's immutable lexical context: recursion and
    /// re-entrant construction each require independent `this` status.
    fn initialize_derived_activation(
        &mut self,
        activation: &DerivedConstructorActivationIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let this = self
            .owned_env_slot(&activation.this_binding)
            .ok_or_else(|| {
                EmitError::unsupported("derived constructor activation owner has no `this` slot")
            })?;
        let status = self
            .owned_env_slot(&activation.this_status_binding)
            .ok_or_else(|| {
                EmitError::unsupported("derived constructor activation owner has no status slot")
            })?;
        let new_target = self
            .owned_env_slot(&activation.new_target_binding)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "derived constructor activation owner has no new.target slot",
                )
            })?;
        let active_function = self
            .owned_env_slot(&activation.active_function_binding)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "derived constructor activation owner has no active-function slot",
                )
            })?;
        let (Some(new_target_payload), Some(new_target_tag)) =
            (self.new_target_payload_local(), self.new_target_tag_local())
        else {
            return Err(EmitError::unsupported(
                "derived constructor activation requires the multi-value call ABI",
            ));
        };

        // `this` is deliberately initialized to an unobservable undefined
        // value.  GetDerivedThis gates it on the false status slot.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.write_env_slot_from_locals(
            this,
            0,
            self.scratch_local,
            self.result_tag_local,
            function,
        );

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.scratch_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.write_env_slot_from_locals(
            status,
            0,
            self.scratch_local,
            self.result_tag_local,
            function,
        );
        self.write_env_slot_from_locals(
            new_target,
            0,
            new_target_payload,
            new_target_tag,
            function,
        );

        // Param 0 is the immutable function context.  The current functions
        // ABI stores the executing function object in that context; later
        // construct lowering consumes this activation slot rather than a
        // mutable per-function context field.
        self.load_i64_to_local_from_offset(
            0,
            HEAP_CLASS_FUNCTION_CONTEXT_ACTIVE_FUNCTION_OFFSET,
            self.scratch_local,
            function,
        );
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.write_env_slot_from_locals(
            active_function,
            0,
            self.scratch_local,
            self.result_tag_local,
            function,
        );
        Ok(())
    }

    pub(crate) const fn memarg32(offset: u64) -> MemArg {
        Self::memarg32_in(0, offset)
    }

    pub(crate) const fn memarg32_in(memory_index: u32, offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 2,
            memory_index,
        }
    }

    pub(crate) const fn memarg16(offset: u64) -> MemArg {
        Self::memarg16_in(0, offset)
    }

    pub(crate) const fn memarg16_in(memory_index: u32, offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 1,
            memory_index,
        }
    }

    pub(crate) const fn memarg8(offset: u64) -> MemArg {
        Self::memarg8_in(0, offset)
    }

    pub(crate) const fn memarg8_in(memory_index: u32, offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 0,
            memory_index,
        }
    }

    pub(crate) const fn shared_memarg64(offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 3,
            memory_index: 1,
        }
    }

    pub(crate) const fn shared_memarg32(offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 2,
            memory_index: 1,
        }
    }

    pub(crate) const fn shared_memarg16(offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 1,
            memory_index: 1,
        }
    }

    pub(crate) const fn shared_memarg8(offset: u64) -> MemArg {
        MemArg {
            offset,
            align: 0,
            memory_index: 1,
        }
    }

    pub(crate) fn buffer_memarg64(&self, offset: u64) -> MemArg {
        Self::memarg64_in(self.buffer_memory_index(), offset)
    }

    pub(crate) fn buffer_memarg32(&self, offset: u64) -> MemArg {
        Self::memarg32_in(self.buffer_memory_index(), offset)
    }

    pub(crate) fn buffer_memarg16(&self, offset: u64) -> MemArg {
        Self::memarg16_in(self.buffer_memory_index(), offset)
    }

    pub(crate) fn buffer_memarg8(&self, offset: u64) -> MemArg {
        Self::memarg8_in(self.buffer_memory_index(), offset)
    }

    pub(crate) fn buffer_memory_index(&self) -> u32 {
        // Split modules keep object/runtime state in private memory 0 while
        // allocating every ArrayBuffer backing store from memory 1. Ordinary
        // buffers remain semantically private because only their owning
        // instance has metadata containing the disjoint host allocation.
        u32::from(
            self.functions
                .shared_memory_alloc_function_index()
                .is_some(),
        )
    }
}
