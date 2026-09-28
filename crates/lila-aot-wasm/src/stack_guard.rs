//! Planning and emission primitives for catchable native-stack exhaustion.
//!
//! Each original defined-function index remains the callable guard wrapper.
//! The real body is relocated after all original defined functions, so direct
//! calls, table elements, and other references already naming that index keep
//! entering through the guard. The wrapper passes the relocated body's defined
//! index and its own defined index to a leaf host callback before tail-calling
//! the real body. Relocated bodies capture their entry Realm and restore it
//! after ordinary calls, while tail calls deliberately keep the callee's Realm
//! active.

use wasm_encoder::{BlockType, Instruction, ValType};

use crate::abi::{CallAbi, CallResultSlot};
use crate::code_sink::Function;
use crate::environments::global_environment::GLOBAL_ENV_REALM_OFFSET;
use crate::heap::{
    ENV_PARENT_OFFSET, HEAP_CLASS_FUNCTION_CONTEXT_LEXICAL_ENV_OFFSET,
    HEAP_FUNCTION_DEFINING_REALM_OFFSET, HEAP_REALM_INTRINSICS_OFFSET,
    HEAP_REALM_INTRINSICS_RANGE_ERROR_PROTOTYPE_OFFSET,
};

/// Wasm custom-section name carrying stack-guard relocation metadata.
pub const STACK_GUARD_CUSTOM_SECTION: &str = "lila.stack_guard";
const STACK_GUARD_METADATA_MAGIC: &[u8; 8] = b"LILASTK\0";
const STACK_GUARD_METADATA_VERSION: u32 = 2;

/// An index in a module's defined-function space (imports are excluded).
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct DefinedFunctionIndex(u32);

impl DefinedFunctionIndex {
    pub(crate) const fn new(index: u32) -> Self {
        Self(index)
    }

    pub(crate) const fn get(self) -> u32 {
        self.0
    }

    pub(crate) fn absolute_wasm_index(self, imported_function_count: u32) -> Option<u32> {
        imported_function_count.checked_add(self.0)
    }
}

/// One original JS-ABI function, now represented by a wrapper at its old index
/// and a body relocated after the original function space.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GuardedFunction {
    wrapper: DefinedFunctionIndex,
    body: DefinedFunctionIndex,
}

/// Where a guarded wrapper can obtain the called function's Realm without
/// entering its body. Prepared-script thunks are classified separately from
/// ordinary closures emitted inside them: the thunk's ten-parameter ABI starts
/// with a lexical environment, while its caller installs the selected Realm in
/// `CURRENT_REALM` before entering it. JS helpers have heterogeneous arg0
/// values and therefore inherit the Realm already installed by their caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FunctionRealmSource {
    PreparedScript,
    LexicalEnvironment {
        has_function_context: bool,
        missing_environment: MissingEnvironmentRealmSource,
    },
    FunctionEnvironment,
    Active,
}

/// How a wrapper resolves the target Realm when its ABI legitimately carries
/// a zero environment before runtime initialization.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MissingEnvironmentRealmSource {
    Trap,
    ActiveOrCurrent,
}

impl GuardedFunction {
    pub(crate) const fn wrapper(self) -> DefinedFunctionIndex {
        self.wrapper
    }

    pub(crate) const fn body(self) -> DefinedFunctionIndex {
        self.body
    }
}

/// The complete relocation map for the functions selected for stack guards.
///
/// The caller selects only emitted JS-ABI bodies (ordinary source functions,
/// builtins, host builtins and JS-ABI runtime helpers), excluding
/// `RuntimeErrorObject`. Prepared-script bodies are also eligible even though
/// they use ten parameters. Non-JS allocator/mutation helpers and the main
/// body are not part of this JS relocation map; main has its own `() -> i64`
/// entry wrapper and is relocated after these bodies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StackGuardPlan {
    original_defined_function_count: u32,
    guarded_functions: Vec<GuardedFunction>,
}

/// Function-index data embedded in the Wasm artifact for the engine's host
/// stack-budget callback. Indices are relative to the defined-function space
/// and remain stable across imported-function changes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StackGuardMetadata {
    pub(crate) original_defined_function_count: u32,
    /// Relocated `main` body after all guarded JS bodies. The wrapper stays at
    /// defined index zero and uses `(startup_body, 0)` in the host callback.
    pub(crate) startup_body: DefinedFunctionIndex,
    pub(crate) guarded_functions: Vec<GuardedFunction>,
    /// Every emitted helper that is deliberately left without a JS guard:
    /// `RuntimeErrorObject` and non-JS-ABI allocation/mutation helpers.
    pub(crate) unguarded_helper_indices: Vec<DefinedFunctionIndex>,
}

impl StackGuardMetadata {
    /// Encode versioned artifact metadata. The engine parses this before
    /// instantiation and cross-checks every function layout against Wasmtime.
    pub(crate) fn encode(&self) -> Vec<u8> {
        let guarded_count = u32::try_from(self.guarded_functions.len())
            .expect("validated stack-guard plan fits the Wasm function index space");
        let helper_count = u32::try_from(self.unguarded_helper_indices.len())
            .expect("emitted helper count fits the Wasm function index space");
        let mut bytes = Vec::with_capacity(
            STACK_GUARD_METADATA_MAGIC.len()
                + 20
                + guarded_count as usize * 8
                + helper_count as usize * 4,
        );
        bytes.extend_from_slice(STACK_GUARD_METADATA_MAGIC);
        bytes.extend_from_slice(&STACK_GUARD_METADATA_VERSION.to_le_bytes());
        bytes.extend_from_slice(&self.original_defined_function_count.to_le_bytes());
        bytes.extend_from_slice(&self.startup_body.get().to_le_bytes());
        bytes.extend_from_slice(&guarded_count.to_le_bytes());
        for guarded in &self.guarded_functions {
            bytes.extend_from_slice(&guarded.wrapper().get().to_le_bytes());
            bytes.extend_from_slice(&guarded.body().get().to_le_bytes());
        }
        bytes.extend_from_slice(&helper_count.to_le_bytes());
        for helper in &self.unguarded_helper_indices {
            bytes.extend_from_slice(&helper.get().to_le_bytes());
        }
        bytes
    }
}

impl StackGuardPlan {
    /// Build a deterministic relocation map from the original defined-function
    /// count and the selected original indices.
    pub(crate) fn new(
        original_defined_function_count: u32,
        guarded_indices: impl IntoIterator<Item = DefinedFunctionIndex>,
    ) -> Result<Self, StackGuardPlanError> {
        let mut guarded_indices: Vec<_> = guarded_indices.into_iter().collect();
        guarded_indices.sort_unstable();

        if guarded_indices
            .iter()
            .any(|index| index.get() >= original_defined_function_count)
        {
            return Err(StackGuardPlanError::GuardIndexOutOfRange);
        }
        if guarded_indices.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(StackGuardPlanError::DuplicateGuardIndex);
        }

        let appended_count = u32::try_from(guarded_indices.len())
            .map_err(|_| StackGuardPlanError::FunctionIndexSpaceOverflow)?;
        original_defined_function_count
            .checked_add(appended_count)
            .ok_or(StackGuardPlanError::FunctionIndexSpaceOverflow)?;

        let guarded_functions = guarded_indices
            .into_iter()
            .enumerate()
            .map(|(ordinal, wrapper)| {
                let body = original_defined_function_count
                    + u32::try_from(ordinal).expect("guarded function count was validated as u32");
                GuardedFunction {
                    wrapper,
                    body: DefinedFunctionIndex::new(body),
                }
            })
            .collect();

        Ok(Self {
            original_defined_function_count,
            guarded_functions,
        })
    }

    pub(crate) const fn original_defined_function_count(&self) -> u32 {
        self.original_defined_function_count
    }

    pub(crate) fn guarded_functions(&self) -> &[GuardedFunction] {
        &self.guarded_functions
    }

    pub(crate) fn appended_body_count(&self) -> u32 {
        u32::try_from(self.guarded_functions.len())
            .expect("guarded function count was validated as u32")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StackGuardPlanError {
    GuardIndexOutOfRange,
    DuplicateGuardIndex,
    FunctionIndexSpaceOverflow,
    LocalIndexOverflow,
    UnsupportedDefaultParameterType,
}

impl core::fmt::Display for StackGuardPlanError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::GuardIndexOutOfRange => {
                f.write_str("stack-guard function index is outside the original function space")
            }
            Self::DuplicateGuardIndex => {
                f.write_str("stack-guard plan contains a duplicate function index")
            }
            Self::FunctionIndexSpaceOverflow => {
                f.write_str("stack-guard relocation exceeds the Wasm function index space")
            }
            Self::LocalIndexOverflow => f.write_str("stack-guard wrapper local indices exceed u32"),
            Self::UnsupportedDefaultParameterType => f.write_str(
                "stack-guard error constructor needs a default for its Wasm parameter type",
            ),
        }
    }
}

impl std::error::Error for StackGuardPlanError {}

/// Emit a wrapper for a JS-ABI function body.
///
/// `can_enter_import_index` names the leaf `(i32 target, i32 wrapper) -> i32`
/// import. A nonzero result permits entry. `runtime_error_object_wasm_index`
/// names the unguarded `RuntimeErrorObject` helper. The wrapper resolves the
/// target function's Realm into a local before checking the budget;
/// runtime-helper wrappers instead copy the already-active Realm because their
/// first argument is not a function environment. The rejection branch uses
/// that Realm to write its `%RangeError.prototype%` payload. These sequences
/// contain no user callbacks.
/// The ABI supplies the parameter/result types and local-index boundary.
pub(crate) fn emit_guarded_wrapper(
    guarded: GuardedFunction,
    imported_function_count: u32,
    can_enter_import_index: u32,
    runtime_error_object_wasm_index: u32,
    abi: CallAbi,
    range_error_name_payload: i64,
    range_error_message_payload: i64,
    realm_source: FunctionRealmSource,
) -> Result<Function, StackGuardPlanError> {
    let parameter_count = abi.parameter_count() as u32;
    let result_types = abi.result_types();
    let prototype_local = parameter_count;
    let first_result_local = prototype_local
        .checked_add(1)
        .ok_or(StackGuardPlanError::LocalIndexOverflow)?;
    let target_realm_local = first_result_local
        .checked_add(result_types.len() as u32)
        .ok_or(StackGuardPlanError::LocalIndexOverflow)?;
    let body_wasm_index = guarded
        .body()
        .absolute_wasm_index(imported_function_count)
        .ok_or(StackGuardPlanError::FunctionIndexSpaceOverflow)?;

    let mut local_types = Vec::with_capacity(result_types.len() + 2);
    local_types.push(ValType::I64); // RangeError prototype
    local_types.extend(result_types);
    local_types.push(ValType::I64); // active Realm
    let mut function = Function::new_with_locals_types(local_types);

    // Resolve the target Realm before the callback so a rejected call creates
    // its RangeError in that Realm. The prelude is Wasm-only and its exact
    // native frame is included in the compiled wrapper bound.
    emit_target_function_realm(
        &mut function,
        realm_source,
        prototype_local,
        target_realm_local,
    );
    function.instruction(&Instruction::I32Const(guarded.body().get() as i32));
    function.instruction(&Instruction::I32Const(guarded.wrapper().get() as i32));
    function.instruction(&Instruction::Call(can_enter_import_index));
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));

    emit_range_error_prototype(&mut function, prototype_local, target_realm_local);
    function.instruction(&Instruction::LocalGet(prototype_local));
    function.instruction(&Instruction::I64Const(range_error_name_payload));
    function.instruction(&Instruction::I64Const(range_error_message_payload));
    for parameter_type in CallAbi::Raw.parameter_types().into_iter().skip(3) {
        emit_default_argument(&mut function, parameter_type)?;
    }
    function.instruction(&Instruction::Call(runtime_error_object_wasm_index));
    store_call_results(&mut function, first_result_local);
    // RuntimeErrorObject returns a Normal completion for its newly allocated
    // object. The guard publishes that object through the standard Throw tuple.
    function.instruction(&Instruction::I64Const(crate::abi::COMPLETION_KIND_THROW));
    function.instruction(&Instruction::LocalSet(
        first_result_local + CallResultSlot::Completion.index(),
    ));

    function.instruction(&Instruction::Else);
    function.instruction(&Instruction::LocalGet(target_realm_local));
    function.instruction(&Instruction::GlobalSet(
        crate::module::STACK_GUARD_ACTIVE_REALM_GLOBAL_INDEX,
    ));
    for parameter in 0..parameter_count {
        function.instruction(&Instruction::LocalGet(parameter));
    }
    // A successful guard must tail-call the real body. Otherwise the wrapper
    // frame remains below a proper tail call from that body, growing once per
    // tail transfer and eventually trapping despite tail-call lowering.
    function.instruction(&Instruction::ReturnCall(body_wasm_index));
    function.instruction(&Instruction::End);

    for result in CallResultSlot::ALL {
        function.instruction(&Instruction::LocalGet(first_result_local + result.index()));
    }
    function.instruction(&Instruction::End);
    Ok(function)
}

fn emit_default_argument(
    function: &mut Function,
    parameter_type: ValType,
) -> Result<(), StackGuardPlanError> {
    match parameter_type {
        ValType::I64 => {
            function.instruction(&Instruction::I64Const(0));
        }
        ValType::Ref(reference) => {
            if !reference.nullable {
                return Err(StackGuardPlanError::UnsupportedDefaultParameterType);
            }
            function.instruction(&Instruction::RefNull(reference.heap_type));
        }
        ValType::I32 | ValType::F32 | ValType::F64 | ValType::V128 => {
            return Err(StackGuardPlanError::UnsupportedDefaultParameterType);
        }
    }
    Ok(())
}

/// Emit the exported `main` entry wrapper. It checks the compiled entry body
/// before that body's native frame is allocated. A denied startup is surfaced
/// by the host callback as a resource error because the JS Realm and its
/// error-construction helpers are not initialized yet.
pub(crate) fn emit_main_entry_wrapper(
    startup_body: DefinedFunctionIndex,
    imported_function_count: u32,
    can_enter_import_index: u32,
) -> Result<Function, StackGuardPlanError> {
    let body_wasm_index = startup_body
        .absolute_wasm_index(imported_function_count)
        .ok_or(StackGuardPlanError::FunctionIndexSpaceOverflow)?;
    let mut function = Function::new_with_locals_types([]);
    function.instruction(&Instruction::I32Const(startup_body.get() as i32));
    function.instruction(&Instruction::I32Const(0));
    function.instruction(&Instruction::Call(can_enter_import_index));
    function.instruction(&Instruction::I32Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Unreachable);
    function.instruction(&Instruction::End);
    function.instruction(&Instruction::ReturnCall(body_wasm_index));
    function.instruction(&Instruction::End);
    Ok(function)
}

fn emit_target_function_realm(
    function: &mut Function,
    source: FunctionRealmSource,
    environment_local: u32,
    realm_local: u32,
) {
    match source {
        FunctionRealmSource::Active => {
            emit_active_realm_or_current(function, realm_local);
        }
        FunctionRealmSource::FunctionEnvironment => {
            emit_function_object_realm(function, realm_local);
        }
        FunctionRealmSource::PreparedScript => {
            // PreparedScript's parameter zero is a lexical environment, not a
            // function object. `emit_prepared_script_dispatch` first switches
            // CURRENT_REALM to the target Realm, then calls the thunk with that
            // Realm's environment; it restores CURRENT_REALM after the thunk
            // returns. Reading function-object metadata from param zero makes
            // the wrapper interpret an environment record as a function and
            // trap before any prepared script can run.
            function.instruction(&Instruction::GlobalGet(
                crate::module::CURRENT_REALM_GLOBAL_INDEX,
            ));
            function.instruction(&Instruction::LocalSet(realm_local));
        }
        FunctionRealmSource::LexicalEnvironment {
            has_function_context,
            missing_environment,
        } => {
            function.instruction(&Instruction::LocalGet(0));
            function.instruction(&Instruction::LocalSet(environment_local));
            function.instruction(&Instruction::LocalGet(environment_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            match missing_environment {
                MissingEnvironmentRealmSource::Trap => {
                    function.instruction(&Instruction::Unreachable);
                }
                MissingEnvironmentRealmSource::ActiveOrCurrent => {
                    emit_active_realm_or_current(function, realm_local);
                }
            }
            function.instruction(&Instruction::Else);
            if has_function_context {
                load_i64_from_local_offset(
                    function,
                    environment_local,
                    HEAP_CLASS_FUNCTION_CONTEXT_LEXICAL_ENV_OFFSET,
                    environment_local,
                );
            }
            // Lexical environments have an acyclic parent chain ending at the
            // Global Environment Record. Walk it iteratively: this pre-callback
            // wrapper sequence cannot add native frames, regardless of lexical
            // nesting depth.
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            function.instruction(&Instruction::LocalGet(environment_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Unreachable);
            function.instruction(&Instruction::End);
            load_i64_from_local_offset(function, environment_local, ENV_PARENT_OFFSET, realm_local);
            function.instruction(&Instruction::LocalGet(realm_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::BrIf(1));
            function.instruction(&Instruction::LocalGet(realm_local));
            function.instruction(&Instruction::LocalSet(environment_local));
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            load_i64_from_local_offset(
                function,
                environment_local,
                GLOBAL_ENV_REALM_OFFSET,
                realm_local,
            );
            function.instruction(&Instruction::End);
        }
    }
    function.instruction(&Instruction::LocalGet(realm_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Unreachable);
    function.instruction(&Instruction::End);
}

fn emit_function_object_realm(function: &mut Function, realm_local: u32) {
    function.instruction(&Instruction::LocalGet(0));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    emit_active_realm_or_current(function, realm_local);
    function.instruction(&Instruction::Else);
    load_i64_from_local_offset(
        function,
        0,
        HEAP_FUNCTION_DEFINING_REALM_OFFSET,
        realm_local,
    );
    function.instruction(&Instruction::End);
}

fn emit_active_realm_or_current(function: &mut Function, realm_local: u32) {
    function.instruction(&Instruction::GlobalGet(
        crate::module::STACK_GUARD_ACTIVE_REALM_GLOBAL_INDEX,
    ));
    function.instruction(&Instruction::LocalSet(realm_local));
    function.instruction(&Instruction::LocalGet(realm_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::GlobalGet(
        crate::module::CURRENT_REALM_GLOBAL_INDEX,
    ));
    function.instruction(&Instruction::LocalSet(realm_local));
    function.instruction(&Instruction::End);
}

fn emit_range_error_prototype(function: &mut Function, prototype_local: u32, realm_local: u32) {
    load_i64_from_local_offset(
        function,
        realm_local,
        HEAP_REALM_INTRINSICS_OFFSET,
        prototype_local,
    );
    function.instruction(&Instruction::LocalGet(prototype_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Unreachable);
    function.instruction(&Instruction::End);
    load_i64_from_local_offset(
        function,
        prototype_local,
        HEAP_REALM_INTRINSICS_RANGE_ERROR_PROTOTYPE_OFFSET,
        prototype_local,
    );
    function.instruction(&Instruction::LocalGet(prototype_local));
    function.instruction(&Instruction::I64Eqz);
    function.instruction(&Instruction::If(BlockType::Empty));
    function.instruction(&Instruction::Unreachable);
    function.instruction(&Instruction::End);
}

fn load_i64_from_local_offset(
    function: &mut Function,
    source_local: u32,
    offset: u64,
    target_local: u32,
) {
    function.instruction(&Instruction::LocalGet(source_local));
    function.instruction(&Instruction::I32WrapI64);
    function.instruction(&Instruction::I64Load(wasm_encoder::MemArg {
        offset,
        align: 3,
        memory_index: 0,
    }));
    function.instruction(&Instruction::LocalSet(target_local));
}

fn store_call_results(function: &mut Function, first_result_local: u32) {
    for result in CallResultSlot::ALL.into_iter().rev() {
        function.instruction(&Instruction::LocalSet(first_result_local + result.index()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_encoder::{
        CodeSection, ConstExpr, EntityType, ExportKind, ExportSection, FunctionSection,
        GlobalSection, GlobalType, ImportSection, MemorySection, MemoryType, Module, TypeSection,
    };

    fn has_tail_call_to(function: Function, expected_index: u32) -> bool {
        let raw_body = function
            .into_body_named(&"stack_guard_test")
            .into_raw_body();
        let body = wasmparser::FunctionBody::new(wasmparser::BinaryReader::new(&raw_body, 0));
        let operators = body
            .get_operators_reader()
            .expect("guard wrapper should parse")
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .expect("guard wrapper operators should parse");
        operators.iter().any(|operator| matches!(
            operator,
            wasmparser::Operator::ReturnCall { function_index } if *function_index == expected_index
        ))
    }

    #[test]
    fn accepted_function_and_startup_wrappers_tail_call_their_bodies() {
        let guarded = GuardedFunction {
            wrapper: DefinedFunctionIndex::new(2),
            body: DefinedFunctionIndex::new(8),
        };
        let wrapper = emit_guarded_wrapper(
            guarded,
            3,
            0,
            4,
            CallAbi::Js,
            0,
            0,
            FunctionRealmSource::Active,
        )
        .expect("guard wrapper should emit");
        assert!(has_tail_call_to(wrapper, 11));

        let startup = emit_main_entry_wrapper(DefinedFunctionIndex::new(12), 3, 0)
            .expect("startup wrapper should emit");
        assert!(has_tail_call_to(startup, 15));
    }

    #[test]
    fn main_wrapper_validates_with_a_tail_call_to_the_startup_body() {
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.ty().function([], [ValType::I64]);
        types
            .ty()
            .function([ValType::I32, ValType::I32], [ValType::I32]);
        module.section(&types);

        let mut imports = ImportSection::new();
        imports.import(
            "lila_host",
            "stack_guard_can_enter",
            EntityType::Function(1),
        );
        module.section(&imports);

        let mut functions = FunctionSection::new();
        functions.function(0); // wrapper at defined index zero
        functions.function(0); // relocated startup body
        module.section(&functions);

        let mut exports = ExportSection::new();
        exports.export("main", ExportKind::Func, 1);
        module.section(&exports);

        let mut code = CodeSection::new();
        let wrapper = emit_main_entry_wrapper(DefinedFunctionIndex::new(1), 1, 0)
            .expect("main wrapper should emit");
        code.raw(&wrapper.into_body_named(&"main-wrapper").into_raw_body());
        let mut body = Function::new_with_locals_types([]);
        body.instruction(&Instruction::I64Const(42));
        body.instruction(&Instruction::End);
        code.raw(&body.into_body_named(&"startup-body").into_raw_body());
        module.section(&code);

        wasmparser::Validator::new()
            .validate_all(&module.finish())
            .expect("the main wrapper and i64 startup body must validate together");
    }

    #[test]
    fn lexical_function_wrappers_validate_with_both_zero_environment_policies() {
        for has_function_context in [false, true] {
            for missing_environment in [
                MissingEnvironmentRealmSource::Trap,
                MissingEnvironmentRealmSource::ActiveOrCurrent,
            ] {
                let mut module = Module::new();
                let mut types = TypeSection::new();
                types
                    .ty()
                    .function([ValType::I32, ValType::I32], [ValType::I32]);
                crate::gc_types::arg_vector::register(&mut types);
                types
                    .ty()
                    .function(CallAbi::Raw.parameter_types(), CallAbi::Raw.result_types());
                types
                    .ty()
                    .function(CallAbi::Js.parameter_types(), CallAbi::Js.result_types());
                module.section(&types);

                let mut imports = ImportSection::new();
                imports.import(
                    "lila_host",
                    "stack_guard_can_enter",
                    EntityType::Function(0),
                );
                imports.import(
                    "lila_runtime",
                    "runtime_error_object",
                    EntityType::Function(2),
                );
                module.section(&imports);

                let mut functions = FunctionSection::new();
                functions.function(3); // stack-guard wrapper
                functions.function(3); // relocated body
                module.section(&functions);

                let mut memory = MemorySection::new();
                memory.memory(MemoryType {
                    minimum: 1,
                    maximum: None,
                    memory64: false,
                    shared: false,
                    page_size_log2: None,
                });
                module.section(&memory);

                let highest_guard_global = crate::module::STACK_GUARD_ACTIVE_REALM_GLOBAL_INDEX
                    .max(crate::module::CURRENT_REALM_GLOBAL_INDEX);
                let mut globals = GlobalSection::new();
                for _ in 0..=highest_guard_global {
                    globals.global(
                        GlobalType {
                            val_type: ValType::I64,
                            mutable: true,
                            shared: false,
                        },
                        &ConstExpr::i64_const(0),
                    );
                }
                module.section(&globals);

                let guarded = GuardedFunction {
                    wrapper: DefinedFunctionIndex::new(0),
                    body: DefinedFunctionIndex::new(1),
                };
                let wrapper = emit_guarded_wrapper(
                    guarded,
                    2,
                    0,
                    1,
                    CallAbi::Js,
                    0,
                    0,
                    FunctionRealmSource::LexicalEnvironment {
                        has_function_context,
                        missing_environment,
                    },
                )
                .expect("lexical wrapper should emit");
                let mut body = Function::new_with_locals_types([]);
                for _ in 0..4 {
                    body.instruction(&Instruction::I64Const(0));
                }
                body.instruction(&Instruction::End);
                let mut code = CodeSection::new();
                code.raw(&wrapper.into_body_named(&"lexical-wrapper").into_raw_body());
                code.raw(&body.into_body_named(&"lexical-body").into_raw_body());
                module.section(&code);

                wasmparser::Validator::new()
                    .validate_all(&module.finish())
                    .expect("lexical wrapper branches must validate together");
            }
        }
    }

    #[test]
    fn prepared_script_wrapper_uses_current_realm_and_validates_ten_parameter_abi() {
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types
            .ty()
            .function([ValType::I32, ValType::I32], [ValType::I32]);
        crate::gc_types::arg_vector::register(&mut types);
        types
            .ty()
            .function(CallAbi::Raw.parameter_types(), CallAbi::Raw.result_types());
        types.ty().function(
            CallAbi::PreparedScript.parameter_types(),
            CallAbi::PreparedScript.result_types(),
        );
        module.section(&types);

        let mut imports = ImportSection::new();
        imports.import(
            "lila_host",
            "stack_guard_can_enter",
            EntityType::Function(0),
        );
        imports.import(
            "lila_runtime",
            "runtime_error_object",
            EntityType::Function(2),
        );
        module.section(&imports);

        let mut functions = FunctionSection::new();
        functions.function(3); // ten-parameter wrapper at defined index zero
        functions.function(3); // relocated prepared-script body
        module.section(&functions);

        let mut memory = MemorySection::new();
        memory.memory(MemoryType {
            minimum: 1,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        });
        module.section(&memory);

        let highest_guard_global = crate::module::STACK_GUARD_ACTIVE_REALM_GLOBAL_INDEX
            .max(crate::module::CURRENT_REALM_GLOBAL_INDEX);
        let mut globals = GlobalSection::new();
        for _ in 0..=highest_guard_global {
            globals.global(
                GlobalType {
                    val_type: ValType::I64,
                    mutable: true,
                    shared: false,
                },
                &ConstExpr::i64_const(0),
            );
        }
        module.section(&globals);

        let guarded = GuardedFunction {
            wrapper: DefinedFunctionIndex::new(0),
            body: DefinedFunctionIndex::new(1),
        };
        let wrapper = emit_guarded_wrapper(
            guarded,
            2,
            0,
            1,
            CallAbi::PreparedScript,
            0,
            0,
            FunctionRealmSource::PreparedScript,
        )
        .expect("prepared-script wrapper should emit");
        let raw_wrapper = wrapper
            .into_body_named(&"prepared-script-wrapper")
            .into_raw_body();
        let body = wasmparser::FunctionBody::new(wasmparser::BinaryReader::new(&raw_wrapper, 0));
        let operators = body
            .get_operators_reader()
            .expect("prepared-script wrapper operators should parse")
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .expect("prepared-script wrapper operators should be valid");
        assert!(matches!(
            operators.first(),
            Some(wasmparser::Operator::GlobalGet { global_index })
                if *global_index == crate::module::CURRENT_REALM_GLOBAL_INDEX
        ));

        let mut prepared_body = Function::new_with_locals_types([]);
        for _ in 0..4 {
            prepared_body.instruction(&Instruction::I64Const(0));
        }
        prepared_body.instruction(&Instruction::End);
        let mut code = CodeSection::new();
        code.raw(&raw_wrapper);
        code.raw(
            &prepared_body
                .into_body_named(&"prepared-script-body")
                .into_raw_body(),
        );
        module.section(&code);

        wasmparser::Validator::new()
            .validate_all(&module.finish())
            .expect("prepared-script wrappers must forward their real ten-param ABI");
    }
}
