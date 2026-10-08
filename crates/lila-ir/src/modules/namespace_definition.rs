//! Trusted definitions for canonical module activations and original callable source.

use crate::*;

#[derive(Debug, Default)]
pub(crate) struct LinkedScriptDefinitions {
    pub(super) synchronous: Option<super::synchronous_definition::ModuleExecutionDefinitions>,
    pub(super) entry: Option<super::LinkedModuleEntry>,
    pub(super) callable_sources: super::callable_source::CallableSourceDefinitions,
}

impl LinkedScriptDefinitions {
    pub(crate) fn has_realm_reusable_modules(&self) -> bool {
        self.synchronous.is_some()
    }

    pub(crate) fn apply<'a>(
        &self,
        script: &'a Script,
        analysis: &mut Analysis<'a>,
        interner: &Interner,
    ) {
        if let Some(synchronous) = &self.synchronous {
            synchronous.apply(script, analysis, interner);
        }
        if let Some(entry) = self.entry {
            entry.apply(script, analysis);
        }
        self.callable_sources.apply(script, analysis);
    }
}
