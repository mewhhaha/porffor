//! Closed policy for host borrowing of linear memory and heap roots.

#![allow(
    dead_code,
    reason = "T05 host-boundary metadata precedes executable semantic rooting"
)]

pub(crate) enum HeapHostBoundaryPolicy {
    ImportCallOnlyWithTransientTaggedRoots,
}

impl HeapHostBoundaryPolicy {
    pub(crate) const fn name(&self) -> &'static str {
        match self {
            Self::ImportCallOnlyWithTransientTaggedRoots => "host-import-memory-borrow",
        }
    }

    pub(crate) const fn borrowed_root_source(&self) -> super::heap_root_sources::HeapRootSource {
        match self {
            Self::ImportCallOnlyWithTransientTaggedRoots => {
                super::heap_root_sources::HeapRootSource::HostBorrowedValues
            }
        }
    }
}

pub(crate) const HEAP_HOST_BOUNDARY_POLICY: HeapHostBoundaryPolicy =
    HeapHostBoundaryPolicy::ImportCallOnlyWithTransientTaggedRoots;
