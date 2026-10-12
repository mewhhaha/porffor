//! Collection-local incidence of independently live owners and keys.
use crate::prelude::*;
use crate::vm::{VMGcRef, WeakGcEdge, WeakGcRegistry};

struct Dependency {
    original: u32,
    slot: usize,
    gate: u8,
}

/// Raw old-space indices never participate in the root census. All storage is
/// reserved before the heap flips, and first-copy notifications cannot allocate.
#[derive(Default)]
pub(super) struct EphemeronMarking {
    dependencies: Vec<Dependency>,
    live_gates: Vec<u8>,
    ready: Vec<usize>,
}

impl EphemeronMarking {
    pub(super) fn new(registry: &WeakGcRegistry) -> Result<Self> {
        let count = registry
            .slots
            .iter()
            .filter(|slot| {
                matches!(
                    slot.edge,
                    Some(WeakGcEdge::Ephemeron {
                        key: Some(_),
                        value: Some(_),
                        ..
                    })
                )
            })
            .count();
        if count == 0 {
            return Ok(Self::default());
        }
        let mut marking = Self::default();
        marking.dependencies.try_reserve_exact(
            count
                .checked_mul(2)
                .ok_or_else(|| format_err!("ephemeron dependency count overflow"))?,
        )?;
        marking.live_gates.try_reserve_exact(registry.slots.len())?;
        marking.live_gates.resize(registry.slots.len(), 0);
        marking.ready.try_reserve_exact(count)?;
        for (slot, entry) in registry.slots.iter().enumerate() {
            if let Some(WeakGcEdge::Ephemeron {
                owner,
                key: Some(key),
                value: Some(_),
            }) = entry.edge.as_ref()
            {
                marking.dependencies.push(Dependency {
                    original: owner.as_raw_u32(),
                    slot,
                    gate: 1,
                });
                marking.dependencies.push(Dependency {
                    original: key.as_raw_u32(),
                    slot,
                    gate: 2,
                });
            }
        }
        marking
            .dependencies
            .sort_unstable_by_key(|dependency| dependency.original);
        Ok(marking)
    }

    /// Called exactly when an original-space object is first copied. Equal
    /// owner/key identities satisfy both gates, while duplicate slots remain
    /// independent. Each conditional edge enters the queue at most once.
    pub(super) fn copied(&mut self, original: &VMGcRef) {
        let raw = original.as_raw_u32();
        let first = self
            .dependencies
            .partition_point(|dependency| dependency.original < raw);
        for dependency in self.dependencies[first..]
            .iter()
            .take_while(|dependency| dependency.original == raw)
        {
            let gates = &mut self.live_gates[dependency.slot];
            let previous = *gates;
            *gates |= dependency.gate;
            if previous != 3 && *gates == 3 {
                debug_assert!(self.ready.len() < self.ready.capacity());
                self.ready.push(dependency.slot);
            }
        }
    }

    pub(super) fn pop(&mut self) -> Option<usize> {
        self.ready.pop()
    }

    pub(super) fn is_empty(&self) -> bool {
        self.ready.is_empty()
    }
}
