//! Closed required-phase domain for the passive collector contract.

#![allow(
    dead_code,
    reason = "T05 collector phases remain metadata until collection is executable"
)]

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RequiredHeapCollectorPhase {
    StopTheWorld,
    RootScan,
    MarkStrong,
    ProcessEphemerons,
    ClearWeakRefs,
    QueueFinalizers,
    Sweep,
    Resume,
}

impl RequiredHeapCollectorPhase {
    pub(crate) const fn name(&self) -> &'static str {
        match self {
            Self::StopTheWorld => "stop-the-world",
            Self::RootScan => "scan-roots",
            Self::MarkStrong => "mark-strong-graph",
            Self::ProcessEphemerons => "process-ephemerons",
            Self::ClearWeakRefs => "clear-weakrefs",
            Self::QueueFinalizers => "queue-finalizers",
            Self::Sweep => "sweep-unmarked",
            Self::Resume => "resume",
        }
    }
}

pub(crate) const REQUIRED_HEAP_COLLECTOR_PHASES: &[RequiredHeapCollectorPhase] = &[
    RequiredHeapCollectorPhase::StopTheWorld,
    RequiredHeapCollectorPhase::RootScan,
    RequiredHeapCollectorPhase::MarkStrong,
    RequiredHeapCollectorPhase::ProcessEphemerons,
    RequiredHeapCollectorPhase::ClearWeakRefs,
    RequiredHeapCollectorPhase::QueueFinalizers,
    RequiredHeapCollectorPhase::Sweep,
    RequiredHeapCollectorPhase::Resume,
];
