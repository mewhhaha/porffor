/// Host behavior after the Wasm main export drains its Promise jobs.
/// This does not change Promise settlement or the handling of rejected reactions.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum PromiseRejectionPolicy {
    /// Report the oldest unhandled rejection as a failed run and print later ones.
    #[default]
    FailRun,
    /// Use the ECMAScript default HostPromiseRejectionTracker behavior.
    /// Preserve Script completion without running diagnostic coercions.
    /// Async Module graphs remain unsupported with this policy until their
    /// evaluation completion is independent of unhandled-rejection reporting.
    Ignore,
}
