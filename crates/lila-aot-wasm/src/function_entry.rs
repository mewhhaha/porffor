//! The executable entry owned by one planned callable, and its actual inputs.
//!
//! Each declared role has one semantic input bundle and typed callable reference.
//! Activations never occupy a JavaScript value slot.

use super::*;
use crate::emit::{ControlFrameKind, FunctionBuilder};
use crate::emitted_function::FunctionIdentity;
use crate::module::StaticSignature;

mod body_entry_locals;
mod call_inputs;
mod executable_code;
mod runtime_dispatch;
mod saved_activation_call;
pub(crate) use body_entry_locals::{BodyEntryLocals, ResumableEntryLocals};
pub(crate) use call_inputs::{
    AsyncBodyInputs, AsyncGeneratorBodyInputs, EntryArguments, GeneratorBodyInputs,
    OrdinaryBodyInputs, PreparedScriptInputs,
};
pub(crate) use runtime_dispatch::{
    RuntimeAsyncBodyEntry, RuntimeAsyncGeneratorBodyEntry, RuntimeFunctionEntryDispatch,
    RuntimeGeneratorBodyEntry, RuntimeOrdinaryBodyEntry,
};

#[derive(Debug, Clone)]
enum EntryOrigin {
    User {
        id: FunctionId,
        name: String,
        protocol: FunctionProtocolIr,
    },
    PreparedScript {
        id: FunctionId,
        kind: PreparedScriptKind,
    },
    StandardBuiltin(StandardBuiltinId),
    HostBuiltin(HostBuiltinId),
    EmptyOrdinaryFunction,
}

/// The planner owns the protocol and declared Wasm entry together. Its index
/// is private; code references and calls pass through the matching body role.
#[derive(Debug, Clone)]
pub(crate) struct PlannedFunctionEntry {
    origin: EntryOrigin,
    wasm_index: u32,
}

impl PlannedFunctionEntry {
    pub(crate) fn user(source: &FunctionIr, wasm_index: u32) -> Self {
        Self {
            origin: EntryOrigin::User {
                id: source.id.clone(),
                name: source.name.clone(),
                protocol: source.protocol,
            },
            wasm_index,
        }
    }

    pub(crate) fn prepared(source: &PreparedScriptUnit, wasm_index: u32) -> Self {
        Self {
            origin: EntryOrigin::PreparedScript {
                id: source.id.function_id(),
                kind: source.kind.clone(),
            },
            wasm_index,
        }
    }

    pub(crate) fn standard(builtin: StandardBuiltinId, wasm_index: u32) -> Self {
        Self {
            origin: EntryOrigin::StandardBuiltin(builtin),
            wasm_index,
        }
    }

    pub(crate) fn host(builtin: HostBuiltinId, wasm_index: u32) -> Self {
        Self {
            origin: EntryOrigin::HostBuiltin(builtin),
            wasm_index,
        }
    }

    /// The zero-source Function constructor shares Function.prototype's
    /// ordinary body, while its newly materialized callable is constructable.
    pub(crate) fn empty_ordinary_function(&self) -> Result<Self, EmitError> {
        match self.origin {
            EntryOrigin::StandardBuiltin(StandardBuiltinId::FunctionPrototype) => Ok(Self {
                origin: EntryOrigin::EmptyOrdinaryFunction,
                ..self.clone()
            }),
            EntryOrigin::User { .. }
            | EntryOrigin::PreparedScript { .. }
            | EntryOrigin::StandardBuiltin(_)
            | EntryOrigin::HostBuiltin(_)
            | EntryOrigin::EmptyOrdinaryFunction => Err(entry_error(
                "empty Function allocation shares only Function.prototype's body",
            )),
        }
    }

    pub(crate) fn emit_ref_func(&self, function: &mut Function) {
        function.instruction(&Instruction::RefFunc(self.wasm_index));
    }

    pub(crate) fn emit_direct_call_instruction(&self, function: &mut Function) {
        function.instruction(&Instruction::Call(self.wasm_index));
    }

    pub(crate) fn assert_body_index(&self, wasm_index: u32) {
        assert_eq!(
            self.wasm_index, wasm_index,
            "planned entry and emitted body occupy the same function index"
        );
    }

    /// Existing lexical/construct metadata is projected from its one owner.
    /// Prepared Script metadata is ordinary for lexical lookup only; its
    /// executable body projection below is always the separate Script role.
    pub(crate) fn protocol(&self) -> FunctionProtocolIr {
        match &self.origin {
            EntryOrigin::User { protocol, .. } => *protocol,
            EntryOrigin::PreparedScript { .. } => FunctionProtocolIr::OrdinaryCallOnly,
            EntryOrigin::EmptyOrdinaryFunction => FunctionProtocolIr::OrdinaryCallAndConstruct,
            EntryOrigin::StandardBuiltin(builtin) => {
                if builtin.constructable() {
                    FunctionProtocolIr::OrdinaryCallAndConstruct
                } else {
                    FunctionProtocolIr::OrdinaryCallOnly
                }
            }
            EntryOrigin::HostBuiltin(builtin) => {
                if DynamicSourceIntrinsic::from_function_id(&builtin.function_id())
                    .is_some_and(DynamicSourceIntrinsic::constructable)
                {
                    FunctionProtocolIr::OrdinaryCallAndConstruct
                } else {
                    FunctionProtocolIr::OrdinaryCallOnly
                }
            }
        }
    }

    pub(crate) fn body(&self) -> PlannedBodyEntry<'_> {
        match &self.origin {
            EntryOrigin::PreparedScript { kind, .. } => {
                PlannedBodyEntry::PreparedScript(PreparedScriptEntry(self, kind))
            }
            EntryOrigin::User { protocol, .. } => match protocol.execution_kind() {
                FunctionExecutionKind::Ordinary => {
                    PlannedBodyEntry::Ordinary(OrdinaryBodyEntry(self))
                }
                FunctionExecutionKind::Generator => {
                    PlannedBodyEntry::Generator(GeneratorBodyEntry(self))
                }
                FunctionExecutionKind::Async => PlannedBodyEntry::Async(AsyncBodyEntry(self)),
                FunctionExecutionKind::AsyncGenerator => {
                    PlannedBodyEntry::AsyncGenerator(AsyncGeneratorBodyEntry(self))
                }
            },
            EntryOrigin::StandardBuiltin(_)
            | EntryOrigin::HostBuiltin(_)
            | EntryOrigin::EmptyOrdinaryFunction => {
                PlannedBodyEntry::Ordinary(OrdinaryBodyEntry(self))
            }
        }
    }

    pub(crate) fn ordinary(&self) -> Result<OrdinaryBodyEntry<'_>, EmitError> {
        match self.body() {
            PlannedBodyEntry::Ordinary(entry) => Ok(entry),
            PlannedBodyEntry::Generator(_)
            | PlannedBodyEntry::Async(_)
            | PlannedBodyEntry::AsyncGenerator(_)
            | PlannedBodyEntry::PreparedScript(_) => Err(entry_error(
                "ordinary body call requires an ordinary callable entry",
            )),
        }
    }

    pub(crate) fn asynchronous(&self) -> Result<AsyncBodyEntry<'_>, EmitError> {
        match self.body() {
            PlannedBodyEntry::Async(entry) => Ok(entry),
            PlannedBodyEntry::Ordinary(_)
            | PlannedBodyEntry::Generator(_)
            | PlannedBodyEntry::AsyncGenerator(_)
            | PlannedBodyEntry::PreparedScript(_) => Err(entry_error(
                "async activation allocation requires an async entry",
            )),
        }
    }

    pub(crate) fn prepared_script(&self) -> Result<PreparedScriptEntry<'_>, EmitError> {
        match self.body() {
            PlannedBodyEntry::PreparedScript(entry) => Ok(entry),
            PlannedBodyEntry::Ordinary(_)
            | PlannedBodyEntry::Generator(_)
            | PlannedBodyEntry::Async(_)
            | PlannedBodyEntry::AsyncGenerator(_) => Err(entry_error(
                "prepared Script call requires a prepared Script entry",
            )),
        }
    }

    pub(crate) fn identity(&self) -> FunctionIdentity {
        match &self.origin {
            EntryOrigin::User { id, name, .. } => FunctionIdentity::Script {
                id: id.clone(),
                name: name.clone(),
            },
            EntryOrigin::PreparedScript { id, .. } => FunctionIdentity::PreparedScript {
                id: id.clone(),
                name: id.clone(),
            },
            EntryOrigin::StandardBuiltin(id) => FunctionIdentity::StandardBuiltin(*id),
            EntryOrigin::HostBuiltin(id) => FunctionIdentity::HostBuiltin(*id),
            EntryOrigin::EmptyOrdinaryFunction => {
                FunctionIdentity::StandardBuiltin(StandardBuiltinId::FunctionPrototype)
            }
        }
    }

    pub(crate) fn signature(&self) -> StaticSignature {
        match self.body() {
            PlannedBodyEntry::PreparedScript(_) => StaticSignature::PreparedScript,
            PlannedBodyEntry::Ordinary(_) => StaticSignature::JavaScriptFunction,
            PlannedBodyEntry::Generator(_) => StaticSignature::GeneratorFunction,
            PlannedBodyEntry::Async(_) => StaticSignature::AsyncFunction,
            PlannedBodyEntry::AsyncGenerator(_) => StaticSignature::AsyncGeneratorFunction,
        }
    }
}

fn entry_error(message: &'static str) -> EmitError {
    EmitError::unsupported(format!("compiler entry-role invariant: {message}"))
}

pub(crate) enum PlannedBodyEntry<'a> {
    Ordinary(OrdinaryBodyEntry<'a>),
    Generator(GeneratorBodyEntry<'a>),
    Async(AsyncBodyEntry<'a>),
    AsyncGenerator(AsyncGeneratorBodyEntry<'a>),
    PreparedScript(PreparedScriptEntry<'a>),
}

pub(crate) struct OrdinaryBodyEntry<'a>(&'a PlannedFunctionEntry);
pub(crate) struct GeneratorBodyEntry<'a>(&'a PlannedFunctionEntry);
pub(crate) struct AsyncBodyEntry<'a>(&'a PlannedFunctionEntry);
pub(crate) struct AsyncGeneratorBodyEntry<'a>(&'a PlannedFunctionEntry);
pub(crate) struct PreparedScriptEntry<'a>(&'a PlannedFunctionEntry, &'a PreparedScriptKind);
