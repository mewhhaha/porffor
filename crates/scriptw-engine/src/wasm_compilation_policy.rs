//! The exact policy retained across lower/emission/cache/execution boundaries.
use super::{
    CompilationUnit, CompileOptions, HostSurfacePolicy, IntlCompilationProfile,
    LanguageFeaturePolicy,
};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmCompilationPolicy {
    host_surface_policy: HostSurfacePolicy,
    language_features: LanguageFeaturePolicy,
    intl_profile: IntlCompilationProfile,
}

impl WasmCompilationPolicy {
    pub(crate) fn from_options(options: &CompileOptions) -> Self {
        Self {
            host_surface_policy: options.host_surface_policy,
            language_features: options.language_features,
            intl_profile: options.intl_profile.clone(),
        }
    }
    pub(crate) fn from_unit(unit: &CompilationUnit) -> Self {
        Self {
            host_surface_policy: unit.host_surface_policy,
            language_features: unit.ir.language_features(),
            intl_profile: unit.intl_profile.clone(),
        }
    }
    pub(crate) fn host_surface_policy(&self) -> HostSurfacePolicy {
        self.host_surface_policy
    }
    pub(crate) fn language_features(&self) -> LanguageFeaturePolicy {
        self.language_features
    }
    pub(crate) fn intl_profile(&self) -> &IntlCompilationProfile {
        &self.intl_profile
    }
}
