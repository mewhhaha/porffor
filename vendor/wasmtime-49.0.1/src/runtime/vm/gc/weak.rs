//! Store-owned weak edges for the copying collector.
//!
//! These are native GC references, deliberately outside the ordinary root
//! iterator. Only `kept` and previously scheduled cleanups are strong roots.
//! Owners are weak: this registry must never keep a dead WeakMap alive. The
//! collector traces conditional edges to a fixed point before clearing weak
//! targets, and rewrites every surviving reference before reusing from-space.

use super::VMGcRef;
use crate::hash_map::HashMap;
use crate::prelude::*;
use alloc::collections::VecDeque;

pub(crate) trait WeakGcTrace {
    /// Return the new reference if already marked, without retaining it.
    fn forwarded(&self, reference: &VMGcRef) -> Result<Option<VMGcRef>>;
    /// Retain this reference, accepting both old and already moved references.
    fn retain(&mut self, reference: &VMGcRef) -> Result<VMGcRef>;
}

pub(crate) enum WeakGcEdges {
    Reference(Option<VMGcRef>),
    Ephemerons(HashMap<u32, (VMGcRef, VMGcRef)>),
    Finalization(Vec<FinalizationCell>),
}

pub(crate) struct FinalizationCell {
    pub target: VMGcRef,
    pub holding: VMGcRef,
    pub token: Option<VMGcRef>,
}

pub(crate) struct PendingGcCleanup {
    pub owner: VMGcRef,
    pub holding: VMGcRef,
    pub token: Option<VMGcRef>,
}

struct WeakGcOwner {
    owner: VMGcRef,
    edges: WeakGcEdges,
}

#[derive(Default)]
pub(crate) struct GcWeakStore {
    // The address is only a lookup cache inside the runtime, never a guest
    // handle. It is rebuilt after movement; each record owns the actual ref.
    owners: HashMap<u32, WeakGcOwner>,
    kept: HashMap<u32, VMGcRef>,
    pending: VecDeque<PendingGcCleanup>,
}

impl GcWeakStore {
    pub fn insert(&mut self, owner: VMGcRef, edges: WeakGcEdges) -> Result<()> {
        use crate::hash_map::Entry;
        match self.owners.entry(owner.as_raw_u32()) {
            Entry::Vacant(entry) => {
                entry.insert(WeakGcOwner { owner, edges });
                Ok(())
            }
            Entry::Occupied(_) => bail!("native GC weak owner is already initialized"),
        }
    }

    pub fn edges(&mut self, owner: &VMGcRef) -> Result<&mut WeakGcEdges> {
        self.owners
            .get_mut(&owner.as_raw_u32())
            .map(|owner| &mut owner.edges)
            .ok_or_else(|| format_err!("native GC weak owner is not initialized"))
    }

    pub fn keep(&mut self, target: VMGcRef) {
        self.kept.entry(target.as_raw_u32()).or_insert(target);
    }

    pub fn clear_kept(&mut self) {
        self.kept.clear();
    }

    pub fn pop_cleanup(&mut self) -> Option<PendingGcCleanup> {
        self.pending.pop_front()
    }

    pub fn pop_holding(&mut self, owner: &VMGcRef) -> Result<Option<VMGcRef>> {
        let WeakGcEdges::Finalization(_) = self.edges(owner)? else {
            bail!("native GC weak owner is not a finalization registry");
        };
        let owner_id = owner.as_raw_u32();
        let next = self
            .pending
            .iter()
            .position(|cell| cell.owner.as_raw_u32() == owner_id);
        Ok(next.map(|index| self.pending.remove(index).unwrap().holding))
    }

    pub fn unregister(&mut self, owner: &VMGcRef, token: &VMGcRef) -> Result<bool> {
        let WeakGcEdges::Finalization(cells) = self.edges(owner)? else {
            bail!("native GC weak owner is not a finalization registry");
        };
        let owner_id = owner.as_raw_u32();
        let token_id = token.as_raw_u32();
        let before = cells.len();
        cells.retain(|cell| cell.token.as_ref().map(VMGcRef::as_raw_u32) != Some(token_id));
        let mut removed = cells.len() != before;
        let before = self.pending.len();
        self.pending.retain(|cell| {
            cell.owner.as_raw_u32() != owner_id
                || cell.token.as_ref().map(VMGcRef::as_raw_u32) != Some(token_id)
        });
        removed |= self.pending.len() != before;
        Ok(removed)
    }

    /// These roots are explicit job-lifetime roots, not weak edge endpoints.
    pub fn trace_roots(&mut self, tracer: &mut impl WeakGcTrace) -> Result<()> {
        for reference in self.kept.values_mut() {
            *reference = tracer.retain(reference)?;
        }
        for cleanup in &mut self.pending {
            cleanup.owner = tracer.retain(&cleanup.owner)?;
            cleanup.holding = tracer.retain(&cleanup.holding)?;
        }
        Ok(())
    }

    /// Called after draining the ordinary worklist. New grey objects require
    /// another ordinary scan and another conditional scan before weak clearing.
    pub fn trace_conditionals(&mut self, tracer: &mut impl WeakGcTrace) -> Result<()> {
        for record in self.owners.values_mut() {
            if tracer.forwarded(&record.owner)?.is_none() {
                continue;
            }
            match &mut record.edges {
                WeakGcEdges::Reference(_) => {}
                WeakGcEdges::Ephemerons(entries) => {
                    for (key, value) in entries.values_mut() {
                        if tracer.forwarded(key)?.is_some() {
                            *value = tracer.retain(value)?;
                        }
                    }
                }
                WeakGcEdges::Finalization(cells) => {
                    // Holdings are strong while their registry is live, even
                    // before the target dies. A holding may in turn retain a
                    // different target or an ephemeron key.
                    for cell in cells {
                        cell.holding = tracer.retain(&cell.holding)?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Run only after the strong/ephemeron fixed point. In particular a dead
    /// key/value cycle must not become live by following its own weak edge.
    pub fn sweep(&mut self, tracer: &impl WeakGcTrace) -> Result<()> {
        let mut owners = HashMap::default();
        for (_, mut record) in core::mem::take(&mut self.owners) {
            let Some(owner) = tracer.forwarded(&record.owner)? else {
                continue;
            };
            record.owner = owner;
            match &mut record.edges {
                WeakGcEdges::Reference(target) => {
                    *target = target
                        .as_ref()
                        .map(|target| tracer.forwarded(target))
                        .transpose()?
                        .flatten();
                }
                WeakGcEdges::Ephemerons(entries) => {
                    let mut live = HashMap::default();
                    for (_, (key, value)) in core::mem::take(entries) {
                        if let Some(key) = tracer.forwarded(&key)? {
                            let value = tracer.forwarded(&value)?.ok_or_else(|| {
                                format_err!("live ephemeron lost its value during collection")
                            })?;
                            live.insert(key.as_raw_u32(), (key, value));
                        }
                    }
                    *entries = live;
                }
                WeakGcEdges::Finalization(cells) => {
                    let mut live = Vec::new();
                    for cell in core::mem::take(cells) {
                        let holding = tracer.forwarded(&cell.holding)?.ok_or_else(|| {
                            format_err!("live finalization registry lost its holding")
                        })?;
                        let token = cell
                            .token
                            .as_ref()
                            .map(|token| tracer.forwarded(token))
                            .transpose()?
                            .flatten();
                        if let Some(target) = tracer.forwarded(&cell.target)? {
                            live.push(FinalizationCell {
                                target,
                                holding,
                                token,
                            });
                        } else {
                            self.pending.push_back(PendingGcCleanup {
                                owner: record.owner.unchecked_copy(),
                                holding,
                                token,
                            });
                        }
                    }
                    *cells = live;
                }
            }
            owners.insert(record.owner.as_raw_u32(), record);
        }
        self.owners = owners;
        // Existing queued tokens are weak as well. New queued tokens above
        // already point into to-space; forwarded accepts both spaces.
        for cleanup in &mut self.pending {
            cleanup.token = cleanup
                .token
                .as_ref()
                .map(|token| tracer.forwarded(token))
                .transpose()?
                .flatten();
        }
        self.kept = core::mem::take(&mut self.kept)
            .into_values()
            .map(|reference| (reference.as_raw_u32(), reference))
            .collect();
        Ok(())
    }
}
