//! Native weak reachability extension used by Lila's Wasmtime target.
//!
//! Every owner, key, target and value is a real, rooted Wasm reference at the
//! API boundary. The runtime stores weak edges outside the ordinary root set
//! and processes them as part of its copying collection. No guest pointer or
//! integer object identity crosses this interface.

use crate::prelude::*;
use crate::runtime::vm::{FinalizationCell, GcWeakStore, VMGcRef, WeakGcEdges};
use crate::store::{AutoAssertNoGc, StoreOpaque};
use crate::{AnyRef, AsContextMut, Rooted};

/// Version of Lila's vendored native weak-reachability extension.
/// This facility is not part of the WebAssembly GC specification or upstream
/// Wasmtime's public API.
pub const NATIVE_WEAK_GC_ABI_VERSION: u32 = 1;

fn reference(store: &StoreOpaque, reference: &AnyRef, must_be_heap: bool) -> Result<VMGcRef> {
    ensure!(
        reference.comes_from_same_store(store),
        "GC weak edge belongs to another Store"
    );
    let reference = reference.inner.try_gc_ref(store)?;
    ensure!(
        !must_be_heap || !reference.is_i31(),
        "i31 cannot be a GC weak owner, key, target or token"
    );
    // Only the copying collector accepts these references. It has no clone or
    // drop barriers; ordinary Rooted/OwnedRooted are deliberately not stored.
    Ok(reference.unchecked_copy())
}

fn weak<'a>(store: &'a mut AutoAssertNoGc<'_>) -> Result<&'a mut GcWeakStore> {
    let gc = store.require_gc_store_mut()?;
    ensure!(
        gc.gc_heap.supports_native_weak_edges(),
        "native weak GC requires the copying collector"
    );
    Ok(&mut gc.weak)
}

/// Weak reference operations on a native GC owner. The weak relationship must
/// not also be stored in a strong field. Ordinary strong fields, including
/// other references to this same target, retain their normal tracing semantics.
/// Initialization and successful reads keep the target alive until the host
/// calls [`GcWeakHeap::clear_kept_objects`] at the end of its current job.
pub struct GcWeakRef;

impl GcWeakRef {
    /// Associate a previously unregistered owner with a weak target.
    pub fn initialize(mut store: impl AsContextMut, owner: &AnyRef, target: &AnyRef) -> Result<()> {
        let mut store = AutoAssertNoGc::new(store.as_context_mut().0);
        let owner = reference(&store, owner, true)?;
        let target = reference(&store, target, true)?;
        let weak = weak(&mut store)?;
        weak.insert(owner, WeakGcEdges::Reference(Some(target.unchecked_copy())))?;
        weak.keep(target);
        Ok(())
    }

    /// Return a rooted live target, or `None` after collection has cleared it.
    pub fn get(mut store: impl AsContextMut, owner: &AnyRef) -> Result<Option<Rooted<AnyRef>>> {
        let mut store = AutoAssertNoGc::new(store.as_context_mut().0);
        let owner = reference(&store, owner, true)?;
        let weak = weak(&mut store)?;
        let WeakGcEdges::Reference(target) = weak.edges(&owner)? else {
            bail!("native GC weak owner is not a weak reference");
        };
        let target = target.as_ref().map(VMGcRef::unchecked_copy);
        if let Some(target) = &target {
            weak.keep(target.unchecked_copy());
        }
        Ok(target.map(|target| AnyRef::from_cloned_gc_ref(&mut store, target)))
    }
}

/// Ephemeron operations. A value is retained only when both this native owner
/// and its key are reachable; a value-to-key cycle cannot retain itself.
pub struct GcEphemeronTable;

impl GcEphemeronTable {
    /// Initialize an empty table for a previously unregistered GC owner.
    pub fn initialize(mut store: impl AsContextMut, owner: &AnyRef) -> Result<()> {
        let mut store = AutoAssertNoGc::new(store.as_context_mut().0);
        let owner = reference(&store, owner, true)?;
        weak(&mut store)?.insert(owner, WeakGcEdges::Ephemerons(Default::default()))
    }

    /// Insert or replace a value. Keys must be heap references; boxed or i31
    /// values are allowed. Languages with nullable/scalar values should use
    /// their stored-value representation for values at this boundary.
    pub fn set(
        mut store: impl AsContextMut,
        owner: &AnyRef,
        key: &AnyRef,
        value: &AnyRef,
    ) -> Result<()> {
        let mut store = AutoAssertNoGc::new(store.as_context_mut().0);
        let owner = reference(&store, owner, true)?;
        let key = reference(&store, key, true)?;
        let value = reference(&store, value, false)?;
        let WeakGcEdges::Ephemerons(entries) = weak(&mut store)?.edges(&owner)? else {
            bail!("native GC weak owner is not an ephemeron table");
        };
        entries.insert(key.as_raw_u32(), (key, value));
        Ok(())
    }

    /// Look up a key, rooting the value before returning to the caller.
    pub fn get(
        mut store: impl AsContextMut,
        owner: &AnyRef,
        key: &AnyRef,
    ) -> Result<Option<Rooted<AnyRef>>> {
        let mut store = AutoAssertNoGc::new(store.as_context_mut().0);
        let owner = reference(&store, owner, true)?;
        let key = reference(&store, key, true)?;
        let WeakGcEdges::Ephemerons(entries) = weak(&mut store)?.edges(&owner)? else {
            bail!("native GC weak owner is not an ephemeron table");
        };
        let value = entries
            .get(&key.as_raw_u32())
            .map(|(_, value)| value.unchecked_copy());
        Ok(value.map(|value| AnyRef::from_cloned_gc_ref(&mut store, value)))
    }

    /// Delete a key. Returns whether the entry existed.
    pub fn delete(mut store: impl AsContextMut, owner: &AnyRef, key: &AnyRef) -> Result<bool> {
        let mut store = AutoAssertNoGc::new(store.as_context_mut().0);
        let owner = reference(&store, owner, true)?;
        let key = reference(&store, key, true)?;
        let WeakGcEdges::Ephemerons(entries) = weak(&mut store)?.edges(&owner)? else {
            bail!("native GC weak owner is not an ephemeron table");
        };
        Ok(entries.remove(&key.as_raw_u32()).is_some())
    }
}

/// Native finalization registrations. Holding values are strong while the
/// owner lives, targets and unregister tokens are weak. Collection only queues
/// cleanup records; it never invokes guest code or host cleanup callbacks.
pub struct GcFinalizationRegistry;

impl GcFinalizationRegistry {
    /// Initialize a previously unregistered native registry owner.
    pub fn initialize(mut store: impl AsContextMut, owner: &AnyRef) -> Result<()> {
        let mut store = AutoAssertNoGc::new(store.as_context_mut().0);
        let owner = reference(&store, owner, true)?;
        weak(&mut store)?.insert(owner, WeakGcEdges::Finalization(Vec::new()))
    }

    /// Register a target and holding, with an optional weak unregister token.
    /// Multiple cells for the same target or token are allowed.
    pub fn register(
        mut store: impl AsContextMut,
        owner: &AnyRef,
        target: &AnyRef,
        holding: &AnyRef,
        token: Option<&AnyRef>,
    ) -> Result<()> {
        let mut store = AutoAssertNoGc::new(store.as_context_mut().0);
        let owner = reference(&store, owner, true)?;
        let target = reference(&store, target, true)?;
        let holding = reference(&store, holding, false)?;
        let token = token
            .map(|token| reference(&store, token, true))
            .transpose()?;
        ensure!(
            target.as_raw_u32() != holding.as_raw_u32(),
            "finalization target cannot equal its holding"
        );
        let WeakGcEdges::Finalization(cells) = weak(&mut store)?.edges(&owner)? else {
            bail!("native GC weak owner is not a finalization registry");
        };
        cells.push(FinalizationCell {
            target,
            holding,
            token,
        });
        Ok(())
    }

    /// Remove every registration and queued holding matching this token.
    pub fn unregister(
        mut store: impl AsContextMut,
        owner: &AnyRef,
        token: &AnyRef,
    ) -> Result<bool> {
        let mut store = AutoAssertNoGc::new(store.as_context_mut().0);
        let owner = reference(&store, owner, true)?;
        let token = reference(&store, token, true)?;
        weak(&mut store)?.unregister(&owner, &token)
    }

    /// Take the next holding for a registry whose cleanup job is already
    /// running. Call this after each successful callback, rather than removing
    /// all cells in advance: a callback may unregister the remaining cells.
    /// Other registries' queued cells are left untouched. If a callback throws,
    /// stop draining; undelivered holdings remain queued for later cleanup.
    pub fn take_holding(
        mut store: impl AsContextMut,
        owner: &AnyRef,
    ) -> Result<Option<Rooted<AnyRef>>> {
        let mut store = AutoAssertNoGc::new(store.as_context_mut().0);
        let owner = reference(&store, owner, true)?;
        let holding = weak(&mut store)?.pop_holding(&owner)?;
        Ok(holding.map(|holding| AnyRef::from_cloned_gc_ref(&mut store, holding)))
    }
}

/// A cleanup record returned outside collection. Both references are rooted
/// in the caller's current scope before the queue relinquishes ownership.
pub struct GcFinalizationCleanup {
    /// Registry whose language-level cleanup callback should receive `holding`.
    pub owner: Rooted<AnyRef>,
    /// The registration's held value.
    pub holding: Rooted<AnyRef>,
}

/// Job lifetime and cleanup queue operations shared by native weak facilities.
pub struct GcWeakHeap;

impl GcWeakHeap {
    /// Release WeakRef's kept-alive set at a language job boundary. This does
    /// not itself collect, and must never run inside a still-active job.
    pub fn clear_kept_objects(mut store: impl AsContextMut) -> Result<()> {
        let mut store = AutoAssertNoGc::new(store.as_context_mut().0);
        // Clearing an unallocated heap is harmless and must not allocate it.
        if store.optional_gc_store_mut().is_some() {
            weak(&mut store)?.clear_kept();
        }
        Ok(())
    }

    /// Start a registry's cleanup after collection. The host owns scheduling and
    /// callback exceptions, and must finish the current language job first.
    /// After delivering the first holding, use
    /// [`GcFinalizationRegistry::take_holding`] to drain this same registry one
    /// callback at a time before selecting another registry's cleanup job.
    pub fn take_cleanup(mut store: impl AsContextMut) -> Result<Option<GcFinalizationCleanup>> {
        let mut store = AutoAssertNoGc::new(store.as_context_mut().0);
        if store.optional_gc_store_mut().is_none() {
            return Ok(None);
        }
        Ok(weak(&mut store)?
            .pop_cleanup()
            .map(|cleanup| GcFinalizationCleanup {
                owner: AnyRef::from_cloned_gc_ref(&mut store, cleanup.owner),
                holding: AnyRef::from_cloned_gc_ref(&mut store, cleanup.holding),
            }))
    }
}
