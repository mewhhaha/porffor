//! Admission of the actual invocation RSS sampler's observations.
use super::{ByteChange, ByteSummary};
use lila_engine::{WasmProcessMemoryObservation, WasmProcessMemorySample,
    WasmProcessMemoryScope, WasmProcessMemoryUnavailable, WasmRuntimeProfile};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Scope { RuntimeInvocationProcessRss }

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Unavailable {
    UnsupportedPlatform, ProcStatusReadFailed, ProcStatusMalformed,
    SamplingThreadStartFailed, SamplingThreadPanicked, SampleCountOverflow,
}
impl From<WasmProcessMemoryUnavailable> for Unavailable {
    fn from(reason: WasmProcessMemoryUnavailable) -> Self {
        match reason {
            WasmProcessMemoryUnavailable::UnsupportedPlatform => Self::UnsupportedPlatform,
            WasmProcessMemoryUnavailable::ProcStatusReadFailed => Self::ProcStatusReadFailed,
            WasmProcessMemoryUnavailable::ProcStatusMalformed => Self::ProcStatusMalformed,
            WasmProcessMemoryUnavailable::SamplingThreadStartFailed => Self::SamplingThreadStartFailed,
            WasmProcessMemoryUnavailable::SamplingThreadPanicked => Self::SamplingThreadPanicked,
            WasmProcessMemoryUnavailable::SampleCountOverflow => Self::SampleCountOverflow,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case", deny_unknown_fields)]
pub(super) enum Measurement {
    Sampled {
        scope: Scope,
        interval_ns: u64,
        elapsed_ns: u64,
        sample_count: u64,
        initial_rss_bytes: u64,
        final_rss_bytes: u64,
        maximum_rss_bytes: u64,
    },
    Unavailable { reason: Unavailable },
}
impl Measurement {
    pub(super) fn from_profile(profile: &WasmRuntimeProfile) -> Result<Self, String> {
        match &profile.process_memory {
            WasmProcessMemoryObservation::Sampled(sample) => Ok(Self::Sampled {
                scope: match sample.scope() {
                    WasmProcessMemoryScope::RuntimeInvocationProcessRss => Scope::RuntimeInvocationProcessRss,
                },
                interval_ns: interval_ns(),
                elapsed_ns: sample.elapsed().as_nanos().try_into()
                    .map_err(|_| "process-memory sample span exceeds report capacity")?,
                sample_count: sample.sample_count(),
                initial_rss_bytes: sample.initial_rss_bytes(),
                final_rss_bytes: sample.final_rss_bytes(),
                maximum_rss_bytes: sample.maximum_rss_bytes(),
            }),
            WasmProcessMemoryObservation::Unavailable(reason) => Ok(Self::Unavailable { reason: (*reason).into() }),
        }
    }

    pub(super) fn validate(&self) -> Result<(), String> {
        match self {
            Self::Sampled { scope: Scope::RuntimeInvocationProcessRss, interval_ns: interval,
                elapsed_ns, sample_count, initial_rss_bytes, final_rss_bytes, maximum_rss_bytes } => {
                // Endpoint reads bracket every invocation. Timeout-driven reads
                // wait at least one interval; they cannot exceed this bound.
                if *interval != interval_ns() || *sample_count < 2
                    || u128::from(*sample_count) > u128::from(*elapsed_ns) / u128::from(*interval) + 2
                    || *maximum_rss_bytes < (*initial_rss_bytes).max(*final_rss_bytes)
                    || [*initial_rss_bytes, *final_rss_bytes, *maximum_rss_bytes]
                        .into_iter().any(|bytes| bytes % 1024 != 0) {
                    return Err("process RSS observation differs from the invocation sampler contract".into());
                }
                Ok(())
            }
            Self::Unavailable { .. } => Ok(()),
        }
    }

    pub(super) fn validate_for_invocation(&self, total_ns: u64) -> Result<(), String> {
        self.validate()?;
        if let Self::Sampled { elapsed_ns, .. } = self {
            if *elapsed_ns > total_ns {
                return Err("process RSS sampling span exceeds its runtime invocation".into());
            }
        }
        Ok(())
    }
}

fn interval_ns() -> u64 { WasmProcessMemorySample::SAMPLE_INTERVAL.as_nanos() as u64 }

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case", deny_unknown_fields)]
pub(super) enum Summary {
    Sampled { scope: Scope, interval_ns: u64, maximum_rss: ByteSummary },
    /// One failed observation leaves the complete per-case metric unavailable;
    /// successful partial samples remain in the original report.
    Unavailable { reasons: Vec<Unavailable> },
}
impl Summary {
    pub(super) fn from_samples<'a>(samples: impl Iterator<Item = &'a Measurement>) -> Result<Self, String> {
        let mut values = Vec::new();
        let mut unavailable = BTreeSet::new();
        for sample in samples {
            sample.validate()?;
            match sample {
                Measurement::Sampled { maximum_rss_bytes, .. } => values.push(*maximum_rss_bytes),
                Measurement::Unavailable { reason } => { unavailable.insert(*reason); }
            }
        }
        if !unavailable.is_empty() {
            return Ok(Self::Unavailable { reasons: unavailable.into_iter().collect() });
        }
        Ok(Self::Sampled { scope: Scope::RuntimeInvocationProcessRss,
            interval_ns: interval_ns(), maximum_rss: ByteSummary::from_samples(&values)? })
    }

    pub(super) fn available(&self) -> Option<&ByteSummary> {
        match self { Self::Sampled { maximum_rss, .. } => Some(maximum_rss), Self::Unavailable { .. } => None }
    }

    pub(super) fn change(before: &Self, after: &Self) -> Option<ByteChange> {
        Some(ByteChange::between(before.available()?, after.available()?))
    }
}

#[cfg(test)]
pub(super) fn fixture_measurement(maximum: u64) -> Measurement {
    Measurement::Sampled { scope: Scope::RuntimeInvocationProcessRss,
        interval_ns: interval_ns(), elapsed_ns: 1, sample_count: 2,
        initial_rss_bytes: maximum, final_rss_bytes: maximum, maximum_rss_bytes: maximum }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sample_contract_rejects_lifetime_scope_forged_maxima_and_impossible_counts() {
        let original = fixture_measurement(1024);
        let mut wire = serde_json::to_value(&original).unwrap();
        wire["scope"] = "process-lifetime-high-water-mark".into();
        assert!(serde_json::from_value::<Measurement>(wire).is_err());
        let mut wire = serde_json::to_value(&original).unwrap();
        wire["maximum_rss_bytes"] = 0.into();
        assert!(serde_json::from_value::<Measurement>(wire).unwrap().validate().is_err());
        let mut wire = serde_json::to_value(&original).unwrap();
        wire["sample_count"] = 3.into();
        assert!(serde_json::from_value::<Measurement>(wire).unwrap().validate().is_err());
        let mut wire = serde_json::to_value(&original).unwrap();
        wire["interval_ns"] = 0.into();
        assert!(serde_json::from_value::<Measurement>(wire).unwrap().validate().is_err());
    }

    #[test]
    fn one_unavailable_invocation_cannot_publish_a_partial_numeric_budget_metric() {
        let samples = [fixture_measurement(1024),
            Measurement::Unavailable { reason: Unavailable::ProcStatusReadFailed },
            fixture_measurement(9 * 1024)];
        let summary = Summary::from_samples(samples.iter()).unwrap();
        assert!(summary.available().is_none());
        assert_eq!(summary, Summary::Unavailable { reasons: vec![Unavailable::ProcStatusReadFailed] });
        assert!(Summary::change(&summary,
            &Summary::from_samples([fixture_measurement(1024)].iter()).unwrap()).is_none());
    }
}
