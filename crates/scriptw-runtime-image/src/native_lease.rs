//! Weak ownership shares concurrent loads without retaining decoded storage.
use super::NativeRuntimeBytes;
use std::sync::{Arc, Mutex, Weak};

// Failure and overlapping leases belong to one source profile only.
static NATIVE_HOST: Mutex<Result<Weak<NativeRuntimeBytes>, ()>> = Mutex::new(Ok(Weak::new()));
static PORTABLE: Mutex<Result<Weak<NativeRuntimeBytes>, ()>> = Mutex::new(Ok(Weak::new()));

pub(super) fn acquire_native_host() -> Option<Arc<NativeRuntimeBytes>> {
    // SAFETY: the Cargo target defines this exact immutable encoded array.
    acquire(&NATIVE_HOST, unsafe { &super::SCRIPTW_RUNTIME_NATIVE })
}

pub(super) fn acquire_portable() -> Option<Arc<NativeRuntimeBytes>> {
    // SAFETY: the Cargo target defines this exact immutable encoded array.
    acquire(&PORTABLE, unsafe {
        &super::SCRIPTW_RUNTIME_PORTABLE_NATIVE
    })
}

fn acquire(
    slot: &Mutex<Result<Weak<NativeRuntimeBytes>, ()>>,
    encoded: &'static [u8],
) -> Option<Arc<NativeRuntimeBytes>> {
    // Share only overlapping deserializations. A weak lease retains no
    // decoded bytes after the last load; each Module owns its copied mmap.
    let mut state = slot.lock().ok()?;
    let weak = state.as_mut().ok()?;
    if let Some(image) = weak.upgrade() {
        return Some(image);
    }
    // Only Cargo's immutable target-defined encoded array enters
    // this private decoder. Exact framing, output/window bounds and digest
    // precede publication. No caller bytes or mutable aliases are accepted.
    let Some(decoded) = super::native_storage::decode(encoded) else {
        *state = Err(());
        return None;
    };
    let image = Arc::new(decoded);
    *weak = Arc::downgrade(&image);
    return Some(image);
}
