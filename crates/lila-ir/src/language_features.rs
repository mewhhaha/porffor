//! Explicit opt-in semantics layered on the pinned ECMA-262 baseline.
//! Host-surface selection grants no proposal authority.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum PrivateElementExtensibility {
    #[default]
    Ecma262,
    NonExtensibleAppliesToPrivate,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct LanguageFeaturePolicy {
    private_element_extensibility: PrivateElementExtensibility,
}

impl LanguageFeaturePolicy {
    pub const fn with_private_element_extensibility(
        self,
        policy: PrivateElementExtensibility,
    ) -> Self {
        Self {
            private_element_extensibility: policy,
        }
    }

    pub const fn private_element_extensibility(self) -> PrivateElementExtensibility {
        self.private_element_extensibility
    }

    /// Closed code shared by the P-owned immutable field and program cache key.
    /// No raw-word constructor can mint a language policy.
    pub const fn wire_code(self) -> i32 {
        match self.private_element_extensibility {
            PrivateElementExtensibility::Ecma262 => 0,
            PrivateElementExtensibility::NonExtensibleAppliesToPrivate => 1,
        }
    }
}
