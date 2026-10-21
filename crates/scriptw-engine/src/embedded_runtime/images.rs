//! Select a single profile's immutable Cargo authority before touching payloads.
use scriptw_aot_wasm::RuntimeSourceCompilationProfile;

#[derive(Clone, Copy)]
pub(super) struct Images {
    pub(super) profile: RuntimeSourceCompilationProfile,
    pub(super) package: &'static [u8],
    pub(super) native: &'static [u8],
    pub(super) manifest: &'static [u8],
}

impl Images {
    pub(super) fn for_profile(profile: RuntimeSourceCompilationProfile) -> Self {
        use scriptw_runtime_image as image;
        let (package, native, manifest) = match profile {
            RuntimeSourceCompilationProfile::NativeHost => {
                (image::package(), image::native(), image::manifest_bytes())
            }
            RuntimeSourceCompilationProfile::Portable => (
                image::portable_package(),
                image::portable_native(),
                image::portable_manifest_bytes(),
            ),
        };
        Self {
            profile,
            package,
            native,
            manifest,
        }
    }
}
