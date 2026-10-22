use super::*;
#[test]
fn concurrent_loads_share_bytes_and_the_last_lease_releases_them() {
    if !cfg!(all(
        target_endian = "little",
        any(
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "riscv64"
        )
    )) {
        assert!(native().is_none());
        assert!(portable_native().is_none());
        return;
    }
    use std::sync::{Arc, Barrier};
    type Acquire = fn() -> Option<Arc<NativeRuntimeBytes>>;
    let readers: [Acquire; 2] = [native, portable_native];
    for (profile, read) in readers.into_iter().enumerate() {
        let barrier = Arc::new(Barrier::new(4));
        let threads = (0..4)
            .map(|_| {
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    let bytes = read().expect("supported Cargo image");
                    barrier.wait();
                    bytes
                })
            })
            .collect::<Vec<_>>();
        let leases = threads
            .into_iter()
            .map(|t| t.join().unwrap())
            .collect::<Vec<_>>();
        assert!(leases.iter().all(|image| Arc::ptr_eq(image, &leases[0])));
        let other = readers[1 - profile]().expect("other supported Cargo profile");
        assert!(
            !Arc::ptr_eq(&other, &leases[0]),
            "profiles must not share their decoded-byte lease slot"
        );
        drop(other);
        let digest = *leases[0].digest();
        let weak = Arc::downgrade(&leases[0]);
        drop(leases);
        assert!(
            weak.upgrade().is_none(),
            "the cache must not retain a strong decoded-byte owner"
        );
        let later = read().expect("a new actual load may decode again");
        assert_eq!(later.digest(), &digest);
    }
}
