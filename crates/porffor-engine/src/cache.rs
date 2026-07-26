use std::borrow::Cow;
use std::cmp::Reverse;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::SystemTime;

use wasmtime::{Cache, CacheConfig, CacheStore};

pub(crate) const CACHE_LIMIT_BYTES: u64 = 1024 * 1024 * 1024;
pub(crate) const CACHE_PRUNE_PERCENT: u64 = 70;
pub(crate) const HALF_CACHE_LIMIT_BYTES: u64 = CACHE_LIMIT_BYTES / 2;

const FUNCTION_CACHE_FORMAT: &str = "cranelift-functions-v1";
const MODULE_CACHE_DIR: &str = "wasmtime-modules-v1";
const PROGRAM_CACHE_DIR: &str = "program-wasm-v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheDirectoryStatus {
    pub path: PathBuf,
    pub bytes: u64,
    pub files: u64,
    pub limit_bytes: Option<u64>,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheStatus {
    pub function_cache: CacheDirectoryStatus,
    pub module_cache: CacheDirectoryStatus,
    pub program_cache: CacheDirectoryStatus,
    pub legacy_wasmtime_cache: CacheDirectoryStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachePruneReport {
    pub porffor_bytes_removed: u64,
    pub porffor_files_removed: u64,
    pub legacy_bytes_removed: u64,
    pub legacy_files_removed: u64,
}

#[derive(Debug)]
pub(crate) struct FunctionCache {
    directory: PathBuf,
    limit_bytes: u64,
    current_bytes: AtomicU64,
    hits: AtomicU64,
    misses: AtomicU64,
    prune_lock: Mutex<()>,
    temp_counter: AtomicU64,
}

impl FunctionCache {
    pub(crate) fn new(directory: PathBuf, limit_bytes: u64) -> io::Result<Self> {
        fs::create_dir_all(&directory)?;
        let initial_bytes = directory_usage(&directory)?.0;
        let cache = Self {
            directory,
            limit_bytes,
            current_bytes: AtomicU64::new(initial_bytes),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
            prune_lock: Mutex::new(()),
            temp_counter: AtomicU64::new(0),
        };
        cache.prune_if_needed()?;
        Ok(cache)
    }

    fn entry_path(&self, key: &[u8]) -> PathBuf {
        let mut name = String::with_capacity(key.len() * 2);
        const HEX: &[u8; 16] = b"0123456789abcdef";
        for byte in key {
            name.push(HEX[(byte >> 4) as usize] as char);
            name.push(HEX[(byte & 0x0f) as usize] as char);
        }
        self.directory.join(name)
    }

    fn insert_atomic(&self, path: &Path, value: &[u8]) -> io::Result<()> {
        let suffix = self.temp_counter.fetch_add(1, Ordering::Relaxed);
        let mut temp_name: OsString = path.file_name().unwrap_or_default().to_os_string();
        temp_name.push(format!(".tmp-{}-{suffix}", std::process::id()));
        let temp = path.with_file_name(temp_name);
        fs::write(&temp, value)?;
        match fs::rename(&temp, path) {
            Ok(()) => Ok(()),
            Err(err) => {
                let _ = fs::remove_file(&temp);
                Err(err)
            }
        }
    }

    fn prune_if_needed(&self) -> io::Result<()> {
        let _guard = self
            .prune_lock
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        prune_directory_to_limit(&self.directory, self.limit_bytes)?;
        self.current_bytes
            .store(directory_usage(&self.directory)?.0, Ordering::Relaxed);
        Ok(())
    }

    pub(crate) fn counters(&self) -> (u64, u64) {
        (
            self.hits.load(Ordering::Relaxed),
            self.misses.load(Ordering::Relaxed),
        )
    }

    pub(crate) fn contains(&self, key: &[u8]) -> bool {
        self.entry_path(key).is_file()
    }

    pub(crate) fn read(&self, key: &[u8]) -> Option<Vec<u8>> {
        let bytes = <Self as CacheStore>::get(self, key).map(Cow::into_owned)?;
        // The program cache can live on a noatime mount, so a read alone
        // cannot make a frequently reused artifact survive LRU pruning.
        if let Ok(file) = fs::File::options().write(true).open(self.entry_path(key)) {
            let _ = file.set_modified(SystemTime::now());
        }
        Some(bytes)
    }

    pub(crate) fn write(&self, key: &[u8], value: Vec<u8>) -> bool {
        <Self as CacheStore>::insert(self, key, value)
    }

    pub(crate) fn remove(&self, key: &[u8]) {
        let path = self.entry_path(key);
        if let Ok(metadata) = path.metadata() {
            if fs::remove_file(path).is_ok() {
                self.current_bytes
                    .fetch_sub(metadata.len(), Ordering::Relaxed);
            }
        }
    }
}

impl CacheStore for FunctionCache {
    fn get(&self, key: &[u8]) -> Option<Cow<'_, [u8]>> {
        let path = self.entry_path(key);
        match fs::read(path) {
            Ok(bytes) => {
                self.hits.fetch_add(1, Ordering::Relaxed);
                Some(Cow::Owned(bytes))
            }
            Err(_) => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    fn insert(&self, key: &[u8], value: Vec<u8>) -> bool {
        let path = self.entry_path(key);
        let old_len = path.metadata().map(|metadata| metadata.len()).unwrap_or(0);
        if self.insert_atomic(&path, &value).is_err() {
            return false;
        }
        let new_len = value.len() as u64;
        let retained = if new_len >= old_len {
            self.current_bytes
                .fetch_add(new_len - old_len, Ordering::Relaxed)
                .saturating_add(new_len - old_len)
        } else {
            self.current_bytes
                .fetch_sub(old_len - new_len, Ordering::Relaxed)
                .saturating_sub(old_len - new_len)
        };
        // The common path is O(1). The directory walk only happens on startup
        // and when the tracked total crosses the hard limit.
        retained <= self.limit_bytes || self.prune_if_needed().is_ok()
    }
}

pub(crate) fn function_cache_directory() -> PathBuf {
    porffor_cache_root().join(FUNCTION_CACHE_FORMAT)
}

pub(crate) fn module_cache_directory() -> PathBuf {
    porffor_cache_root().join(MODULE_CACHE_DIR)
}

pub(crate) fn program_cache_directory() -> PathBuf {
    porffor_cache_root().join(PROGRAM_CACHE_DIR)
}

pub(crate) fn module_cache() -> io::Result<Cache> {
    module_cache_at(module_cache_directory(), HALF_CACHE_LIMIT_BYTES)
}

fn module_cache_at(directory: PathBuf, limit_bytes: u64) -> io::Result<Cache> {
    fs::create_dir_all(&directory)?;
    // Wasmtime prunes on a background worker. A short-lived Test262 case
    // process can exit before that worker handles its update, so enforce the
    // same bound synchronously whenever a process opens Porffor's cache.
    prune_directory_to_limit(&directory, limit_bytes)?;

    let mut config = CacheConfig::new();
    config
        .with_directory(directory)
        .with_files_total_size_soft_limit(limit_bytes)
        .with_files_total_size_limit_percent_if_deleting(CACHE_PRUNE_PERCENT as u8)
        .with_file_count_limit_percent_if_deleting(CACHE_PRUNE_PERCENT as u8)
        // Ask the long-lived process worker to check after every update too.
        .with_cleanup_interval(std::time::Duration::ZERO);
    Cache::new(config).map_err(io::Error::other)
}

pub fn cache_status() -> io::Result<CacheStatus> {
    Ok(CacheStatus {
        function_cache: directory_status(&function_cache_directory(), Some(CACHE_LIMIT_BYTES))?,
        module_cache: directory_status(&module_cache_directory(), Some(HALF_CACHE_LIMIT_BYTES))?,
        program_cache: directory_status(&program_cache_directory(), Some(HALF_CACHE_LIMIT_BYTES))?,
        legacy_wasmtime_cache: directory_status(&legacy_wasmtime_cache_directory(), None)?,
    })
}

pub fn prune_caches(include_legacy_wasmtime: bool) -> io::Result<CachePruneReport> {
    let function_path = function_cache_directory();
    let module_path = module_cache_directory();
    let program_path = program_cache_directory();
    let function = remove_directory_contents(&function_path)?;
    let module = remove_directory_contents(&module_path)?;
    let program = remove_directory_contents(&program_path)?;
    // A long-lived embedder may invoke the reusable command entrypoint again
    // after pruning. Keep initialized cache objects usable without requiring a
    // process restart.
    fs::create_dir_all(function_path)?;
    fs::create_dir_all(module_path)?;
    fs::create_dir_all(program_path)?;
    let legacy = if include_legacy_wasmtime {
        remove_directory_contents(&legacy_wasmtime_cache_directory())?
    } else {
        (0, 0)
    };
    Ok(CachePruneReport {
        porffor_bytes_removed: function
            .0
            .saturating_add(module.0)
            .saturating_add(program.0),
        porffor_files_removed: function
            .1
            .saturating_add(module.1)
            .saturating_add(program.1),
        legacy_bytes_removed: legacy.0,
        legacy_files_removed: legacy.1,
    })
}

fn porffor_cache_root() -> PathBuf {
    if let Some(path) = std::env::var_os("PORFFOR_CACHE_DIR") {
        return PathBuf::from(path);
    }
    platform_cache_root().join("porffor")
}

fn legacy_wasmtime_cache_directory() -> PathBuf {
    platform_cache_root().join("wasmtime")
}

fn platform_cache_root() -> PathBuf {
    if let Some(path) = std::env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(path);
    }
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".cache");
    }
    std::env::temp_dir()
}

fn directory_status(path: &Path, limit_bytes: Option<u64>) -> io::Result<CacheDirectoryStatus> {
    let exists = path.exists();
    let (bytes, files) = directory_usage(path)?;
    Ok(CacheDirectoryStatus {
        path: path.to_path_buf(),
        bytes,
        files,
        limit_bytes,
        exists,
    })
}

fn directory_usage(path: &Path) -> io::Result<(u64, u64)> {
    if !path.exists() {
        return Ok((0, 0));
    }
    let mut bytes = 0u64;
    let mut files = 0u64;
    let mut pending = vec![path.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let Some(metadata) = existing_entry_metadata(&entry)? else {
                continue;
            };
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                bytes = bytes.saturating_add(metadata.len());
                files = files.saturating_add(1);
            }
        }
    }
    Ok((bytes, files))
}

fn prune_directory_to_limit(path: &Path, limit_bytes: u64) -> io::Result<()> {
    let mut entries = cache_files(path)?;
    let total = entries.iter().map(|entry| entry.1).sum::<u64>();
    if total <= limit_bytes {
        return Ok(());
    }
    let target = limit_bytes.saturating_mul(CACHE_PRUNE_PERCENT) / 100;
    entries.sort_by_key(|entry| Reverse(entry.2));
    let mut retained = total;
    while retained > target {
        let Some((path, bytes, _)) = entries.pop() else {
            break;
        };
        match fs::remove_file(path) {
            Ok(()) => retained = retained.saturating_sub(bytes),
            Err(err) if err.kind() == io::ErrorKind::NotFound => {
                retained = retained.saturating_sub(bytes)
            }
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

fn cache_files(path: &Path) -> io::Result<Vec<(PathBuf, u64, SystemTime)>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let mut files = Vec::new();
    let mut pending = vec![path.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let Some(metadata) = existing_entry_metadata(&entry)? else {
                continue;
            };
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                let accessed = metadata.accessed().unwrap_or(SystemTime::UNIX_EPOCH);
                let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                let used = accessed.max(modified);
                files.push((entry.path(), metadata.len(), used));
            }
        }
    }
    Ok(files)
}

fn existing_entry_metadata(entry: &fs::DirEntry) -> io::Result<Option<fs::Metadata>> {
    match entry.metadata() {
        Ok(metadata) => Ok(Some(metadata)),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

fn remove_directory_contents(path: &Path) -> io::Result<(u64, u64)> {
    let usage = directory_usage(path)?;
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(usage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    fn temp_cache(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "porffor-cache-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ))
    }

    #[test]
    fn function_cache_round_trips_and_overwrites_corruption() {
        let root = temp_cache("round-trip");
        let _ = fs::remove_dir_all(&root);
        let cache = FunctionCache::new(root.clone(), 1024).expect("cache should initialize");
        assert!(cache.get(b"key").is_none());
        assert!(cache.insert(b"key", vec![1, 2, 3]));
        assert_eq!(cache.get(b"key").as_deref(), Some(&[1, 2, 3][..]));
        assert!(cache.insert(b"key", vec![9]));
        assert_eq!(cache.get(b"key").as_deref(), Some(&[9][..]));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn program_cache_reads_refresh_recency_when_access_times_do_not_change() {
        let root = temp_cache("read-recency");
        let _ = fs::remove_dir_all(&root);
        let cache = FunctionCache::new(root.clone(), 100).expect("cache should initialize");
        assert!(cache.insert(b"reused", vec![1; 30]));
        assert!(cache.insert(b"unused", vec![2; 40]));
        let old = fs::FileTimes::new()
            .set_accessed(SystemTime::UNIX_EPOCH)
            .set_modified(SystemTime::UNIX_EPOCH);
        fs::File::options()
            .write(true)
            .open(cache.entry_path(b"reused"))
            .unwrap()
            .set_times(old)
            .unwrap();
        fs::File::options()
            .write(true)
            .open(cache.entry_path(b"unused"))
            .unwrap()
            .set_times(
                fs::FileTimes::new()
                    .set_accessed(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1))
                    .set_modified(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1)),
            )
            .unwrap();

        assert_eq!(cache.read(b"reused").as_deref(), Some(&[1; 30][..]));
        assert!(cache.insert(b"new", vec![3; 31]));

        assert!(cache.contains(b"reused"));
        assert!(!cache.contains(b"unused"));
        assert!(cache.contains(b"new"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn function_cache_contains_checks_entry_metadata_without_counting_a_read() {
        let root = temp_cache("contains");
        let _ = fs::remove_dir_all(&root);
        let cache = FunctionCache::new(root.clone(), 1024).expect("cache should initialize");

        assert!(!cache.contains(b"key"));
        assert!(cache.insert(b"key", vec![1, 2, 3]));
        assert!(cache.contains(b"key"));
        assert_eq!(cache.counters(), (0, 0));

        cache.remove(b"key");
        assert!(!cache.contains(b"key"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn function_cache_is_safe_under_concurrent_atomic_writes() {
        let root = temp_cache("concurrent");
        let _ = fs::remove_dir_all(&root);
        let cache = Arc::new(FunctionCache::new(root.clone(), 4096).unwrap());
        let threads = (0..8)
            .map(|value| {
                let cache = cache.clone();
                std::thread::spawn(move || cache.insert(b"same-key", vec![value; 64]))
            })
            .collect::<Vec<_>>();
        for thread in threads {
            assert!(thread.join().unwrap());
        }
        let value = cache.get(b"same-key").expect("entry should exist");
        assert_eq!(value.len(), 64);
        assert!(value.iter().all(|byte| *byte == value[0]));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn function_cache_prunes_to_seventy_percent_after_limit() {
        let root = temp_cache("prune");
        let _ = fs::remove_dir_all(&root);
        let cache = FunctionCache::new(root.clone(), 100).unwrap();
        assert!(cache.insert(b"one", vec![1; 60]));
        assert!(cache.insert(b"two", vec![2; 60]));
        let (bytes, _) = directory_usage(&root).unwrap();
        assert!(bytes <= 70, "cache retained {bytes} bytes");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn module_cache_prunes_short_lived_process_growth_on_startup() {
        let root = temp_cache("module-startup-prune");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("older"), [1; 60]).unwrap();
        fs::write(root.join("newer"), [2; 60]).unwrap();

        let _cache = module_cache_at(root.clone(), 100).unwrap();
        let (bytes, _) = directory_usage(&root).unwrap();
        assert!(bytes <= 70, "cache retained {bytes} bytes");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn cache_scan_skips_an_entry_removed_before_metadata_lookup() {
        let root = temp_cache("removed-entry");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("vanished"), [1]).unwrap();
        let entry = fs::read_dir(&root)
            .unwrap()
            .next()
            .expect("cache entry should exist")
            .unwrap();
        fs::remove_file(entry.path()).unwrap();

        assert!(existing_entry_metadata(&entry).unwrap().is_none());
        let _ = fs::remove_dir_all(root);
    }
}
