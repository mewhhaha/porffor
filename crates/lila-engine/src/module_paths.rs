//! Shared lexical identity for module entry and dependency paths.

use std::path::{Component, Path, PathBuf};

/// Fold dot components before the caller resolves filesystem symlinks.
///
/// This preserves the loader's existing specifier policy: `link/../dep.js`
/// names the lexical parent's dependency, not the symlink target's parent.
/// Normalization performs no IO and works for missing/virtual components.
/// The caller still canonicalizes the result and checks root confinement.
pub(crate) fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    let mut floor = 0usize;
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if out.components().count() > floor {
                    out.pop();
                }
            }
            other => {
                out.push(other.as_os_str());
                if matches!(other, Component::RootDir | Component::Prefix(_)) {
                    floor = out.components().count();
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let root = std::env::temp_dir().join(format!(
                "lila-module-path-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&root).unwrap();
            Self(root)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn existing_files_do_not_change_lexical_normalization() {
        let fixture = Fixture::new();
        let file = fixture.0.join("entry.js");
        let before = normalize(&file);
        fs::write(&file, "export const value = 1;").unwrap();
        assert_eq!(normalize(&file), before);
        assert_eq!(before, file);
    }

    #[test]
    fn missing_components_are_normalized_without_filesystem_lookups() {
        let fixture = Fixture::new();
        let virtual_path = fixture.0.join("missing").join("..").join("entry.js");
        assert_eq!(normalize(&virtual_path), fixture.0.join("entry.js"));
    }

    #[test]
    fn parent_components_never_turn_an_absolute_path_relative() {
        let fixture = Fixture::new();
        let root = fixture.0.canonicalize().unwrap();
        let mut path = root.clone();
        for _ in 0..root.components().count() + 2 {
            path.push("..");
        }
        path.push("lila-nonexistent-module-path-fixture.js");
        let normalized = normalize(&path);
        assert!(normalized.is_absolute());
        assert_eq!(
            normalized.file_name().unwrap(),
            "lila-nonexistent-module-path-fixture.js"
        );
    }

    #[cfg(unix)]
    #[test]
    fn lexical_parent_precedes_symlink_resolution() {
        let fixture = Fixture::new();
        fs::create_dir_all(fixture.0.join("real/nested")).unwrap();
        let lexical = fixture.0.join("dep.js");
        let physical = fixture.0.join("real/dep.js");
        fs::write(&lexical, "export const selected = 'lexical';").unwrap();
        fs::write(&physical, "export const selected = 'physical';").unwrap();
        std::os::unix::fs::symlink(
            fixture.0.join("real/nested"),
            fixture.0.join("link"),
        )
        .unwrap();
        let request = fixture.0.join("link/../dep.js");
        let selected = normalize(&request).canonicalize().unwrap();
        assert_eq!(selected, lexical.canonicalize().unwrap());
        assert_ne!(selected, request.canonicalize().unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn surviving_symlinks_are_still_resolved_before_confinement() {
        let fixture = Fixture::new();
        fs::create_dir(fixture.0.join("root")).unwrap();
        fs::create_dir(fixture.0.join("outside")).unwrap();
        fs::write(fixture.0.join("outside/dep.js"), "outside").unwrap();
        std::os::unix::fs::symlink(
            fixture.0.join("outside"),
            fixture.0.join("root/link"),
        )
        .unwrap();
        let selected = normalize(&fixture.0.join("root/link/dep.js"))
            .canonicalize()
            .unwrap();
        assert!(!selected.starts_with(fixture.0.join("root").canonicalize().unwrap()));
        assert_eq!(
            selected,
            fixture.0.join("outside/dep.js").canonicalize().unwrap()
        );
    }

    #[test]
    fn entry_and_dependency_spellings_share_one_lexical_identity() {
        let fixture = Fixture::new();
        assert_eq!(
            normalize(&fixture.0.join("sub/../entry.js")),
            normalize(&fixture.0.join("./entry.js"))
        );
    }
}
