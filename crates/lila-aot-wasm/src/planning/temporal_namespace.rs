use super::*;

/// Evidence that every constructor and `Temporal.Now` member advertised by
/// the IR namespace shapes has been rooted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TemporalRootsSeeded(());

/// Whether this plan installs the complete `Temporal` namespace.
///
/// `Rooting` is a private construction-time state that breaks the existing
/// cycles between Temporal constructor families. It cannot produce
/// [`TemporalNamespaceMembers`], so bootstrap can observe only `Absent` or a
/// fully rooted namespace.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum TemporalNamespacePlan {
    #[default]
    Absent,
    Rooting,
    Rooted(TemporalRootsSeeded),
}

impl TemporalNamespacePlan {
    /// Root every member before publishing the installation witness.
    pub(crate) fn root(plan: &mut RuntimeBootstrapPlan) {
        if !matches!(plan.temporal, Self::Absent) {
            return;
        }

        plan.temporal = Self::Rooting;
        for (_, builtin) in TEMPORAL_NAMESPACE_CONSTRUCTORS
            .iter()
            .chain(TEMPORAL_NOW_NAMESPACE_MEMBERS)
        {
            plan.require_standard_builtin(*builtin);
        }
        plan.temporal = Self::Rooted(TemporalRootsSeeded(()));
    }

    pub(crate) fn members(self, full_standard_globals: bool) -> Option<TemporalNamespaceMembers> {
        if full_standard_globals || matches!(self, Self::Rooted(_)) {
            Some(TemporalNamespaceMembers {
                constructors: TEMPORAL_NAMESPACE_CONSTRUCTORS,
                now_members: TEMPORAL_NOW_NAMESPACE_MEMBERS,
            })
        } else {
            None
        }
    }
}

/// Proof that bootstrap may install the complete namespace without per-member
/// planning guards. Its fields and constructor are private to this module.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TemporalNamespaceMembers {
    constructors: &'static [(&'static str, StandardBuiltinId)],
    now_members: &'static [(&'static str, StandardBuiltinId)],
}

impl TemporalNamespaceMembers {
    pub(crate) fn constructors_in_installation_order(
        self,
    ) -> impl Iterator<Item = (&'static str, StandardBuiltinId)> {
        self.constructors.iter().copied()
    }

    pub(crate) fn now_members_in_installation_order(
        self,
    ) -> impl Iterator<Item = (&'static str, StandardBuiltinId)> {
        self.now_members.iter().copied()
    }
}
