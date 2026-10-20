//! Discharged assertions share one bounded structural scanner. Capture effects
//! use full ordered Run snapshots or an actual entry-Clear independence proof.
use super::*;
mod regions;

#[derive(Clone, Copy)]
pub(super) enum AssertionStateEvidence {
    InputOnly,
    CompleteChoiceSnapshots,
    /// All capture writes belong to the actual entry Clear. The scanner also
    /// reports whether every path keeps the cursor fixed outside assertions.
    IndependentIterations {
        clear_first: I64Local,
        clear_end: I64Local,
        zero_width: I32Local,
    },
}

pub(super) fn emit_input_only_assertion(
    builder: &FunctionBuilder<'_>,
    workspace: &MatcherWorkspace,
    pair: &CountedLoopLocals,
    current: &CountedMatcherLocals,
    source_pc: I64Local,
    eligible: I32Local,
    f: &mut Function,
) {
    regions::emit_closed_assertion(
        builder,
        workspace,
        pair,
        current,
        source_pc,
        eligible,
        AssertionStateEvidence::InputOnly,
        f,
    );
}

pub(super) fn emit_choice_template_body(
    builder: &FunctionBuilder<'_>,
    workspace: &MatcherWorkspace,
    pair: &CountedLoopLocals,
    current: &CountedMatcherLocals,
    eligible: I32Local,
    f: &mut Function,
) {
    regions::emit_required_body(builder, workspace, pair, current, eligible, f);
}

pub(super) fn emit_independent_iteration_body(
    builder: &FunctionBuilder<'_>,
    workspace: &MatcherWorkspace,
    pair: &CountedLoopLocals,
    current: &CountedMatcherLocals,
    clear_first: I64Local,
    clear_end: I64Local,
    zero_width: I32Local,
    eligible: I32Local,
    f: &mut Function,
) {
    regions::emit_independent_body(
        builder,
        workspace,
        pair,
        current,
        eligible,
        AssertionStateEvidence::IndependentIterations {
            clear_first,
            clear_end,
            zero_width,
        },
        f,
    );
}

pub(super) fn emit_input_only_greedy_region(
    builder: &FunctionBuilder<'_>,
    workspace: &MatcherWorkspace,
    pair: &CountedLoopLocals,
    current: &CountedMatcherLocals,
    first: I64Local,
    end: I64Local,
    eligible: I32Local,
    f: &mut Function,
) {
    regions::emit_greedy_region(builder, workspace, pair, current, first, end, eligible, f);
}
