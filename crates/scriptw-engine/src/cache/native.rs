//! Native stencils are reusable across Scriptw edits, not native compiler edits.

use super::FunctionCache;
use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::io;
use std::sync::Arc;
use wasmtime::CacheStore;

#[derive(Debug)]
pub(crate) struct NativeFunctionCache {
    store: Arc<FunctionCache>,
    compiler_identity: [u8; 64],
}

impl NativeFunctionCache {
    pub(crate) fn new(store: Arc<FunctionCache>, identity: &str) -> io::Result<Self> {
        if identity.len() != 64
            || !identity
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "native compiler identity must be canonical SHA-256",
            ));
        }
        Ok(Self {
            store,
            compiler_identity: identity.as_bytes().try_into().unwrap(),
        })
    }

    fn key(&self, key: &[u8]) -> [u8; 32] {
        let mut digest = Sha256::new();
        digest.update(b"scriptw-native-function-v1");
        digest.update(self.compiler_identity);
        digest.update((key.len() as u64).to_le_bytes());
        digest.update(key);
        digest.finalize().into()
    }

    pub(crate) fn counters(&self) -> (u64, u64) {
        self.store.counters()
    }
}

impl CacheStore for NativeFunctionCache {
    fn get(&self, key: &[u8]) -> Option<Cow<'_, [u8]>> {
        self.store.get(&self.key(key))
    }

    fn insert(&self, key: &[u8], value: Vec<u8>) -> bool {
        self.store.insert(&self.key(key), value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_stencils_share_one_budget_but_never_alias_other_compiler_generations() {
        let directory =
            std::env::temp_dir().join(format!("scriptw-native-cache-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        let store = Arc::new(FunctionCache::new(directory.clone(), 1024 * 1024).unwrap());
        let first = NativeFunctionCache::new(store.clone(), &"0".repeat(64)).unwrap();
        let second = NativeFunctionCache::new(store.clone(), &"1".repeat(64)).unwrap();
        assert!(NativeFunctionCache::new(store.clone(), "missing").is_err());
        assert!(first.insert(b"same-opaque-stencil", vec![1, 2]));
        assert_eq!(first.get(b"same-opaque-stencil").unwrap().as_ref(), &[1, 2]);
        assert!(second.get(b"same-opaque-stencil").is_none());
        assert!(second.insert(b"same-opaque-stencil", vec![3, 4]));
        assert_eq!(first.get(b"same-opaque-stencil").unwrap().as_ref(), &[1, 2]);
        assert_eq!(
            second.get(b"same-opaque-stencil").unwrap().as_ref(),
            &[3, 4]
        );
        assert!(store.get(b"same-opaque-stencil").is_none());
        std::fs::remove_dir_all(directory).unwrap();
    }
}
