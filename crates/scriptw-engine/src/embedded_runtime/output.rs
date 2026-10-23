//! Replace a Cargo output without modifying an older reader's shared inode.
use std::{
    ffi::OsString,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT_STAGING_FILE: AtomicU64 = AtomicU64::new(0);

struct StagedOutput(PathBuf);
impl Drop for StagedOutput {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub(super) fn write(path: &Path, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "output needs a filename"))?;
    let (staged, mut file) = loop {
        let sequence = NEXT_STAGING_FILE.fetch_add(1, Ordering::Relaxed);
        let mut temporary = OsString::from(".");
        temporary.push(name);
        temporary.push(format!(".{}.{sequence}.pending", std::process::id()));
        let temporary = path.with_file_name(temporary);
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
        {
            Ok(file) => break (StagedOutput(temporary), file),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    };
    file.write_all(bytes.as_ref())?;
    drop(file);
    // Renaming the directory entry replaces a symlink or hardlink itself.
    // Writing through that entry would corrupt another completed producer.
    fs::rename(&staged.0, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            let id = NEXT_STAGING_FILE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "scriptw-output-publication-{}-{id}",
                std::process::id()
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn only(&self, expected: &[&str]) {
            let mut names: Vec<_> = fs::read_dir(&self.0)
                .unwrap()
                .map(|entry| entry.unwrap().file_name())
                .collect();
            names.sort();
            let mut expected: Vec<_> = expected.iter().map(OsString::from).collect();
            expected.sort();
            assert_eq!(names, expected);
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn replaces_complete_outputs_including_empty_unsupported_images() {
        let directory = Directory::new();
        let path = directory.0.join("image.bin");
        write(&path, b"first image").unwrap();
        write(&path, b"second").unwrap();
        assert_eq!(fs::read(&path).unwrap(), b"second");
        write(&path, b"").unwrap();
        assert!(fs::read(&path).unwrap().is_empty());
        directory.only(&["image.bin"]);
    }

    #[cfg(unix)]
    #[test]
    fn replacing_a_symlink_preserves_the_completed_shared_image() {
        let directory = Directory::new();
        let old = directory.0.join("completed.bin");
        let output = directory.0.join("next.bin");
        fs::write(&old, b"original source").unwrap();
        std::os::unix::fs::symlink(&old, &output).unwrap();
        write(&output, b"next source").unwrap();
        assert_eq!(fs::read(&old).unwrap(), b"original source");
        assert_eq!(fs::read(&output).unwrap(), b"next source");
        assert!(!fs::symlink_metadata(&output)
            .unwrap()
            .file_type()
            .is_symlink());
        directory.only(&["completed.bin", "next.bin"]);
    }

    #[test]
    fn replacing_a_hardlink_preserves_the_completed_shared_image() {
        let directory = Directory::new();
        let old = directory.0.join("completed.bin");
        let output = directory.0.join("next.bin");
        fs::write(&old, b"original source").unwrap();
        fs::hard_link(&old, &output).unwrap();
        write(&output, b"next source").unwrap();
        assert_eq!(fs::read(&old).unwrap(), b"original source");
        assert_eq!(fs::read(&output).unwrap(), b"next source");
        directory.only(&["completed.bin", "next.bin"]);
    }

    #[test]
    fn failed_publication_keeps_the_destination_and_removes_staging() {
        let directory = Directory::new();
        let output = directory.0.join("occupied");
        fs::create_dir(&output).unwrap();
        fs::write(output.join("retained.bin"), b"untouched").unwrap();
        assert!(write(&output, b"new bytes").is_err());
        assert_eq!(fs::read(output.join("retained.bin")).unwrap(), b"untouched");
        directory.only(&["occupied"]);
    }
}
