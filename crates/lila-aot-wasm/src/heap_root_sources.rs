//! Closed source identities for the passive heap-root inventory.

#![allow(
    dead_code,
    reason = "T05 root inventory precedes executable semantic tracing"
)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeapRootSource {
    RealmGlobals,
    ActiveFrameLocals,
    LexicalEnvironments,
    CompletionRecords,
    FunctionTable,
    HostBorrowedValues,
    PendingJobs,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum HeapRootKind {
    PersistentNonTagged,
    PersistentTaggedValues,
    TransientTaggedValues,
}

struct HeapRootMetadata {
    name: &'static str,
    owner: &'static str,
    kind: HeapRootKind,
}

impl HeapRootSource {
    const fn metadata(self) -> HeapRootMetadata {
        match self {
            Self::RealmGlobals => HeapRootMetadata {
                name: "realm-globals",
                owner: "module-globals",
                kind: HeapRootKind::PersistentNonTagged,
            },
            Self::ActiveFrameLocals => HeapRootMetadata {
                name: "active-frame-locals",
                owner: "function-locals",
                kind: HeapRootKind::TransientTaggedValues,
            },
            Self::LexicalEnvironments => HeapRootMetadata {
                name: "lexical-environments",
                owner: "environment-chain",
                kind: HeapRootKind::PersistentTaggedValues,
            },
            Self::CompletionRecords => HeapRootMetadata {
                name: "completion-records",
                owner: "completion-abi",
                kind: HeapRootKind::TransientTaggedValues,
            },
            Self::FunctionTable => HeapRootMetadata {
                name: "function-table",
                owner: "indirect-call-table",
                kind: HeapRootKind::PersistentNonTagged,
            },
            Self::HostBorrowedValues => HeapRootMetadata {
                name: "host-borrowed-values",
                owner: "host-import-boundary",
                kind: HeapRootKind::TransientTaggedValues,
            },
            Self::PendingJobs => HeapRootMetadata {
                name: "pending-jobs",
                owner: "job-queue",
                kind: HeapRootKind::PersistentTaggedValues,
            },
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        self.metadata().name
    }

    pub(crate) const fn owner(self) -> &'static str {
        self.metadata().owner
    }

    pub(crate) const fn kind(self) -> HeapRootKind {
        self.metadata().kind
    }
}

pub(crate) const HEAP_ROOT_SOURCES: &[HeapRootSource] = &[
    HeapRootSource::RealmGlobals,
    HeapRootSource::ActiveFrameLocals,
    HeapRootSource::LexicalEnvironments,
    HeapRootSource::CompletionRecords,
    HeapRootSource::FunctionTable,
    HeapRootSource::HostBorrowedValues,
    HeapRootSource::PendingJobs,
];
