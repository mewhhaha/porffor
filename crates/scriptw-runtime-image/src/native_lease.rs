//! Weak ownership shares concurrent loads without retaining decoded storage.
use super::NativeRuntimeBytes;

pub(super) fn acquire() -> Option<std::sync::Arc<NativeRuntimeBytes>> {
    use std::sync::{Arc, Mutex, Weak};
    // Share only overlapping deserializations. A weak lease retains no
    // decoded bytes after the last load; each Module owns its copied mmap.
    static IMAGE: Mutex<Result<Weak<NativeRuntimeBytes>, ()>> = Mutex::new(Ok(Weak::new()));
    let mut state = IMAGE.lock().ok()?;
    let weak = state.as_mut().ok()?;
    if let Some(image) = weak.upgrade() {
        return Some(image);
    }
    // SAFETY: only Cargo's immutable target-defined encoded array enters
    // this private decoder. Exact framing, output/window bounds and digest
    // precede publication. No caller bytes or mutable aliases are accepted.
    let Some(decoded) =
        super::native_storage::decode(unsafe { &super::SCRIPTW_RUNTIME_PORTABLE_NATIVE })
    else {
        *state = Err(());
        return None;
    };
    let image = Arc::new(decoded);
    *weak = Arc::downgrade(&image);
    return Some(image);
}
