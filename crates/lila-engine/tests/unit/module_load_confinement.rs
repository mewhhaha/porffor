use super::*;
use std::fs;
use std::sync::atomic::{AtomicU64, Ordering};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let base = std::env::temp_dir().join(format!(
            "lila-load-confinement-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(base.join("root")).unwrap();
        Self(base)
    }

    fn root(&self) -> PathBuf {
        self.0.join("root")
    }

    fn loader(&self) -> FilesystemModuleLoader {
        FilesystemModuleLoader::new(Some(self.root().to_str().unwrap()), None).unwrap()
    }

    fn key(path: &Path) -> ModuleKey {
        ModuleKey::from_host(path.to_str().unwrap())
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn direct_load_rejects_outside_keys_before_reading_source() {
    let fixture = Fixture::new();
    let outside = fixture.0.join("outside.js");
    // Invalid UTF-8 would produce Io if the implementation tried to read it.
    fs::write(&outside, [0xff]).unwrap();
    assert!(matches!(
        fixture.loader().load(&Fixture::key(&outside)),
        Err(ModuleLoadError::Denied { .. })
    ));
}

#[test]
fn host_loaded_entry_cannot_bypass_the_root() {
    let fixture = Fixture::new();
    let outside = fixture.0.join("outside.js");
    fs::write(&outside, "export const outside = true;").unwrap();
    let entry = ModuleEntry::HostLoad {
        locator: outside.to_str().unwrap().to_owned(),
    };
    assert!(matches!(
        load_module_graph(&entry, &fixture.loader()),
        Err(ModuleLoadError::Denied { .. })
    ));
}

#[test]
fn a_similarly_prefixed_sibling_directory_is_not_inside_the_root() {
    let fixture = Fixture::new();
    let sibling = fixture.0.join("root-sibling");
    fs::create_dir(&sibling).unwrap();
    let outside = sibling.join("entry.js");
    fs::write(&outside, "export const outside = true;").unwrap();
    assert!(matches!(
        fixture.loader().load(&Fixture::key(&outside)),
        Err(ModuleLoadError::Denied { .. })
    ));
}

#[test]
fn direct_load_inside_the_root_preserves_source_and_key() {
    let fixture = Fixture::new();
    let inside = fixture.root().join("entry.js");
    fs::write(&inside, "export const inside = true;").unwrap();
    let key = Fixture::key(&inside);
    let loaded = fixture.loader().load(&key).unwrap();
    assert_eq!(loaded.key, key);
    assert_eq!(
        loaded.kind,
        LoadedModuleKind::Source("export const inside = true;".to_owned())
    );
}

#[cfg(unix)]
#[test]
fn a_resolved_path_replaced_by_an_outside_symlink_is_denied_at_load() {
    let fixture = Fixture::new();
    let inside = fixture.root().join("entry.js");
    let outside = fixture.0.join("outside.js");
    fs::write(&inside, "export const inside = true;").unwrap();
    fs::write(&outside, "export const outside = true;").unwrap();
    let loader = fixture.loader();
    let key = loader
        .resolve(None, &ModuleRequestKeyIr::plain("entry.js"))
        .unwrap();
    fs::remove_file(&inside).unwrap();
    std::os::unix::fs::symlink(&outside, &inside).unwrap();
    assert!(matches!(
        loader.load(&key),
        Err(ModuleLoadError::Denied { .. })
    ));
}
