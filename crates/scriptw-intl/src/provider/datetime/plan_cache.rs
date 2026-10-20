//! Only this provider's admitted immutable plans are retained, never inputs or results.
use std::collections::VecDeque;
use std::mem::size_of;
use std::sync::{Arc, Mutex};

use crate::datetime::EncodedDateTimePlan;
use crate::IntlDataDigest;

use super::plan::ValidatedPlan;
use super::AdmittedDateTimePlan;

const MAX_ENTRIES: usize = 32;
const MAX_RETAINED_BYTES: usize = 2 * 1024 * 1024;
const MAX_KEY_BYTES: usize = 16 * 1024;

struct Entry {
    encoded: Box<[u8]>,
    selected: Arc<ValidatedPlan>,
    retained_bytes: usize,
}

struct Entries {
    values: VecDeque<Entry>,
    retained_bytes: usize,
}
const QUEUE_BYTES: usize = MAX_ENTRIES * size_of::<Entry>();
impl Default for Entries {
    fn default() -> Self {
        Self {
            values: VecDeque::with_capacity(MAX_ENTRIES),
            retained_bytes: QUEUE_BYTES,
        }
    }
}

/// Owned by one immutable DateTimeProvider. Foreign providers never consult it.
/// Exact bytes include the magic and all three admitted provider image digests.
pub(super) struct PlanCache {
    identity: [u8; 96],
    max_retained_bytes: usize,
    entries: Mutex<Entries>,
}

impl PlanCache {
    pub(super) fn new(
        image: &IntlDataDigest,
        locale: &IntlDataDigest,
        zones: &IntlDataDigest,
    ) -> Self {
        let mut identity = [0; 96];
        identity[..32].copy_from_slice(image.as_bytes());
        identity[32..64].copy_from_slice(locale.as_bytes());
        identity[64..].copy_from_slice(zones.as_bytes());
        Self {
            identity,
            max_retained_bytes: MAX_RETAINED_BYTES,
            entries: Mutex::default(),
        }
    }
    pub(super) fn get(&self, encoded: &EncodedDateTimePlan) -> Option<Arc<ValidatedPlan>> {
        if encoded.as_bytes().len() > MAX_KEY_BYTES
            || encoded.as_bytes().get(8..104) != Some(self.identity.as_slice())
        {
            return None;
        }
        self.entries.lock().ok()?.values.iter().find_map(|entry| {
            (entry.encoded.as_ref() == encoded.as_bytes()).then(|| Arc::clone(&entry.selected))
        })
    }

    // The caller has completed the original decode/identity/zone/selection path.
    // A poisoned cache or an over-budget plan simply leaves this optional reuse cold.
    pub(super) fn remember(&self, admitted: AdmittedDateTimePlan) {
        let AdmittedDateTimePlan { encoded, selected } = admitted;
        let retained_bytes = selected
            .retained_bytes()
            .saturating_add(encoded.len())
            .saturating_add(2 * size_of::<usize>());
        if encoded.len() > MAX_KEY_BYTES
            || retained_bytes > self.max_retained_bytes
            || encoded.get(8..104) != Some(self.identity.as_slice())
        {
            return;
        }
        let Ok(mut entries) = self.entries.lock() else {
            return;
        };
        if entries
            .values
            .iter()
            .any(|entry| entry.encoded.as_ref() == encoded.as_ref())
        {
            return;
        }
        while entries.values.len() >= MAX_ENTRIES
            || entries.retained_bytes.saturating_add(retained_bytes) > self.max_retained_bytes
        {
            let Some(retired) = entries.values.pop_front() else {
                return;
            };
            entries.retained_bytes -= retired.retained_bytes;
        }
        entries.retained_bytes += retained_bytes;
        entries.values.push_back(Entry {
            encoded,
            selected,
            retained_bytes,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::{date_components, provider, request};
    use super::*;
    use crate::{DateTimeStyleSelection, FixedTimeZoneOffset, TimeZoneSelection};

    fn empty_cache() -> PlanCache {
        let provider = provider();
        PlanCache::new(
            &provider.image_digest,
            &provider.locale_digest,
            &provider.named_time_zone_digest,
        )
    }
    fn admitted(index: i64) -> (EncodedDateTimePlan, AdmittedDateTimePlan) {
        let mut input = request(
            "en-US",
            DateTimeStyleSelection::Components(date_components()),
        );
        input.time_zone =
            TimeZoneSelection::FixedOffset(FixedTimeZoneOffset::from_seconds(index * 60).unwrap());
        let encoded = provider().select_plan(input).unwrap().plan;
        let admitted = provider().decode_plan(&encoded).unwrap().select().unwrap();
        (encoded, admitted)
    }

    #[test]
    fn exact_validated_plan_owner_is_reused_without_cloning_formats_across_threads() {
        let cache = empty_cache();
        let (key, entry) = admitted(0);
        let selected = Arc::clone(&entry.selected);
        cache.remember(entry);
        std::thread::scope(|scope| {
            for _ in 0..4 {
                let cache = &cache;
                let key = &key;
                let selected = &selected;
                scope.spawn(move || {
                    for _ in 0..64 {
                        assert!(Arc::ptr_eq(selected, &cache.get(key).unwrap()));
                    }
                });
            }
        });
        let (_, duplicate) = admitted(0);
        cache.remember(duplicate);
        assert!(Arc::ptr_eq(&selected, &cache.get(&key).unwrap()));
        assert_eq!(cache.entries.lock().unwrap().values.len(), 1);
        for offset in [0, 8, 40, 72, key.as_bytes().len() - 1] {
            let mut bytes = key.as_bytes().to_vec();
            bytes[offset] ^= 1;
            assert!(cache.get(&EncodedDateTimePlan::from_bytes(bytes)).is_none());
        }
        let oversized = EncodedDateTimePlan::from_bytes(vec![0; MAX_KEY_BYTES + 1]);
        assert!(cache.get(&oversized).is_none());
    }

    #[test]
    fn entry_eviction_keeps_a_live_immutable_plan_valid() {
        let cache = empty_cache();
        let (first, entry) = admitted(0);
        let live = Arc::clone(&entry.selected);
        cache.remember(entry);
        for index in 1..=MAX_ENTRIES {
            let (_, entry) = admitted(index as i64);
            cache.remember(entry);
        }
        assert!(cache.get(&first).is_none());
        let entries = cache.entries.lock().unwrap();
        assert_eq!(entries.values.len(), MAX_ENTRIES);
        assert!(entries.retained_bytes <= MAX_RETAINED_BYTES);
        assert_eq!(
            entries.retained_bytes,
            QUEUE_BYTES
                + entries
                    .values
                    .iter()
                    .map(|entry| entry.retained_bytes)
                    .sum::<usize>()
        );
        assert!(live.format(crate::DateTimeValueKind::Legacy).is_ok());
    }

    #[test]
    fn actual_owned_allocation_budget_evicts_and_oversized_plans_remain_uncached() {
        let mut cache = empty_cache();
        let (first_key, first) = admitted(0);
        let first_bytes =
            first.selected.retained_bytes() + first.encoded.len() + 2 * size_of::<usize>();
        cache.max_retained_bytes = QUEUE_BYTES + first_bytes * 2 - 1;
        cache.remember(first);
        let (second_key, second) = admitted(1);
        cache.remember(second);
        assert!(cache.get(&first_key).is_none());
        assert!(cache.get(&second_key).is_some());
        let entries = cache.entries.lock().unwrap();
        assert_eq!(entries.values.len(), 1);
        assert!(entries.retained_bytes <= cache.max_retained_bytes);
        drop(entries);
        let mut cold = empty_cache();
        cold.max_retained_bytes = QUEUE_BYTES;
        let (key, oversized) = admitted(0);
        cold.remember(oversized);
        assert!(cold.get(&key).is_none());
        assert_eq!(cold.entries.lock().unwrap().retained_bytes, QUEUE_BYTES);
    }
}
