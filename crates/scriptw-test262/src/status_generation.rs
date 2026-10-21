//! A JSON/text pair becomes visible through one atomic manifest replacement.
//! Generation members are immutable; interrupted writers can leave unselected
//! members, but cannot select a mixed pair.
use crate::{snapshot_publication::write_snapshot_file, PublicationBackend};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct PublishedStatusPaths {
    pub manifest_path: PathBuf,
    pub json_path: PathBuf,
    pub txt_path: PathBuf,
}

#[derive(Serialize)]
struct Member {
    file: String,
    sha256: String,
}

#[derive(Serialize)]
struct Manifest {
    schema_version: u8,
    backend: &'static str,
    generation: String,
    json: Member,
    text: Member,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn generation_id(json: &[u8], text: &[u8]) -> String {
    let mut hash = Sha256::new();
    hash.update(b"scriptw-status-generation-v1\0");
    for bytes in [json, text] {
        hash.update((bytes.len() as u64).to_le_bytes());
        hash.update(bytes);
    }
    format!("{:x}", hash.finalize())
}

/// Publish already-rendered evidence. The manifest is the only mutable commit
/// point; readers must resolve it once and check both member digests.
pub fn publish_status_generation(
    directory: &Path,
    backend: PublicationBackend,
    json: &[u8],
    text: &[u8],
) -> io::Result<PublishedStatusPaths> {
    publish_with(directory, backend, json, text, write_snapshot_file)
}

fn publish_with(
    directory: &Path,
    backend: PublicationBackend,
    json: &[u8],
    text: &[u8],
    mut write: impl FnMut(&Path, &[u8]) -> io::Result<()>,
) -> io::Result<PublishedStatusPaths> {
    fs::create_dir_all(directory)?;
    let generation = generation_id(json, text);
    let stem = format!("published-status-{}", backend.as_str());
    let manifest = Manifest {
        schema_version: 1,
        backend: backend.as_str(),
        json: Member {
            file: format!("{stem}-{generation}.json"),
            sha256: digest(json),
        },
        text: Member {
            file: format!("{stem}-{generation}.txt"),
            sha256: digest(text),
        },
        generation,
    };
    let paths = PublishedStatusPaths {
        manifest_path: directory.join(format!("{stem}.manifest.json")),
        json_path: directory.join(&manifest.json.file),
        txt_path: directory.join(&manifest.text.file),
    };
    for (path, bytes) in [(&paths.json_path, json), (&paths.txt_path, text)] {
        match fs::symlink_metadata(path) {
            Ok(metadata) if !metadata.is_file() => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "immutable status member must be a regular file: {}",
                        path.display()
                    ),
                ))
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error),
        }
        match fs::read(path) {
            Ok(existing) if existing == bytes => {}
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("immutable status member differs: {}", path.display()),
                ))
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => write(path, bytes)?,
            Err(error) => return Err(error),
        }
    }
    let encoded = serde_json::to_vec_pretty(&manifest).map_err(io::Error::other)?;
    write(&paths.manifest_path, &encoded)?;
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Directory(PathBuf);
    impl Directory {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "scriptw-status-pair-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    #[cfg(unix)]
    fn matching_symlink_member_cannot_commit_a_generation() {
        let directory = Directory::new();
        let generation = generation_id(b"json", b"text");
        let target = directory.0.join("other.json");
        fs::write(&target, b"json").unwrap();
        std::os::unix::fs::symlink(
            &target,
            directory
                .0
                .join(format!("published-status-wasm-aot-{generation}.json")),
        )
        .unwrap();
        let error =
            publish_status_generation(&directory.0, PublicationBackend::WasmAot, b"json", b"text")
                .unwrap_err();
        assert!(error.to_string().contains("regular file"));
        assert!(!directory
            .0
            .join("published-status-wasm-aot.manifest.json")
            .exists());
    }

    #[test]
    fn failed_member_or_manifest_write_keeps_the_previous_generation_selected() {
        for fail_at in [1, 2, 3] {
            let directory = Directory::new();
            let old = publish_status_generation(
                &directory.0,
                PublicationBackend::WasmAot,
                b"old json",
                b"old text",
            )
            .unwrap();
            let before = fs::read(&old.manifest_path).unwrap();
            let mut calls = 0;
            let error = publish_with(
                &directory.0,
                PublicationBackend::WasmAot,
                b"new json",
                b"new text",
                |path, bytes| {
                    calls += 1;
                    if calls == fail_at {
                        return Err(io::Error::other("injected publication failure"));
                    }
                    write_snapshot_file(path, bytes)
                },
            )
            .unwrap_err();
            assert!(error.to_string().contains("injected"));
            assert_eq!(fs::read(&old.manifest_path).unwrap(), before);
            assert_eq!(fs::read(&old.json_path).unwrap(), b"old json");
            assert_eq!(fs::read(&old.txt_path).unwrap(), b"old text");
            let new = publish_status_generation(
                &directory.0,
                PublicationBackend::WasmAot,
                b"new json",
                b"new text",
            )
            .unwrap();
            let manifest: serde_json::Value =
                serde_json::from_slice(&fs::read(&new.manifest_path).unwrap()).unwrap();
            assert_eq!(
                manifest["json"]["sha256"],
                digest(&fs::read(&new.json_path).unwrap())
            );
            assert_eq!(
                manifest["text"]["sha256"],
                digest(&fs::read(&new.txt_path).unwrap())
            );
            assert_eq!(
                manifest["generation"],
                generation_id(b"new json", b"new text")
            );
            assert_ne!(new.json_path, old.json_path);
            assert_ne!(new.txt_path, old.txt_path);
        }
    }

    #[test]
    fn a_damaged_immutable_member_cannot_be_selected_by_retry() {
        let directory = Directory::new();
        let paths =
            publish_status_generation(&directory.0, PublicationBackend::WasmAot, b"json", b"text")
                .unwrap();
        fs::write(&paths.txt_path, b"damaged").unwrap();
        assert_eq!(
            publish_status_generation(&directory.0, PublicationBackend::WasmAot, b"json", b"text")
                .unwrap_err()
                .kind(),
            io::ErrorKind::InvalidData
        );
    }
}
