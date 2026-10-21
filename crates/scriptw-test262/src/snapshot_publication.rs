//! Keep the previous checkpoint readable until its replacement is complete.
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

struct PendingSnapshotFile {
    path: PathBuf,
    file: Option<File>,
}

impl PendingSnapshotFile {
    fn create(destination: &Path) -> io::Result<Self> {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let parent = destination.parent().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "snapshot output has no parent directory",
            )
        })?;
        let (path, file) = loop {
            // Keep the temporary name independent of the destination length:
            // a valid near-NAME_MAX snapshot must still be replaceable.
            let path = parent.join(format!(
                ".scriptw-snapshot-tmp-{}-{}",
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ));
            // A terminated writer can leave a temporary and its PID may be
            // reused. Skip that name without taking ownership of its file.
            match OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => break (path, file),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        };
        let pending = Self {
            path,
            file: Some(file),
        };
        match fs::metadata(destination) {
            Ok(metadata) => pending
                .file
                .as_ref()
                .expect("unpublished snapshot owns its file")
                .set_permissions(metadata.permissions())?,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        Ok(pending)
    }

    fn publish(mut self, destination: &Path) -> io::Result<()> {
        let file = self
            .file
            .take()
            .expect("unpublished snapshot owns its file");
        file.sync_all()?;
        drop(file);
        fs::rename(&self.path, destination)
    }
}

impl Drop for PendingSnapshotFile {
    fn drop(&mut self) {
        // Close first so an error also releases the temporary on Windows.
        drop(self.file.take());
        let _ = fs::remove_file(&self.path);
    }
}

/// Atomically replace a file after writing and syncing a sibling temporary.
/// The previous file remains intact if writing or renaming fails.
pub fn write_snapshot_file(path: &Path, bytes: &[u8]) -> io::Result<()> {
    write_snapshot_file_with(path, |file| file.write_all(bytes))
}

pub(super) fn write_snapshot_file_with(
    path: &Path,
    write: impl FnOnce(&mut File) -> io::Result<()>,
) -> io::Result<()> {
    let mut pending = PendingSnapshotFile::create(path)?;
    write(
        pending
            .file
            .as_mut()
            .expect("unpublished snapshot owns its file"),
    )?;
    pending.publish(path)
}
