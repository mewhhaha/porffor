//! Immutable Cargo-produced runtime images, isolated from compiler metadata.
//!
//! Every image comes from the current checked raw runtime producer. Engine
//! independently admits source, target, modes, layout and byte digests before
//! deserializing native code. This crate accepts no caller bytes or paths.

#[cfg(any(scriptw_runtime_image, test))]
mod native_storage;

#[cfg(scriptw_runtime_image)]
include!(concat!(env!("OUT_DIR"), "/image_bindings.rs"));

// The target Rust compiler defines each immutable data array with the same
// symbol and exact type as its generated declaration. The linked archive owns
// its storage for the process lifetime. No initializer enters crate metadata.
#[inline(never)]
pub fn package() -> &'static [u8] {
    #[cfg(scriptw_runtime_image)]
    // SAFETY: Cargo's checked producer and target compiler define this exact
    // immutable array in the linked data archive, with no mutable alias.
    return unsafe { &SCRIPTW_RUNTIME_PACKAGE };
    #[cfg(not(scriptw_runtime_image))]
    &[]
}

#[inline(never)]
pub fn native() -> &'static [u8] {
    #[cfg(scriptw_runtime_image)]
    // SAFETY: same target-defined immutable array contract as package().
    return unsafe { &SCRIPTW_RUNTIME_NATIVE };
    #[cfg(not(scriptw_runtime_image))]
    &[]
}

#[inline(never)]
pub fn manifest_bytes() -> &'static [u8] {
    #[cfg(scriptw_runtime_image)]
    // SAFETY: same target-defined immutable array contract as package().
    return unsafe { &SCRIPTW_RUNTIME_MANIFEST };
    #[cfg(not(scriptw_runtime_image))]
    &[]
}

#[inline(never)]
pub fn portable_package() -> &'static [u8] {
    #[cfg(scriptw_runtime_image)]
    // SAFETY: Cargo's checked producer and target compiler define this exact
    // immutable array in the linked data archive, with no mutable alias.
    return unsafe { &SCRIPTW_RUNTIME_PORTABLE_PACKAGE };
    #[cfg(not(scriptw_runtime_image))]
    &[]
}

#[inline(never)]
pub fn portable_native() -> &'static [u8] {
    #[cfg(scriptw_runtime_image)]
    {
        static IMAGE: std::sync::OnceLock<Option<Box<[u8]>>> = std::sync::OnceLock::new();
        // SAFETY: only Cargo's immutable target-defined encoded array enters
        // this decoder. The checked bytes are published without mutable aliases
        // for the process lifetime; failure leaves ordinary native compilation.
        return IMAGE
            .get_or_init(|| native_storage::decode(unsafe { &SCRIPTW_RUNTIME_PORTABLE_NATIVE }))
            .as_deref()
            .unwrap_or(&[]);
    }
    #[cfg(not(scriptw_runtime_image))]
    &[]
}

#[inline(never)]
pub fn portable_manifest_bytes() -> &'static [u8] {
    #[cfg(scriptw_runtime_image)]
    // SAFETY: same target-defined immutable array contract as package().
    return unsafe { &SCRIPTW_RUNTIME_PORTABLE_MANIFEST };
    #[cfg(not(scriptw_runtime_image))]
    &[]
}
