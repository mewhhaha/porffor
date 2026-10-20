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

#[derive(Default)]
pub(super) struct SourceCompilationProfileAdmission {
    profile: Option<RuntimeSourceCompilationProfile>,
    imports: [bool; 3],
}

impl SourceCompilationProfileAdmission {
    pub(super) fn import(&mut self, import: &Import<'_>) -> Result<(), EmitError> {
        let slot = match import.name {
            "compile_dynamic_function" => 0,
            "compile_global_script" => 1,
            "compile_direct_eval" => 2,
            _ => return Ok(()),
        };
        if import.module != "scriptw_host"
            || !matches!(import.ty, TypeRef::Func(_) | TypeRef::FuncExact(_))
            || self.imports[slot]
        {
            return Err(RuntimeSourceCompilationProfile::malformed());
        }
        self.imports[slot] = true;
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
            RuntimeSourceCompilationProfile::Portable => [false; 3],
            RuntimeSourceCompilationProfile::NativeHost => [true; 3],
        };
        if self.imports != expected {
            return Err(RuntimeSourceCompilationProfile::malformed());
        }
        Ok(profile)
    }
}

#[cfg(test)]
mod tests;
