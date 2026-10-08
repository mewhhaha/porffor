//! Runtime-to-program hooks.
//!
//! Runtime bodies are identical for every program, so anything they need to
//! learn from the program goes through a [`ProgramHook`]: a mutable
//! `(ref null $helper)` global that is null until `main` installs the
//! program's own body for it. Runtime code branches on the global and takes
//! its unprepared path when it is null, so a program without the matching
//! prepared data behaves exactly as if the hook did not exist.

use super::*;
use crate::emit::{ControlFrameKind, FunctionBuilder};
use crate::runtime_helpers::{InstalledHook, ProgramHook};

/// The hooks a program has data for. Only these are installed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ProvidedHooks(u32);

impl ProvidedHooks {
    pub(crate) fn of(functions: &FunctionMetaRegistry, strings: &StringPool) -> Self {
        let mut provided = 0;
        for hook in ProgramHook::ALL {
            let has_data = match hook {
                ProgramHook::PreparedDynamicFunction => {
                    !functions.prepared_dynamic_functions().is_empty()
                }
                ProgramHook::PreparedScript => functions
                    .prepared_scripts()
                    .iter()
                    .any(|script| !matches!(script.kind, PreparedScriptKind::DirectEval(_))),
                ProgramHook::RealmModuleImport => functions.module_graph().is_some(),
                ProgramHook::ModuleBodyReaction => functions.module_execution_record_count() > 0,
                ProgramHook::RegExpProgramCandidate => {
                    strings.runtime_regexp_candidates().next().is_some()
                }
            };
            if has_data {
                provided |= 1 << hook as u32;
            }
        }
        Self(provided)
    }

    pub(crate) const fn provides(self, hook: ProgramHook) -> bool {
        self.0 & (1 << hook as u32) != 0
    }
}

impl FunctionBuilder<'_> {
    /// Emits `if hook installed { installed } else { absent }`. Only the
    /// installed arm receives the token that permits calling the hook.
    pub(crate) fn emit_program_hook_dispatch(
        &mut self,
        hook: ProgramHook,
        installed: impl FnOnce(&mut Self, &InstalledHook, &mut Function) -> Result<(), EmitError>,
        absent: impl FnOnce(&mut Self, &mut Function) -> Result<(), EmitError>,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        function.instruction(&Instruction::GlobalGet(schema.program_hook_global(hook)));
        function.instruction(&Instruction::RefIsNull);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        installed(self, &InstalledHook::mint(hook), function)?;
        function.instruction(&Instruction::Else);
        absent(self, function)?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `main` publishes the program's bodies before any runtime code can ask.
    pub(crate) fn emit_install_program_hooks(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let provided = ProvidedHooks::of(self.functions, self.strings);
        let base = self.runtime_helper_base()?;
        for hook in ProgramHook::ALL {
            if provided.provides(hook) {
                function.instruction(&Instruction::RefFunc(hook.helper().index(base)));
                function.instruction(&Instruction::GlobalSet(schema.program_hook_global(hook)));
            }
        }
        Ok(())
    }

    /// The body of a hook's program-owned helper.
    pub(crate) fn compile_program_hook(
        &mut self,
        hook: ProgramHook,
    ) -> Result<Function, EmitError> {
        match hook {
            ProgramHook::PreparedDynamicFunction => self.compile_prepared_dynamic_function_hook(),
            ProgramHook::PreparedScript => self.compile_prepared_script_hook(),
            ProgramHook::RealmModuleImport => self.compile_realm_module_import_hook(),
            ProgramHook::ModuleBodyReaction => self.compile_module_body_reaction_hook(),
            ProgramHook::RegExpProgramCandidate => self.compile_regexp_program_candidate_hook(),
        }
    }
}
