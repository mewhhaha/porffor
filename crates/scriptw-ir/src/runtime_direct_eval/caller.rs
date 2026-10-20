use crate::direct_eval_context::ScriptImportOrigin;
use crate::{DirectEvalContextIr, DirectEvalDerivedOwner, PrivateNameId, Strictness};
use scriptw_front::EvalInvocationContext;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDirectEvalDerivedOwner {
    Absent,
    ExternalCaller,
}

/// These are facts about the originating caller, never P2-local graph IDs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDirectEvalImportOrigin {
    Absent,
    CallerScriptDiscovery,
    CallerScriptRecord(crate::ModuleUnitId),
}

/// Immutable compiler-produced grammar and lexical-private-name selectors.
/// Decoding alone grants no private-environment or activation authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDirectEvalCallerDescriptor {
    pub(super) strictness: Strictness,
    pub(super) invocation: EvalInvocationContext,
    pub(super) private_names: BTreeMap<String, PrivateNameId>,
    pub(super) derived_owner: RuntimeDirectEvalDerivedOwner,
    pub(super) import_origin: RuntimeDirectEvalImportOrigin,
}
impl RuntimeDirectEvalCallerDescriptor {
    pub const fn strictness(&self) -> Strictness {
        self.strictness
    }
    pub const fn invocation(&self) -> EvalInvocationContext {
        self.invocation
    }
    pub fn private_names(&self) -> &BTreeMap<String, PrivateNameId> {
        &self.private_names
    }
    pub const fn derived_owner(&self) -> RuntimeDirectEvalDerivedOwner {
        self.derived_owner
    }
    pub const fn import_origin(&self) -> RuntimeDirectEvalImportOrigin {
        self.import_origin
    }
}

pub(crate) fn descriptor(context: &DirectEvalContextIr) -> RuntimeDirectEvalCallerDescriptor {
    RuntimeDirectEvalCallerDescriptor {
        strictness: context.strictness(),
        invocation: context.invocation(),
        private_names: context.private_names().clone(),
        derived_owner: match context.derived_owner() {
            Some(
                DirectEvalDerivedOwner::LocalFunction(_) | DirectEvalDerivedOwner::ExternalCaller,
            ) => RuntimeDirectEvalDerivedOwner::ExternalCaller,
            None => RuntimeDirectEvalDerivedOwner::Absent,
        },
        import_origin: match context.script_import_origin() {
            None => RuntimeDirectEvalImportOrigin::Absent,
            Some(ScriptImportOrigin::Discovery) => {
                RuntimeDirectEvalImportOrigin::CallerScriptDiscovery
            }
            Some(ScriptImportOrigin::LoadedRecord(record)) => {
                RuntimeDirectEvalImportOrigin::CallerScriptRecord(record)
            }
            Some(ScriptImportOrigin::ExternalCaller(origin)) => origin,
        },
    }
}

/// One actual immutable native-decoded runtime PrivateEnvironment table.
/// Supply every chain row, including scopes whose names are lexically hidden.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeDirectEvalPrivateScope {
    class_scope: u32,
    slot_count: u32,
}
impl RuntimeDirectEvalPrivateScope {
    pub const fn new(class_scope: u32, slot_count: u32) -> Self {
        Self {
            class_scope,
            slot_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDirectEvalCallerError(pub(crate) &'static str);
impl core::fmt::Display for RuntimeDirectEvalCallerError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(self.0)
    }
}
impl std::error::Error for RuntimeDirectEvalCallerError {}

/// Only this validated combination can enter independent direct-eval lowering.
/// Runtime GC references stay owned and rooted by the native request object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeDirectEvalCaller {
    descriptor: RuntimeDirectEvalCallerDescriptor,
    context: DirectEvalContextIr,
    private_environment_floor: u32,
}
impl RuntimeDirectEvalCaller {
    pub fn validate(
        descriptor: RuntimeDirectEvalCallerDescriptor,
        scopes: impl IntoIterator<Item = RuntimeDirectEvalPrivateScope>,
    ) -> Result<Self, RuntimeDirectEvalCallerError> {
        let mut tables = BTreeMap::new();
        let mut private_environment_floor = 0;
        for scope in scopes {
            if tables.insert(scope.class_scope, scope.slot_count).is_some() {
                return Err(RuntimeDirectEvalCallerError(
                    "duplicate caller private scope",
                ));
            }
            let next = scope
                .class_scope
                .checked_add(1)
                .ok_or(RuntimeDirectEvalCallerError(
                    "caller private-scope identity space is exhausted",
                ))?;
            private_environment_floor = private_environment_floor.max(next);
        }
        for id in descriptor.private_names.values() {
            if !tables
                .get(&id.class_scope())
                .is_some_and(|slots| id.name_ordinal() < *slots)
            {
                return Err(RuntimeDirectEvalCallerError(
                    "caller private name has no actual table slot",
                ));
            }
        }
        if (descriptor.invocation == EvalInvocationContext::DerivedConstructor)
            != (descriptor.derived_owner == RuntimeDirectEvalDerivedOwner::ExternalCaller)
        {
            return Err(RuntimeDirectEvalCallerError(
                "derived caller grammar and owner disagree",
            ));
        }
        let context = DirectEvalContextIr::new(
            descriptor.strictness,
            descriptor.invocation,
            descriptor.private_names.clone(),
            None,
        )
        .with_derived_owner(match descriptor.derived_owner {
            RuntimeDirectEvalDerivedOwner::Absent => None,
            RuntimeDirectEvalDerivedOwner::ExternalCaller => {
                Some(DirectEvalDerivedOwner::ExternalCaller)
            }
        })
        .with_script_import_origin(Some(ScriptImportOrigin::ExternalCaller(
            descriptor.import_origin,
        )));
        Ok(Self {
            descriptor,
            context,
            private_environment_floor,
        })
    }
    pub fn descriptor(&self) -> &RuntimeDirectEvalCallerDescriptor {
        &self.descriptor
    }
    pub fn context(&self) -> &DirectEvalContextIr {
        &self.context
    }
    pub const fn private_environment_floor(&self) -> u32 {
        self.private_environment_floor
    }
}
