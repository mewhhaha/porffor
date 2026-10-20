//! Build-only storage selection; Wasmtime retains exact native cache admission.
use std::{io, path::PathBuf};
use wasmtime::Cache;

pub(super) fn module_cache() -> io::Result<Cache> {
    if let Some(directory) = std::env::var_os("SCRIPTW_BUILD_MODULE_CACHE_DIR") {
        crate::cache::module_cache_at(
            PathBuf::from(directory),
            crate::cache::module_cache_limit_bytes(),
        )
    } else {
        crate::cache::module_cache()
    }
}
