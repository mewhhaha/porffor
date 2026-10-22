//! Hash the same immutable Cargo images within the caller's CPU budget.

use sha2::{Digest, Sha256};

pub(super) fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

pub(super) fn images(
    package_bytes: &[u8],
    native_bytes: &[u8],
    requested_jobs: usize,
) -> Option<([u8; 32], [u8; 32])> {
    let jobs = requested_jobs
        .clamp(1, 2)
        .min(std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get));
    if jobs == 1 {
        return Some((digest(package_bytes), digest(native_bytes)));
    }
    std::thread::scope(|scope| {
        let native = std::thread::Builder::new()
            .name("scriptw-native-image-digest".into())
            .spawn_scoped(scope, || digest(native_bytes));
        match native {
            Ok(native) => {
                let package = digest(package_bytes);
                native.join().ok().map(|native| (package, native))
            }
            // Thread creation is optional; complete both checks serially.
            Err(_) => Some((digest(package_bytes), digest(native_bytes))),
        }
    })
}
