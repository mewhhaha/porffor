use super::*;
use std::sync::Arc;

struct RetainedNativeInstance {
    key: WasmModuleMemoryCacheKey,
    code: Arc<[u8]>,
    instance: wasmtime::Instance,
    module: WasmtimeModule,
    authority: Arc<()>,
    retained_bytes: usize,
}

/// One actual Store owns this cache. Source operands live only on fresh P GC
/// owners; cached imports capture no source or mutable current-P selector.
pub(crate) struct MemoryNativeInstanceCache {
    entries: VecDeque<RetainedNativeInstance>,
    retained_bytes: usize,
    limits: MemoryModuleCacheLimits,
}

impl Default for MemoryNativeInstanceCache {
    fn default() -> Self {
        Self {
            entries: VecDeque::new(),
            retained_bytes: 0,
            limits: MemoryModuleCacheLimits::from_env(),
        }
    }
}

impl MemoryNativeInstanceCache {
    pub(crate) fn get(
        &mut self,
        key: &WasmModuleMemoryCacheKey,
        code: &[u8],
        engine: &crate::WasmtimeEngine,
        authority: &Arc<()>,
    ) -> Option<wasmtime::Instance> {
        let index = self.entries.iter().position(|entry| {
            entry.key == *key
                && entry.code.as_ref() == code
                && Arc::ptr_eq(&entry.authority, authority)
                && crate::WasmtimeEngine::same(entry.module.engine(), engine)
        })?;
        let entry = self
            .entries
            .remove(index)
            .expect("matching Store instance exists");
        let instance = entry.instance;
        self.entries.push_back(entry);
        Some(instance)
    }

    pub(crate) fn retain(
        &mut self,
        key: WasmModuleMemoryCacheKey,
        code: &[u8],
        module: WasmtimeModule,
        instance: wasmtime::Instance,
        authority: Arc<()>,
    ) {
        let Some(bytes) =
            module_image_bytes(&module).and_then(|bytes| bytes.checked_add(code.len()))
        else {
            return;
        };
        if bytes > self.limits.image_bytes.get() {
            return;
        }
        while self.entries.len() >= self.limits.entries.get()
            || !self.limits.has_image_room(self.retained_bytes, bytes)
        {
            let entry = self
                .entries
                .pop_front()
                .expect("a full instance cache has an entry");
            self.retained_bytes = self
                .retained_bytes
                .checked_sub(entry.retained_bytes)
                .expect("retired instance belongs to aggregate");
        }
        self.retained_bytes += bytes;
        self.entries.push_back(RetainedNativeInstance {
            key,
            code: Arc::from(code),
            instance,
            module,
            authority,
            retained_bytes: bytes,
        });
    }
}

#[cfg(test)]
mod tests;
