//! Closed weak-edge identities for the passive heap inventory.

#![allow(
    dead_code,
    reason = "T05 weak-edge metadata precedes an executable weak-reachability facility"
)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeapWeakEdgeKind {
    EphemeronKey,
    EphemeronValue,
    WeakTarget,
    FinalizerHoldings,
    FinalizerToken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeapWeakEdgeRetention {
    DoesNotRetain,
    ConditionalOnReachableEphemeronKey,
    StrongUntilCleanup,
}

impl HeapWeakEdgeKind {
    pub(crate) const fn retention(self) -> HeapWeakEdgeRetention {
        match self {
            Self::EphemeronKey | Self::WeakTarget | Self::FinalizerToken => {
                HeapWeakEdgeRetention::DoesNotRetain
            }
            Self::EphemeronValue => HeapWeakEdgeRetention::ConditionalOnReachableEphemeronKey,
            Self::FinalizerHoldings => HeapWeakEdgeRetention::StrongUntilCleanup,
        }
    }
}

pub(crate) enum HeapWeakEdge {
    WeakMapKey,
    WeakMapValue,
    WeakSetValue,
    WeakRefTarget,
    FinalizationRegistryTarget,
    FinalizationRegistryHoldings,
    FinalizationRegistryUnregisterToken,
}

struct HeapWeakEdgeMetadata {
    record: &'static str,
    name: &'static str,
    kind: HeapWeakEdgeKind,
}

impl HeapWeakEdge {
    const fn metadata(&self) -> HeapWeakEdgeMetadata {
        match self {
            Self::WeakMapKey => HeapWeakEdgeMetadata {
                record: "weak-map-entry",
                name: "key",
                kind: HeapWeakEdgeKind::EphemeronKey,
            },
            Self::WeakMapValue => HeapWeakEdgeMetadata {
                record: "weak-map-entry",
                name: "value",
                kind: HeapWeakEdgeKind::EphemeronValue,
            },
            Self::WeakSetValue => HeapWeakEdgeMetadata {
                record: "weak-set-entry",
                name: "value",
                kind: HeapWeakEdgeKind::EphemeronKey,
            },
            Self::WeakRefTarget => HeapWeakEdgeMetadata {
                record: "weak-ref-record",
                name: "target",
                kind: HeapWeakEdgeKind::WeakTarget,
            },
            Self::FinalizationRegistryTarget => HeapWeakEdgeMetadata {
                record: "finalization-registry-cell",
                name: "target",
                kind: HeapWeakEdgeKind::WeakTarget,
            },
            Self::FinalizationRegistryHoldings => HeapWeakEdgeMetadata {
                record: "finalization-registry-cell",
                name: "holdings",
                kind: HeapWeakEdgeKind::FinalizerHoldings,
            },
            Self::FinalizationRegistryUnregisterToken => HeapWeakEdgeMetadata {
                record: "finalization-registry-cell",
                name: "unregister-token",
                kind: HeapWeakEdgeKind::FinalizerToken,
            },
        }
    }

    pub(crate) const fn record(&self) -> &'static str {
        self.metadata().record
    }

    pub(crate) const fn name(&self) -> &'static str {
        self.metadata().name
    }

    pub(crate) const fn kind(&self) -> HeapWeakEdgeKind {
        self.metadata().kind
    }
}

pub(crate) const HEAP_WEAK_EDGES: &[HeapWeakEdge] = &[
    HeapWeakEdge::WeakMapKey,
    HeapWeakEdge::WeakMapValue,
    HeapWeakEdge::WeakSetValue,
    HeapWeakEdge::WeakRefTarget,
    HeapWeakEdge::FinalizationRegistryTarget,
    HeapWeakEdge::FinalizationRegistryHoldings,
    HeapWeakEdge::FinalizationRegistryUnregisterToken,
];
