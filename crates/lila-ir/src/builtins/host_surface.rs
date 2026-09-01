use super::{HostBuiltinExposure, HostBuiltinId, HostBuiltinSurface};
use crate::DynamicSourceIntrinsic;

impl HostBuiltinExposure {
    pub(super) const fn realm_scope(self) -> super::HostBuiltinRealmScope {
        match self {
            Self::EcmaGlobal => super::HostBuiltinRealmScope::EveryRealm,
            Self::ProductExtension | Self::Test262Capability => {
                super::HostBuiltinRealmScope::EntryRealmOnly
            }
        }
    }
}

/// The host-global surface an IR compilation is authorized to expose.
///
/// This is deliberately a closed policy over [`HostBuiltinExposure`], not a
/// caller-provided list of names. Adding another exposure class therefore
/// requires an explicit decision for both compilation modes below.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostSurfacePolicy {
    Product,
    Test262,
}

impl Default for HostSurfacePolicy {
    fn default() -> Self {
        Self::Product
    }
}

impl HostSurfacePolicy {
    pub const fn allows(self, builtin: HostBuiltinId) -> bool {
        let HostBuiltinSurface::Global(exposure) = builtin.surface() else {
            return false;
        };
        match self {
            Self::Product => match exposure {
                HostBuiltinExposure::EcmaGlobal | HostBuiltinExposure::ProductExtension => true,
                HostBuiltinExposure::Test262Capability => false,
            },
            Self::Test262 => match exposure {
                HostBuiltinExposure::EcmaGlobal
                | HostBuiltinExposure::ProductExtension
                | HostBuiltinExposure::Test262Capability => true,
            },
        }
    }

    pub fn global_builtins(self) -> impl Iterator<Item = HostBuiltinId> {
        HostBuiltinId::global_builtins().filter(move |builtin| self.allows(*builtin))
    }

    pub fn resolve_global(self, name: &str) -> Option<HostBuiltinId> {
        HostBuiltinId::from_global_name(name).filter(|builtin| self.allows(*builtin))
    }

    /// Resolves host callables whose invocation must be rejected as dynamic
    /// source during lowering rather than reach their defensive host body.
    pub fn resolve_dynamic_source_intrinsic(self, name: &str) -> Option<DynamicSourceIntrinsic> {
        self.resolve_global(name)
            .and_then(|builtin| DynamicSourceIntrinsic::from_function_id(&builtin.function_id()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_is_the_authority_for_test262_globals() {
        assert_eq!(HostBuiltinId::ALL.len(), 19);
        assert_eq!(HostBuiltinId::global_builtins().count(), 18);
        for builtin in [
            HostBuiltinId::Print,
            HostBuiltinId::Gc,
            HostBuiltinId::ParseInt,
        ] {
            assert_eq!(
                HostSurfacePolicy::Product.resolve_global(builtin.as_str()),
                Some(builtin)
            );
            assert_eq!(
                HostSurfacePolicy::Test262.resolve_global(builtin.as_str()),
                Some(builtin)
            );
        }

        assert_eq!(
            HostSurfacePolicy::Product.resolve_global(HostBuiltinId::CreateRealm.as_str()),
            None
        );
        assert_eq!(
            HostSurfacePolicy::Test262.resolve_global(HostBuiltinId::CreateRealm.as_str()),
            Some(HostBuiltinId::CreateRealm)
        );
        assert!(!HostSurfacePolicy::Test262.allows(HostBuiltinId::HTMLDDA));

        let realm_eval = DynamicSourceIntrinsic::RealmEvalScript;
        let name = HostBuiltinId::RealmEvalScript
            .global_name()
            .expect("realm eval must have its internal harness name");
        assert_eq!(
            HostBuiltinId::from_global_name(name),
            Some(HostBuiltinId::RealmEvalScript),
        );
        assert_eq!(
            HostBuiltinId::from_function_id(realm_eval.function_id()),
            Some(HostBuiltinId::RealmEvalScript),
        );
        assert_eq!(
            HostSurfacePolicy::Product.resolve_dynamic_source_intrinsic(name),
            None
        );
        assert_eq!(
            HostSurfacePolicy::Test262.resolve_dynamic_source_intrinsic(name),
            Some(realm_eval),
        );
    }
}
