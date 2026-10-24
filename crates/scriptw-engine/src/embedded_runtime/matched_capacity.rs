//! Capacity changes alter Wasmtime's serialized-module compatibility. Produce
//! matching optimized R images for every explicit cohort, including the 16 MiB
//! control, so GC comparisons share the same optimization/storage treatment. Only compiler-produced bytes enter
//! this private owner; it accepts no caller image, path or serialized cache blob.
use super::*;
use crate::{WasmProfileInlining, WasmProfileInliningLimits, WasmRuntimeMatchedImageCacheOutcome};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

const MAX_IMAGES: usize = 2;
const MAX_IMAGE_BYTES: usize = 256 * 1024 * 1024;

/// Mode is observed from the actual destination. Bounds are pinned source
/// facts: supported Config constructors expose no threshold overrides/getters.
/// Producer and destination use those same defaults, and Wasmtime separately
/// checks the serialized exact tunables before admitting native machine code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InliningCompatibility {
    mode: WasmProfileInlining,
    limits: WasmProfileInliningLimits,
}

impl InliningCompatibility {
    fn for_engine(engine: &Engine) -> Result<Self, crate::EngineError> {
        Ok(Self {
            mode: WasmProfileInlining::observe(engine)?,
            limits: crate::runtime_profile::PINNED_WASMTIME_INLINING_LIMITS,
        })
    }
}

struct Image {
    runtime: scriptw_aot_wasm::RuntimeArtifactKey,
    capacity: u64,
    inlining: InliningCompatibility,
    bytes: Arc<[u8]>,
}

struct Images {
    retained: VecDeque<Image>,
    bytes: usize,
    max_images: usize,
    max_bytes: usize,
}

impl Default for Images {
    fn default() -> Self {
        Self {
            retained: VecDeque::new(),
            bytes: 0,
            max_images: MAX_IMAGES,
            max_bytes: MAX_IMAGE_BYTES,
        }
    }
}

impl Images {
    fn get(
        &mut self,
        runtime: scriptw_aot_wasm::RuntimeArtifactKey,
        capacity: u64,
        inlining: InliningCompatibility,
    ) -> Option<Arc<[u8]>> {
        let index = self.retained.iter().position(|entry| {
            entry.runtime == runtime && entry.capacity == capacity && entry.inlining == inlining
        })?;
        let image = self
            .retained
            .remove(index)
            .expect("matched image key exists");
        let bytes = Arc::clone(&image.bytes);
        self.retained.push_back(image);
        Some(bytes)
    }

    fn retain(&mut self, image: Image) -> Arc<[u8]> {
        if let Some(existing) = self.get(image.runtime, image.capacity, image.inlining) {
            return existing;
        }
        if image.bytes.len() > self.max_bytes {
            return image.bytes;
        }
        while self.retained.len() >= self.max_images
            || self.bytes > self.max_bytes - image.bytes.len()
        {
            let removed = self
                .retained
                .pop_front()
                .expect("full matched image cache has an entry");
            self.bytes = self
                .bytes
                .checked_sub(removed.bytes.len())
                .expect("matched image bytes belong to owner");
        }
        self.bytes = self
            .bytes
            .checked_add(image.bytes.len())
            .expect("admitted matched image fits its byte budget");
        let bytes = Arc::clone(&image.bytes);
        self.retained.push_back(image);
        bytes
    }
}

pub(crate) fn load(
    engine: &Engine,
    runtime: &RuntimeArtifact,
) -> Result<(Module, WasmRuntimeMatchedImageCacheOutcome), crate::EngineError> {
    let capacity = engine.get_gc_heap_initial_size();
    let inlining = InliningCompatibility::for_engine(engine)?;
    static IMAGES: OnceLock<Mutex<Images>> = OnceLock::new();
    let images = IMAGES.get_or_init(Default::default);
    let cached = images
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .get(runtime.key(), capacity, inlining);
    let (bytes, outcome) = match cached {
        Some(bytes) => (bytes, WasmRuntimeMatchedImageCacheOutcome::Hit),
        None => {
            let modes = manifest::COMPILATION_MODES;
            let mut config = crate::product_wasmtime_config(modes.runtime)
                .map_err(crate::EngineError::from_wasmtime_setup)?;
            config.gc_heap_initial_size(capacity);
            config.cranelift_regalloc_algorithm(modes.register_allocation.algorithm());
            inlining.mode.configure(&mut config);
            let producer = Engine::new(&config).map_err(|error| {
                crate::EngineError::new(format!(
                    "capacity-matched runtime compiler setup failed: {error:#}"
                ))
            })?;
            crate::PRODUCT_WASMTIME_POLICY
                .verify_engine(&producer)
                .map_err(crate::EngineError::new)?;
            if InliningCompatibility::for_engine(&producer)? != inlining {
                return Err(crate::EngineError::new(
                    "matched R compiler inlining differs from the destination Engine",
                ));
            }
            // This uses the normal checked native module/function disk caches.
            // Compilation remains on the existing bounded native worker pool.
            let module = crate::compile_wasm_module(&producer, runtime.bytes())?;
            let bytes: Arc<[u8]> = module
                .serialize()
                .map_err(|error| {
                    crate::EngineError::new(format!(
                        "capacity-matched runtime serialization failed: {error:#}"
                    ))
                })?
                .into();
            let outcome = if bytes.len() > MAX_IMAGE_BYTES {
                WasmRuntimeMatchedImageCacheOutcome::Oversized
            } else {
                WasmRuntimeMatchedImageCacheOutcome::Miss
            };
            let bytes = images
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .retain(Image {
                    runtime: runtime.key(),
                    capacity,
                    inlining,
                    bytes,
                });
            (bytes, outcome)
        }
    };
    // SAFETY: every byte comes directly from Module::serialize on this pinned
    // compiler, from the admitted runtime's actual Wasm, under the same required
    // capability policy and capacity as the destination. The private cache key
    // binds runtime source/profile/layout identity, capacity and the inlining
    // mode/pinned-bound tuple. Wasmtime still
    // checks native version, target, tunables, features and Engine compatibility;
    // Module::deserialize owns its mapping independently of this Arc's lifetime.
    let module = unsafe { Module::deserialize(engine, &bytes) }.map_err(|error| {
        crate::EngineError::new(format!(
            "capacity-matched native runtime admission failed: {error:#}"
        ))
    })?;
    Ok((module, outcome))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> scriptw_aot_wasm::RuntimeArtifactKey {
        let selection =
            scriptw_intl::IntlDataSelection::new(scriptw_intl::IntlCompilationProfile::default());
        scriptw_aot_wasm::runtime_artifact_with_inputs(
            &selection,
            super::super::inputs(None, RuntimeSourceCompilationProfile::NativeHost),
        )
        .unwrap()
        .key()
    }

    #[test]
    fn matching_image_cache_bounds_bytes_promotes_hits_and_does_not_retain_oversized_images() {
        let key = key();
        let inlining = InliningCompatibility {
            mode: WasmProfileInlining::Disabled,
            limits: crate::runtime_profile::PINNED_WASMTIME_INLINING_LIMITS,
        };
        let mut cache = Images {
            max_images: 2,
            max_bytes: 8,
            ..Images::default()
        };
        let image = |capacity, count| Image {
            runtime: key,
            capacity,
            inlining,
            bytes: vec![capacity as u8; count].into(),
        };
        cache.retain(image(16, 3));
        cache.retain(image(32, 3));
        assert_eq!(cache.bytes, 6);
        assert_eq!(cache.get(key, 16, inlining).unwrap().as_ref(), &[16; 3]);
        cache.retain(image(64, 5));
        assert!(cache.get(key, 32, inlining).is_none());
        assert_eq!(cache.bytes, 8);
        assert_eq!(cache.retained.len(), 2);
        let oversized = cache.retain(image(128, 9));
        assert_eq!(oversized.len(), 9);
        assert!(cache.get(key, 128, inlining).is_none());
        assert_eq!(cache.bytes, 8);
        let winner = cache.retain(image(64, 2));
        assert_eq!(winner.len(), 5);
        assert_eq!(cache.bytes, 8);
        assert_eq!(
            cache
                .retained
                .iter()
                .map(|image| image.bytes.len())
                .sum::<usize>(),
            cache.bytes
        );
    }

    #[test]
    fn matched_images_separate_actual_inlining_modes_within_one_bounded_owner() {
        let key = key();
        let disabled = InliningCompatibility {
            mode: WasmProfileInlining::Disabled,
            limits: crate::runtime_profile::PINNED_WASMTIME_INLINING_LIMITS,
        };
        let bounded = InliningCompatibility {
            mode: WasmProfileInlining::Bounded,
            ..disabled
        };
        let mut cache = Images {
            max_images: 2,
            max_bytes: 8,
            ..Images::default()
        };
        cache.retain(Image {
            runtime: key,
            capacity: 16,
            inlining: disabled,
            bytes: vec![1; 3].into(),
        });
        assert!(cache.get(key, 16, bounded).is_none());
        cache.retain(Image {
            runtime: key,
            capacity: 16,
            inlining: bounded,
            bytes: vec![2; 4].into(),
        });
        assert_eq!(cache.get(key, 16, disabled).unwrap().as_ref(), &[1; 3]);
        assert_eq!(cache.get(key, 16, bounded).unwrap().as_ref(), &[2; 4]);
        assert_eq!(cache.bytes, 7);
        assert_eq!(cache.retained.len(), 2);
        cache.retain(Image {
            runtime: key,
            capacity: 32,
            inlining: bounded,
            bytes: vec![3; 4].into(),
        });
        assert!(cache.get(key, 16, disabled).is_none());
        assert_eq!(cache.bytes, 8);
        assert_eq!(cache.retained.len(), 2);
    }
}
