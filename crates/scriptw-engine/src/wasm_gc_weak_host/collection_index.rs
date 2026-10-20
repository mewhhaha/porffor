//! Epoch-scoped non-rooting lookup for one native weak collection resource.
use super::*;
use std::collections::HashMap;
use wasmtime::{GcWeakHeapEpoch, GcWeakKeyFingerprint};

#[derive(Default)]
pub(super) struct Entries {
    pub(super) entries: HashMap<GcWeakKeyFingerprint, GcEphemeron>,
    epoch: Option<GcWeakHeapEpoch>,
}

impl Entries {
    pub(super) fn find(
        &mut self,
        caller: &mut WasmtimeCaller<'_, WasmHostState>,
        key: &Rooted<AnyRef>,
    ) -> wasmtime::Result<Option<GcWeakKeyFingerprint>> {
        let wanted = GcWeakKeyFingerprint::new(&mut *caller, key)?;
        if self.epoch != Some(wanted.epoch()) {
            // GC updates the registry's weak references before the mutator
            // resumes. Rebuild once from surviving handles, transactionally:
            // allocation/validation failure preserves the existing table.
            let mut next = HashMap::new();
            next.try_reserve(self.entries.len())
                .map_err(wasmtime::Error::new)?;
            for entry in self.entries.values() {
                if let Some(current) = entry.key_fingerprint(&mut *caller)? {
                    if next.insert(current, *entry).is_some() {
                        return Err(wasmtime::Error::msg("duplicate weak collection key"));
                    }
                }
            }
            self.entries = next;
            self.epoch = Some(wanted.epoch());
        }
        let Some(entry) = self.entries.get(&wanted) else {
            return Ok(None);
        };
        if !entry.matches_key(&mut *caller, key)? {
            return Err(wasmtime::Error::msg(
                "weak collection key index identity mismatch",
            ));
        }
        Ok(Some(wanted))
    }

    pub(super) fn insert(
        &mut self,
        caller: &mut WasmtimeCaller<'_, WasmHostState>,
        owner: &Rooted<AnyRef>,
        key: &Rooted<AnyRef>,
        value: Option<&Rooted<AnyRef>>,
    ) -> wasmtime::Result<()> {
        if let Some(found) = self.find(caller, key)? {
            return self.entries[&found].set_value(&mut *caller, value.map(|value| &**value));
        }
        let fingerprint = GcWeakKeyFingerprint::new(&mut *caller, key)?;
        // Reserve before registering a conditional edge, so allocation cannot
        // orphan a value-retaining collector slot. Registration cannot collect.
        self.entries.try_reserve(1).map_err(wasmtime::Error::new)?;
        let entry = GcEphemeron::new(&mut *caller, owner, key, value.map(|value| &**value))?;
        let previous = self.entries.insert(fingerprint, entry);
        debug_assert!(previous.is_none());
        Ok(())
    }

    pub(super) fn remove(
        &mut self,
        caller: &mut WasmtimeCaller<'_, WasmHostState>,
        fingerprint: GcWeakKeyFingerprint,
    ) -> wasmtime::Result<()> {
        self.entries[&fingerprint].remove(&mut *caller)?;
        self.entries.remove(&fingerprint);
        Ok(())
    }
}
