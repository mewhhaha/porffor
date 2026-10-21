use scriptw_test262::{FailureKind, FailureOrigin, OutcomeKind};
use std::collections::BTreeMap;

pub(crate) enum OutcomeCoverage {
    Complete,
    Partial { observed: usize, target: usize },
}

pub(crate) struct OutcomeTotals {
    pub(crate) success: usize,
    pub(crate) not_implemented: usize,
    pub(crate) crash: usize,
    pub(crate) bug: usize,
    pub(crate) unobserved: usize,
}

impl OutcomeTotals {
    pub(crate) fn new(counts: &BTreeMap<OutcomeKind, usize>, coverage: OutcomeCoverage) -> Self {
        let get = |kind| counts.get(&kind).copied().unwrap_or(0);
        Self {
            success: get(OutcomeKind::Success),
            not_implemented: get(OutcomeKind::NotImplemented),
            crash: get(OutcomeKind::Crash),
            bug: get(OutcomeKind::Bug),
            unobserved: match coverage {
                OutcomeCoverage::Complete => 0,
                OutcomeCoverage::Partial { observed, target } => target.saturating_sub(observed),
            },
        }
    }

    pub(crate) fn observed_failures(&self) -> usize {
        self.not_implemented + self.crash + self.bug
    }

    pub(crate) fn remaining_to_green(&self) -> usize {
        self.unobserved + self.observed_failures()
    }
}

fn print_rows(rows: impl Iterator<Item = (&'static str, usize)>, indent: &str) {
    for (label, count) in rows {
        println!("{indent}{label}: {count}");
    }
}

pub(crate) fn print_kind_counts(counts: &BTreeMap<FailureKind, usize>) {
    print_rows(
        FailureKind::ordered_counts(counts).map(|(kind, count)| (kind.as_str(), count)),
        "",
    );
}

pub(crate) fn print_outcome_counts(counts: &BTreeMap<OutcomeKind, usize>) {
    println!("outcomes:");
    print_rows(
        OutcomeKind::ordered_counts(counts).map(|(kind, count)| (kind.as_str(), count)),
        "  ",
    );
}

pub(crate) fn print_origin_counts(counts: &BTreeMap<FailureOrigin, usize>) {
    println!("origins:");
    print_rows(
        FailureOrigin::ordered_counts(counts).map(|(kind, count)| (kind.as_str(), count)),
        "  ",
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_progress_preserves_unobserved_cases_as_remaining_work() {
        let counts = [(OutcomeKind::Success, 3), (OutcomeKind::Bug, 1)].into();
        let partial = OutcomeTotals::new(
            &counts,
            OutcomeCoverage::Partial {
                observed: 4,
                target: 10,
            },
        );
        assert_eq!(partial.unobserved, 6);
        assert_eq!(partial.observed_failures(), 1);
        assert_eq!(partial.remaining_to_green(), 7);
        let complete = OutcomeTotals::new(&counts, OutcomeCoverage::Complete);
        assert_eq!(complete.unobserved, 0);
        assert_eq!(complete.remaining_to_green(), 1);
        assert_eq!(
            OutcomeKind::ordered_counts(&counts).collect::<Vec<_>>(),
            vec![
                (OutcomeKind::Success, 3),
                (OutcomeKind::NotImplemented, 0),
                (OutcomeKind::Crash, 0),
                (OutcomeKind::Bug, 1)
            ]
        );
    }
}
