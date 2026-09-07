//! Shared filesystem identity for module entry and dependency paths.

use std::path::{Component, Path, PathBuf};

/// Resolve an existing path physically before applying lexical normalization.
///
/// Collapsing `link/..` first can select an entirely different JavaScript
/// module: the parent belongs to the symlink's target, not its spelling.
/// Keep the existing lexical fallback for virtual/missing path components.
/// The caller still owns root confinement and file-existence checks.
pub(crate) fn normalize(path: &Path) -> PathBuf {
    if let Ok(resolved) = path.canonicalize() {
        return resolved;
    }

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
    fn existing_files_have_their_canonical_identity() {
        let fixture = Fixture::new();
        let file = fixture.0.join("entry.js");
        fs::write(&file, "export const value = 1;").unwrap();
        assert_eq!(normalize(&file), file.canonicalize().unwrap());
    }

    #[test]
    fn missing_components_keep_the_lexical_fallback() {
        let fixture = Fixture::new();
        let root = fixture.0.canonicalize().unwrap();
        let virtual_path = root.join("missing").join("..").join("entry.js");
        assert_eq!(normalize(&virtual_path), root.join("entry.js"));
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
    fn symlink_parent_selects_the_target_parent_not_the_lexical_parent() {
        let fixture = Fixture::new();
        fs::create_dir_all(fixture.0.join("real/nested")).unwrap();
        let intended = fixture.0.join("real/dep.js");
        let decoy = fixture.0.join("dep.js");
        fs::write(&intended, "export const selected = 'intended';").unwrap();
        fs::write(&decoy, "export const selected = 'decoy';").unwrap();
        std::os::unix::fs::symlink(
            fixture.0.join("real/nested"),
            fixture.0.join("link"),
        )
        .unwrap();
        let resolved = normalize(&fixture.0.join("link/../dep.js"));
        assert_eq!(resolved, intended.canonicalize().unwrap());
        assert_ne!(resolved, decoy.canonicalize().unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn physically_outside_paths_remain_visible_to_the_confinement_check() {
        let fixture = Fixture::new();
        fs::create_dir(fixture.0.join("root")).unwrap();
        fs::create_dir_all(fixture.0.join("outside/nested")).unwrap();
        fs::write(fixture.0.join("outside/dep.js"), "outside").unwrap();
        fs::write(fixture.0.join("root/dep.js"), "inside decoy").unwrap();
        std::os::unix::fs::symlink(
            fixture.0.join("outside/nested"),
            fixture.0.join("root/link"),
        )
        .unwrap();
        let resolved = normalize(&fixture.0.join("root/link/../dep.js"));
        assert!(!resolved.starts_with(fixture.0.join("root").canonicalize().unwrap()));
        assert_eq!(
            resolved,
            fixture.0.join("outside/dep.js").canonicalize().unwrap()
        );
    }

    #[cfg(unix)]
    #[test]
    fn aliased_entry_and_dependency_paths_share_one_identity() {
        let fixture = Fixture::new();
        fs::create_dir_all(fixture.0.join("real/nested")).unwrap();
        fs::write(fixture.0.join("real/entry.js"), "export const value = 1;").unwrap();
        std::os::unix::fs::symlink(
            fixture.0.join("real/nested"),
            fixture.0.join("link"),
        )
        .unwrap();
        assert_eq!(
            normalize(&fixture.0.join("link/../entry.js")),
            normalize(&fixture.0.join("real/entry.js"))
        );
    }
}
