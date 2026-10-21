//! Hash the same immutable Cargo images within the caller's CPU budget.

use sha2::{Digest, Sha256};

fn digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

pub(super) fn images(images: super::Images, requested_jobs: usize) -> Option<([u8; 32], [u8; 32])> {
    let jobs = requested_jobs
        .clamp(1, 2)
        .min(std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get));
    if jobs == 1 {
        return Some((digest(images.package), digest(images.native)));
    }
    std::thread::scope(|scope| {
        let native = std::thread::Builder::new()
            .name("scriptw-native-image-digest".into())
            .spawn_scoped(scope, || digest(images.native));
        match native {
            Ok(native) => {
                let package = digest(images.package);
                native.join().ok().map(|native| (package, native))
            }
            // Thread creation is optional; complete both checks serially.
            Err(_) => Some((digest(images.package), digest(images.native))),
        }
    })
}
