//! Derive summaries only from retained raw nanosecond samples.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Summary {
    pub samples: usize,
    pub minimum_ns: u64,
    pub median_ns: u64,
    pub p90_ns: u64,
    pub p95_ns: u64,
    pub maximum_ns: u64,
}
impl Summary {
    pub fn from_samples(values: &[u64]) -> Result<Self, String> {
        if values.is_empty() {
            return Err("summary requires completed samples".into());
        }
        let mut sorted = values.to_vec();
        sorted.sort_unstable();
        let n = sorted.len();
        let median = if n % 2 == 1 {
            sorted[n / 2]
        } else {
            ((u128::from(sorted[n / 2 - 1]) + u128::from(sorted[n / 2])) / 2) as u64
        };
        let rank = |percent: usize| sorted[(n * percent).div_ceil(100) - 1];
        Ok(Self {
            samples: n,
            minimum_ns: sorted[0],
            median_ns: median,
            p90_ns: rank(90),
            p95_ns: rank(95),
            maximum_ns: sorted[n - 1],
        })
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Change {
    pub baseline_median_ns: u64,
    pub candidate_median_ns: u64,
    pub median_delta_ns: i128,
    pub median_ratio: Option<f64>,
    pub baseline_p95_ns: u64,
    pub candidate_p95_ns: u64,
    pub p95_delta_ns: i128,
    pub p95_ratio: Option<f64>,
}
impl Change {
    pub fn between(before: &Summary, after: &Summary) -> Self {
        let ratio = |before, after| {
            if before == 0 {
                None
            } else {
                Some(after as f64 / before as f64)
            }
        };
        Self {
            baseline_median_ns: before.median_ns,
            candidate_median_ns: after.median_ns,
            median_delta_ns: i128::from(after.median_ns) - i128::from(before.median_ns),
            median_ratio: ratio(before.median_ns, after.median_ns),
            baseline_p95_ns: before.p95_ns,
            candidate_p95_ns: after.p95_ns,
            p95_delta_ns: i128::from(after.p95_ns) - i128::from(before.p95_ns),
            p95_ratio: ratio(before.p95_ns, after.p95_ns),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn distributions_keep_outliers_and_compute_even_medians_without_overflow() {
        let summary = Summary::from_samples(&[1000, 1, 4, 2, 3]).unwrap();
        assert_eq!(
            (summary.median_ns, summary.p90_ns, summary.p95_ns),
            (3, 1000, 1000)
        );
        assert_eq!(
            Summary::from_samples(&[u64::MAX, u64::MAX])
                .unwrap()
                .median_ns,
            u64::MAX
        );
        assert_eq!(Summary::from_samples(&[8, 2, 4, 6]).unwrap().median_ns, 5);
        assert!(Summary::from_samples(&[]).is_err());
    }
    #[test]
    fn changes_keep_signed_deltas_and_zero_baselines_explicit() {
        let zero = Summary::from_samples(&[0]).unwrap();
        let fast = Summary::from_samples(&[10]).unwrap();
        let slow = Summary::from_samples(&[20]).unwrap();
        assert_eq!(Change::between(&zero, &fast).median_ratio, None);
        assert_eq!(Change::between(&slow, &fast).median_delta_ns, -10);
        assert_eq!(Change::between(&fast, &slow).p95_ratio, Some(2.0));
    }
}
