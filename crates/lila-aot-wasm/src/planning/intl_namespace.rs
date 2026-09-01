use super::*;

/// Evidence that [`INTL_NAMESPACE_ROOTS`] has been seeded into a plan's root
/// set, or that the plan initialises every builtin anyway.
///
/// The unit field is private to this module and there is no constructor, so
/// this cannot be built outside [`IntlNamespacePlan::rooted`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IntlRootsSeeded(());

/// Whether this plan installs the `Intl` namespace object.
///
/// The non-`Absent` variant is produced **only** by
/// [`IntlNamespacePlan::rooted`], which takes the root set by `&mut` and
/// seeds [`INTL_NAMESPACE_ROOTS`] into it before it can return. "Marked as
/// installed but missing a member the IR shape declares" is therefore
/// unrepresentable, rather than merely untested — which is what the previous
/// `intl_object: bool` was, and what `init_intl_object`'s per-member
/// `should_initialize_standard_builtin` guard existed to paper over.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum IntlNamespacePlan {
    /// This program never names `Intl` and never reaches an `Intl` builtin,
    /// so no namespace object is emitted at all.
    #[default]
    Absent,
    /// The namespace object is emitted, and every [`INTL_NAMESPACE_ROOTS`]
    /// id is in `standard_roots`.
    RootedWithDateTimeFormatFamily(IntlRootsSeeded),
}

impl IntlNamespacePlan {
    /// The only constructor of
    /// [`IntlNamespacePlan::RootedWithDateTimeFormatFamily`].
    pub(crate) fn rooted(standard_roots: &mut BTreeSet<StandardBuiltinId>) -> Self {
        standard_roots.extend(INTL_NAMESPACE_ROOTS);
        Self::RootedWithDateTimeFormatFamily(IntlRootsSeeded(()))
    }

    /// The member list, or `None` when no `Intl` object is emitted.
    ///
    /// `full_standard_globals` is a parameter rather than a second variant
    /// because it discharges the same obligation by a different route:
    /// `should_initialize_standard_builtin` is unconditionally true under
    /// it, so every member is rooted by definition. Keeping the check here
    /// is what leaves [`IntlNamespaceMembers`] with no reachable
    /// constructor.
    pub(crate) fn members(self, full_standard_globals: bool) -> Option<IntlNamespaceMembers> {
        if full_standard_globals || matches!(self, Self::RootedWithDateTimeFormatFamily(_)) {
            Some(IntlNamespaceMembers {
                members: INTL_NAMESPACE_CONSTRUCTORS,
            })
        } else {
            None
        }
    }
}

/// Proof that every member of the `Intl` namespace object is rooted in the
/// plan that produced it.
///
/// The proof has two halves, and only one of them is this type: minting one
/// requires a plan that has seeded [`INTL_NAMESPACE_ROOTS`], and the `const`
/// block beside that list is what makes "seeded the roots" imply "rooted
/// every member of `INTL_NAMESPACE_CONSTRUCTORS`". The second half is
/// cross-crate — the member list lives in `lila-ir` — so it cannot be a
/// property of this type, only of the build.
///
/// Only [`IntlNamespacePlan::members`] can mint one — reached through
/// [`RuntimeBootstrapPlan::intl_namespace_members`] — and it is the only way
/// to reach the installation list, so `init_intl_object` cannot install a
/// partial `Intl`. The emitter used to re-check
/// `should_initialize_standard_builtin` per member and `continue` past the
/// ones that failed; the omission compiled, formatted cleanly, and produced
/// an `Intl` object whose contents disagreed with the shape the same program
/// was compiled against.
#[derive(Debug, Clone, Copy)]
pub(crate) struct IntlNamespaceMembers {
    /// Private to this module — not merely to `planning` — so nothing else
    /// can name `INTL_NAMESPACE_CONSTRUCTORS` into an
    /// [`IntlNamespaceMembers`] and fabricate the proof. A unit struct would
    /// have been forgeable.
    members: &'static [(&'static str, StandardBuiltinId)],
}

impl IntlNamespaceMembers {
    /// Installation order, which is `Object.getOwnPropertyNames(Intl)` order
    /// and therefore observable. Do not sort it here.
    pub(crate) fn in_installation_order(
        self,
    ) -> impl Iterator<Item = (&'static str, StandardBuiltinId)> {
        self.members.iter().copied()
    }
}
