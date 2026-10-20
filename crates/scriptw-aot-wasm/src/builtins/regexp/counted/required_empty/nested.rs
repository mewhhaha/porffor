//! Exact positive children share the sole completed-pair lifecycle proof.
use super::*;

pub(super) fn emit_exact_completed_child(
    builder: &FunctionBuilder<'_>,
    workspace: &MatcherWorkspace,
    outer: &CountedLoopLocals,
    current: &CountedMatcherLocals,
    scan: &RequiredEmptyReplayLocals,
    f: &mut Function,
) {
    super::super::completed_exact_child::emit_if_completed_exact_child(
        builder,
        workspace,
        outer,
        current,
        scan.pc,
        scan.eligible,
        f,
        |_, _, _| {},
    );
}
