//! Owned descendants of the worker staging directory, removed on every exit.
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub(super) struct Sandbox {
    root: PathBuf,
    live: bool,
}
impl Sandbox {
    pub(super) fn new_in(parent: &Path) -> Result<Self, String> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        for _ in 0..64 {
            let root = parent.join(format!(
                "lila-robustness-probe-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            match std::fs::create_dir(&root) {
                Ok(()) => {
                    let sandbox = Self { root, live: true };
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        std::fs::set_permissions(
                            sandbox.root(),
                            std::fs::Permissions::from_mode(0o700),
                        )
                        .map_err(|error| format!("probe directory permissions: {error}"))?;
                    }
                    return Ok(sandbox);
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(format!("probe directory creation: {error}")),
            }
        }
        Err("probe directory collision limit exceeded".into())
    }
    pub(super) fn root(&self) -> &Path {
        &self.root
    }
    pub(super) fn finish(mut self) -> Result<(), String> {
        std::fs::remove_dir_all(&self.root)
            .map_err(|error| format!("probe directory cleanup: {error}"))?;
        self.live = false;
        Ok(())
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        if self.live {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}

/// Fixture files cannot install links or choose a path outside their own tree.
pub(super) fn relative_fixture_path(name: &str) -> Result<(), String> {
    if name.is_empty() || name.len() > 256 || name.contains('\\') || name.contains('\0') {
        return Err("fixture path requires 1..=256 normal relative bytes".into());
    }
    if name
        .split('/')
        .any(|part| part.is_empty() || part == "." || part == ".." || part.contains(':'))
    {
        return Err("fixture path cannot contain empty, dot, parent or device components".into());
    }
    Ok(())
}
