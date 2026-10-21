//! Source-compilation authority is explicit in the emitted module and identity.
use crate::EmitError;
use wasmparser::{Import, Parser, Payload, TypeRef};

pub const RUNTIME_SOURCE_COMPILATION_PROFILE_CUSTOM_SECTION: &str =
    "scriptw.runtime.source-compilation.v1";

/// Portable artifacts contain compiled Wasm and no native compiler callback.
/// Native execution explicitly selects the host's JS-to-IR-to-Wasm compiler.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum RuntimeSourceCompilationProfile {
    #[default]
    Portable,
    NativeHost,
}

impl RuntimeSourceCompilationProfile {
    pub(crate) const fn wire(self) -> u8 {
        match self {
            Self::Portable => 0,
            Self::NativeHost => 1,
        }
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        match bytes {
            [0] => Some(Self::Portable),
            [1] => Some(Self::NativeHost),
            _ => None,
        }
    }

    /// Read one exact physical profile and its corresponding compiler imports.
    /// This checks authority metadata; ordinary Wasm/type validation is separate.
    pub fn from_module(bytes: &[u8]) -> Result<Self, EmitError> {
        let mut admission = SourceCompilationProfileAdmission::default();
        for payload in Parser::new(0).parse_all(bytes) {
            match payload.map_err(|_| Self::malformed())? {
                Payload::ImportSection(section) => {
                    for import in section.into_imports() {
                        admission.import(&import.map_err(|_| Self::malformed())?)?;
                    }
                }
                Payload::CustomSection(section)
                    if section.name() == RUNTIME_SOURCE_COMPILATION_PROFILE_CUSTOM_SECTION =>
                {
                    admission.section(section.data())?;
                }
                _ => {}
            }
        }
        admission.finish()
    }

    fn malformed() -> EmitError {
        EmitError::unsupported("runtime source-compilation profile or compiler imports mismatch")
    }
}

#[derive(Clone, Copy)]
enum CompilerImportSlot {
    DynamicFunction,
    GlobalScript,
    DirectEval,
}
impl CompilerImportSlot {
    const ALL: [Self; 3] = [Self::DynamicFunction, Self::GlobalScript, Self::DirectEval];
    fn from_name(name: &str) -> Option<Self> {
        match name {
            "compile_dynamic_function" => Some(Self::DynamicFunction),
            "compile_global_script" => Some(Self::GlobalScript),
            "compile_direct_eval" => Some(Self::DirectEval),
            _ => None,
        }
    }
    const fn index(self) -> usize {
        match self {
            Self::DynamicFunction => 0,
            Self::GlobalScript => 1,
            Self::DirectEval => 2,
        }
    }
}

#[derive(Default)]
pub(super) struct SourceCompilationProfileAdmission {
    profile: Option<RuntimeSourceCompilationProfile>,
    imports: [bool; CompilerImportSlot::ALL.len()],
}

impl SourceCompilationProfileAdmission {
    pub(super) fn import(&mut self, import: &Import<'_>) -> Result<(), EmitError> {
        let Some(slot) = CompilerImportSlot::from_name(import.name) else {
            return Ok(());
        };
        if import.module != "scriptw_host"
            || !matches!(import.ty, TypeRef::Func(_) | TypeRef::FuncExact(_))
            || self.imports[slot.index()]
        {
            return Err(RuntimeSourceCompilationProfile::malformed());
        }
        self.imports[slot.index()] = true;
        Ok(())
    }

    pub(super) fn section(&mut self, bytes: &[u8]) -> Result<(), EmitError> {
        if self.profile.is_some() {
            return Err(RuntimeSourceCompilationProfile::malformed());
        }
        self.profile = Some(
            RuntimeSourceCompilationProfile::decode(bytes)
                .ok_or_else(RuntimeSourceCompilationProfile::malformed)?,
        );
        Ok(())
    }

    pub(super) fn finish(self) -> Result<RuntimeSourceCompilationProfile, EmitError> {
        let profile = self
            .profile
            .ok_or_else(RuntimeSourceCompilationProfile::malformed)?;
        let expected = match profile {
            RuntimeSourceCompilationProfile::Portable => [false; CompilerImportSlot::ALL.len()],
            RuntimeSourceCompilationProfile::NativeHost => [true; CompilerImportSlot::ALL.len()],
        };
        if self.imports != expected {
            return Err(RuntimeSourceCompilationProfile::malformed());
        }
        Ok(profile)
    }
}

#[cfg(test)]
mod tests;
