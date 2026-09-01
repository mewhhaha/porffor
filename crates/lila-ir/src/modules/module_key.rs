/// A stable module identity minted by the host's resolution boundary.
///
/// This is deliberately distinct from [`crate::ModuleRequestIr::specifier`]: the
/// latter is source text interpreted relative to a referrer, while this is the
/// normalized key the host resolved that request to. There is no `From<String>`
/// or public field, so request spelling, source text and `import.meta.url`
/// cannot implicitly become graph identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleKey(String);

impl ModuleKey {
    /// Records the stable identity selected by a host resolver.
    ///
    /// This is the only constructor. The host owns normalization because only
    /// it knows whether paths, URLs or an embedder-defined namespace identify
    /// the same module. `lila-ir` retains the resulting value but never derives
    /// one from a request specifier.
    #[must_use]
    pub fn from_host(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    /// The normalized key spelling selected by the host.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Key a one-node graph uses when the caller supplied no filename.
///
/// Not a path, and never resolvable: a caller with no filename also has no
/// directory to resolve relative specifiers against.
pub const ANONYMOUS_MODULE_KEY: &str = "<entry>";
