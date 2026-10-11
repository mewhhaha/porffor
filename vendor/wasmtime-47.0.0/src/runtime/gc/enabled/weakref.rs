//! Store-scoped handles into collector-owned weak edges.
use crate::prelude::*;
use crate::store::{AutoAssertNoGc, StoreId};
use crate::vm::{VMGcRef, WeakGcEdge, WeakGcSlotId};
use crate::{AnyRef, AsContextMut, Rooted};

#[derive(Debug, Clone, Copy)]
struct WeakHandle {
    store: StoreId,
    slot: WeakGcSlotId,
}

impl WeakHandle {
    fn check(&self, store: &AutoAssertNoGc<'_>) -> Result<()> {
        ensure!(
            self.store == store.id(),
            "weak handle used with wrong Store"
        );
        Ok(())
    }
}

fn reference(store: &AutoAssertNoGc<'_>, reference: &AnyRef) -> Result<VMGcRef> {
    ensure!(
        reference.inner.comes_from_same_store(store),
        "GC reference used with wrong Store"
    );
    Ok(reference.inner.try_gc_ref(store)?.unchecked_copy())
}

fn heap_reference(store: &AutoAssertNoGc<'_>, value: &AnyRef) -> Result<VMGcRef> {
    let reference = reference(store, value)?;
    ensure!(
        !reference.is_i31(),
        "weak owners and targets require heap objects"
    );
    Ok(reference)
}

fn insert(store: &mut AutoAssertNoGc<'_>, edge: WeakGcEdge) -> Result<WeakHandle> {
    let id = store.id();
    let gc = store.unwrap_gc_store_mut();
    ensure!(
        gc.gc_heap.supports_weak_refs(),
        "selected collector has no weak/ephemeron facility"
    );
    Ok(WeakHandle {
        store: id,
        slot: gc.weak_gc.insert(edge)?,
    })
}

/// A weak target whose entry is retired when its owner dies.
/// The handle never exposes an unrooted reference. Upgrades are atomic with
/// respect to collection and return a scoped strong root in the given Store.
/// This copyable handle does not deregister on drop. Call `remove` when deleting
/// a registration; collection retires it when its owner becomes unreachable.
#[derive(Debug, Clone, Copy)]
pub struct WeakAnyRef {
    handle: WeakHandle,
}

impl WeakAnyRef {
    /// Register a target without retaining either target or owner strongly.
    /// Requires a collector that implements weak reachability.
    pub fn new(mut store: impl AsContextMut, owner: &AnyRef, target: &AnyRef) -> Result<Self> {
        let mut guard = AutoAssertNoGc::new(store.as_context_mut().0);
        let owner = heap_reference(&guard, owner)?;
        let target = heap_reference(&guard, target)?;
        Ok(Self {
            handle: insert(
                &mut guard,
                WeakGcEdge::Target {
                    owner,
                    target: Some(target),
                },
            )?,
        })
    }

    /// Upgrade a surviving target to a strong root. A cleared or retired target
    /// returns None. A handle used with a different Store returns an error.
    pub fn upgrade(&self, mut store: impl AsContextMut) -> Result<Option<Rooted<AnyRef>>> {
        let mut guard = AutoAssertNoGc::new(store.as_context_mut().0);
        self.handle.check(&guard)?;
        let gc = guard.unwrap_gc_store();
        if !gc.weak_gc.contains(self.handle.slot) {
            return Ok(None);
        }
        let value = match gc.weak_gc.get(self.handle.slot)? {
            WeakGcEdge::Target { target, .. } => target.as_ref().map(VMGcRef::unchecked_copy),
            WeakGcEdge::Ephemeron { .. } => unreachable!("typed weak handle cannot name ephemeron"),
        };
        Ok(value.map(|value| {
            let value = guard.clone_gc_ref(&value);
            Rooted::new(&mut guard, value)
        }))
    }

    /// Remove the weak entry. Repeated removal and a collected owner are safe.
    pub fn remove(&self, mut store: impl AsContextMut) -> Result<()> {
        let mut guard = AutoAssertNoGc::new(store.as_context_mut().0);
        self.handle.check(&guard)?;
        let gc = guard.unwrap_gc_store_mut();
        if gc.weak_gc.contains(self.handle.slot) {
            gc.weak_gc.remove(self.handle.slot)?;
        }
        Ok(())
    }
}

/// A conditional strong edge whose owner and key are both weak.
/// Values are marked only after owner and key become live independently, then
/// participate in the collector's ephemeron fixed point.
/// This copyable handle does not deregister on drop. Call `remove` when deleting
/// an entry; collection retires it when its owner or key becomes unreachable.
#[derive(Debug, Clone, Copy)]
pub struct GcEphemeron {
    handle: WeakHandle,
}

impl GcEphemeron {
    /// Whether the entry still has a surviving owner and key after collection.
    pub fn is_live(&self, mut store: impl AsContextMut) -> Result<bool> {
        let guard = AutoAssertNoGc::new(store.as_context_mut().0);
        self.handle.check(&guard)?;
        Ok(guard.unwrap_gc_store().weak_gc.contains(self.handle.slot))
    }

    /// Register a conditional value without rooting owner or key.
    pub fn new(
        mut store: impl AsContextMut,
        owner: &AnyRef,
        key: &AnyRef,
        value: Option<&AnyRef>,
    ) -> Result<Self> {
        let mut guard = AutoAssertNoGc::new(store.as_context_mut().0);
        let owner = heap_reference(&guard, owner)?;
        let key = heap_reference(&guard, key)?;
        let value = value.map(|value| reference(&guard, value)).transpose()?;
        Ok(Self {
            handle: insert(
                &mut guard,
                WeakGcEdge::Ephemeron {
                    owner,
                    key: Some(key),
                    value,
                },
            )?,
        })
    }

    /// Test actual key identity without exposing or rooting a weak key.
    pub fn matches_key(&self, mut store: impl AsContextMut, key: &AnyRef) -> Result<bool> {
        let guard = AutoAssertNoGc::new(store.as_context_mut().0);
        self.handle.check(&guard)?;
        let wanted = heap_reference(&guard, key)?;
        let gc = guard.unwrap_gc_store();
        if !gc.weak_gc.contains(self.handle.slot) {
            return Ok(false);
        }
        match gc.weak_gc.get(self.handle.slot)? {
            WeakGcEdge::Ephemeron { key, .. } => Ok(key.as_ref().is_some_and(|key| key == &wanted)),
            WeakGcEdge::Target { .. } => unreachable!("typed ephemeron handle cannot name target"),
        }
    }

    /// Upgrade the surviving value to a scoped strong root.
    pub fn value(&self, mut store: impl AsContextMut) -> Result<Option<Rooted<AnyRef>>> {
        let mut guard = AutoAssertNoGc::new(store.as_context_mut().0);
        self.handle.check(&guard)?;
        let gc = guard.unwrap_gc_store();
        if !gc.weak_gc.contains(self.handle.slot) {
            return Ok(None);
        }
        let value = match gc.weak_gc.get(self.handle.slot)? {
            WeakGcEdge::Ephemeron {
                key: Some(_),
                value,
                ..
            } => value.as_ref().map(VMGcRef::unchecked_copy),
            WeakGcEdge::Ephemeron { key: None, .. } => None,
            WeakGcEdge::Target { .. } => unreachable!("typed ephemeron handle cannot name target"),
        };
        Ok(value.map(|value| {
            let value = guard.clone_gc_ref(&value);
            Rooted::new(&mut guard, value)
        }))
    }

    /// Replace only the conditional value of a live entry.
    pub fn set_value(&self, mut store: impl AsContextMut, value: Option<&AnyRef>) -> Result<()> {
        let mut guard = AutoAssertNoGc::new(store.as_context_mut().0);
        self.handle.check(&guard)?;
        let value = value.map(|value| reference(&guard, value)).transpose()?;
        match guard
            .unwrap_gc_store_mut()
            .weak_gc
            .get_mut(self.handle.slot)?
        {
            WeakGcEdge::Ephemeron {
                key: Some(_),
                value: slot,
                ..
            } => *slot = value,
            WeakGcEdge::Ephemeron { key: None, .. } => bail!("ephemeron key has been cleared"),
            WeakGcEdge::Target { .. } => unreachable!("typed ephemeron handle cannot name target"),
        }
        Ok(())
    }

    /// Remove the conditional edge; stale generations cannot remove new edges.
    pub fn remove(&self, mut store: impl AsContextMut) -> Result<()> {
        let mut guard = AutoAssertNoGc::new(store.as_context_mut().0);
        self.handle.check(&guard)?;
        let gc = guard.unwrap_gc_store_mut();
        if gc.weak_gc.contains(self.handle.slot) {
            gc.weak_gc.remove(self.handle.slot)?;
        }
        Ok(())
    }
}
