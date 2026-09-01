//! Closed passive collector policy.

#![allow(
    dead_code,
    reason = "T05 collector policy remains metadata until collection is executable"
)]

use super::heap_collector_phases::{RequiredHeapCollectorPhase, REQUIRED_HEAP_COLLECTOR_PHASES};
use super::heap_root_sources::{HeapRootSource, HEAP_ROOT_SOURCES};
use super::heap_weak_edges::{HeapWeakEdge, HEAP_WEAK_EDGES};

pub(crate) enum HeapCollectorPolicy {
    NonMovingMetadataChecked,
}

impl HeapCollectorPolicy {
    pub(crate) const fn name(&self) -> &'static str {
        match self {
            Self::NonMovingMetadataChecked => "non-moving-tracing-collector",
        }
    }

    pub(crate) const fn moves_objects(&self) -> bool {
        match self {
            Self::NonMovingMetadataChecked => false,
        }
    }

    pub(crate) const fn root_sources(&self) -> &'static [HeapRootSource] {
        match self {
            Self::NonMovingMetadataChecked => HEAP_ROOT_SOURCES,
        }
    }

    pub(crate) const fn weak_edges(&self) -> &'static [HeapWeakEdge] {
        match self {
            Self::NonMovingMetadataChecked => HEAP_WEAK_EDGES,
        }
    }

    pub(crate) const fn required_phases(&self) -> &'static [RequiredHeapCollectorPhase] {
        match self {
            Self::NonMovingMetadataChecked => REQUIRED_HEAP_COLLECTOR_PHASES,
        }
    }

    pub(crate) const fn is_executable(&self) -> bool {
        match self {
            Self::NonMovingMetadataChecked => false,
        }
    }
}

pub(crate) const HEAP_COLLECTOR_POLICY: HeapCollectorPolicy =
    HeapCollectorPolicy::NonMovingMetadataChecked;
