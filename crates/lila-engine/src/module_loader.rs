//! The host side of module loading: `HostResolveImportedModule` and
//! `HostLoadImportedModule`.
//!
//! All loading happens at *compile* time, so the trait is synchronous. The
//! compiler reads the whole transitive closure of an entry module up front and
//! hands it to `lila-ir`, which performs no IO of its own.
//!
//! Nothing here bakes in a test262 path. The runner supplies the root, exactly
//! as any other embedder would.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use lila_front::{ParsedModule, ParsedScript};
use lila_ir::{ModuleGraphSources, ModuleSourceIr};
pub use lila_ir::{ModuleKey, ModuleRequestKeyIr};

#[path = "module_paths.rs"]
mod module_paths;
use module_paths::normalize;

/// What a successful load produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadedModuleKind {
    /// A Source Text Module Record's source text.
    Source(String),
    // No `Json` variant on purpose. A JSON module is `ParseJSONModule`'d, not
    // parsed as ECMAScript, and resolves exactly one name (`default`). Until
    // that record type exists, serving JSON text as a Source Text Module
    // Record would either be a SyntaxError (`{"a": 1}` is not a module body)
    // or, worse, parse as valid JS with zero exports (`[1, 2]`, `"s"`) and
    // report a bogus `MissingExport`. `load` rejects `.json` instead, which
    // leaves the request unresolved and produces one honest link error.
}

/// One loaded module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedModule {
    /// Key the module was loaded under.
    pub key: ModuleKey,
    /// The module's contents.
    pub kind: LoadedModuleKind,
    /// Value `import.meta.url` reports.
    pub meta_url: String,
}

/// Why a resolve or load failed.
///
/// Every variant becomes a `SyntaxError` at compile time, which is what
/// test262's `phase: resolution` negatives expect.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleLoadError {
    /// No module answers this specifier.
    NotFound {
        /// The specifier that could not be resolved.
        specifier: String,
        /// Key of the module that requested it, if any.
        referrer: Option<ModuleKey>,
    },
    /// Resolution escaped the configured root.
    Denied {
        /// The rejected specifier.
        specifier: String,
        /// Why it was rejected.
        reason: String,
    },
    /// An import attribute the host does not implement.
    UnsupportedAttribute {
        /// Attribute key.
        key: String,
        /// Attribute value.
        value: String,
    },
    /// Reading the module failed.
    Io {
        /// Key being read.
        key: ModuleKey,
        /// Underlying error text.
        message: String,
    },
}

impl core::fmt::Display for ModuleLoadError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NotFound { specifier, .. } => {
                write!(f, "unresolved module request: {specifier}")
            }
            Self::Denied { specifier, reason } => {
                write!(f, "module request denied: {specifier} ({reason})")
            }
            Self::UnsupportedAttribute { key, value } => {
                write!(f, "unsupported import attribute: {key}={value}")
            }
            Self::Io { key, message } => {
                write!(f, "failed to read module {}: {message}", key.as_str())
            }
        }
    }
}

/// The host hooks module loading needs.
pub trait HostModuleLoader: Send + Sync {
    /// `HostResolveImportedModule`: phase-free request key plus referrer to a
    /// stable module key.
    ///
    /// # Errors
    /// Returns an error when the specifier names nothing the host will serve.
    fn resolve(
        &self,
        referrer: Option<&ModuleKey>,
        request: &ModuleRequestKeyIr,
    ) -> Result<ModuleKey, ModuleLoadError>;

    /// `HostLoadImportedModule`: key to module contents.
    ///
    /// # Errors
    /// Returns an error when the module cannot be read.
    fn load(&self, key: &ModuleKey) -> Result<LoadedModule, ModuleLoadError>;

    /// Normalizes an entry locator the embedder supplied directly, without a
    /// referrer, and mints its stable [`ModuleKey`].
    ///
    /// The entry module arrives as a locator rather than a specifier, so it never
    /// passes through [`resolve`](Self::resolve) — and if it is not normalized
    /// the same way, a module that imports the entry back lands under a second
    /// key and the entry is loaded, and evaluated, twice. Unnormalizable keys
    /// are returned unchanged: an in-memory entry has no path to normalize.
    fn canonical_key(&self, key: &str) -> ModuleKey {
        ModuleKey::from_host(key)
    }
}

/// A host loader that proves an execution cannot consult module state.
///
/// Differential protocols without an embedded graph use this rather than a
/// filesystem loader. Both operations fail before path normalization or IO.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct RejectAllModuleLoader;

impl HostModuleLoader for RejectAllModuleLoader {
    fn resolve(
        &self,
        referrer: Option<&ModuleKey>,
        request: &ModuleRequestKeyIr,
    ) -> Result<ModuleKey, ModuleLoadError> {
        Err(ModuleLoadError::Denied {
            specifier: request.specifier().to_string(),
            reason: match referrer {
                Some(referrer) => format!(
                    "module loading disabled by host policy for referrer {}",
                    referrer.as_str()
                ),
                None => "module loading disabled by host policy".to_string(),
            },
        })
    }

    fn load(&self, key: &ModuleKey) -> Result<LoadedModule, ModuleLoadError> {
        Err(ModuleLoadError::Denied {
            specifier: key.as_str().to_string(),
            reason: "module loading disabled by host policy".to_string(),
        })
    }
}

/// The source authority for a module graph entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleEntry {
    /// Load the entry through [`HostModuleLoader::load`].
    HostLoad {
        /// Unnormalized locator the host turns into the entry's [`ModuleKey`].
        /// Relative specifiers inside the entry resolve against its directory.
        locator: String,
    },
    /// Use source text already supplied by the embedder.
    ///
    /// Load-bearing for test262: the harness prepends `assert.js` and `sta.js`
    /// to the entry module's text, but relative specifiers inside it must
    /// still resolve against the real on-disk directory that `locator` names.
    InMemory {
        /// Unnormalized locator the host turns into the entry's [`ModuleKey`].
        locator: String,
        /// Exact entry source to parse.
        source_text: String,
    },
}

impl ModuleEntry {
    fn locator(&self) -> &str {
        match self {
            Self::HostLoad { locator } | Self::InMemory { locator, .. } => locator,
        }
    }
}

/// A filesystem loader confined to a root directory.
#[derive(Debug, Clone)]
pub struct FilesystemModuleLoader {
    root: PathBuf,
    entry_dir: PathBuf,
}

impl FilesystemModuleLoader {
    /// Builds a loader rooted at `root`, or at `entry`'s parent directory when
    /// `root` is `None`.
    ///
    /// # Errors
    /// Returns an error when neither a root nor a usable working directory is
    /// available.
    pub fn new(root: Option<&str>, entry: Option<&str>) -> Result<Self, ModuleLoadError> {
        let entry_dir = entry
            .map(Path::new)
            .and_then(Path::parent)
            .filter(|parent| !parent.as_os_str().is_empty())
            .map(Path::to_path_buf)
            .or_else(|| std::env::current_dir().ok())
            .unwrap_or_else(|| PathBuf::from("."));
        let root = root.map(PathBuf::from).unwrap_or_else(|| entry_dir.clone());
        let root = root.canonicalize().unwrap_or(root);
        let entry_dir = entry_dir.canonicalize().unwrap_or(entry_dir);
        Ok(Self { root, entry_dir })
    }

    /// Rejects any path that escapes the root, before it is ever read.
    fn confine(&self, specifier: &str, candidate: &Path) -> Result<PathBuf, ModuleLoadError> {
        let normalized = normalize(candidate);
        let resolved = normalized.canonicalize().unwrap_or(normalized);
        if resolved.starts_with(&self.root) {
            return Ok(resolved);
        }
        Err(ModuleLoadError::Denied {
            specifier: specifier.to_string(),
            reason: format!("resolves outside module root {}", self.root.display()),
        })
    }
}

impl HostModuleLoader for FilesystemModuleLoader {
    fn resolve(
        &self,
        referrer: Option<&ModuleKey>,
        request: &ModuleRequestKeyIr,
    ) -> Result<ModuleKey, ModuleLoadError> {
        // No import attribute is implemented, `type: "json"` included: see
        // [`LoadedModuleKind`]. `AllImportAttributesSupported` failing is a
        // resolution failure, which the caller turns into a link error rather
        // than aborting the compile.
        if let Some(attribute) = request.attributes().first() {
            return Err(ModuleLoadError::UnsupportedAttribute {
                key: attribute.key.clone(),
                value: attribute.value.clone(),
            });
        }
        let specifier = request.specifier();
        let base = referrer
            .map(|key| PathBuf::from(key.as_str()))
            .as_deref()
            .and_then(Path::parent)
            .filter(|parent| !parent.as_os_str().is_empty())
            .map_or_else(|| self.entry_dir.clone(), Path::to_path_buf);
        let candidate = if Path::new(specifier).is_absolute() {
            PathBuf::from(specifier)
        } else if specifier.starts_with("./") || specifier.starts_with("../") {
            base.join(specifier)
        } else {
            // A bare specifier has no package resolution here; it is looked up
            // relative to the root, and nothing else.
            self.root.join(specifier)
        };
        let resolved = self.confine(specifier, &candidate)?;
        if !resolved.is_file() {
            return Err(ModuleLoadError::NotFound {
                specifier: specifier.to_string(),
                referrer: referrer.cloned(),
            });
        }
        Ok(ModuleKey::from_host(
            resolved.to_string_lossy().into_owned(),
        ))
    }

    fn canonical_key(&self, key: &str) -> ModuleKey {
        // Exactly what `resolve` produces, so an entry the graph reaches again
        // through a relative specifier lands on the key it already holds.
        // A key naming nothing on disk (an in-memory entry) stays as written.
        normalize(Path::new(key)).canonicalize().map_or_else(
            |_| ModuleKey::from_host(key),
            |resolved| ModuleKey::from_host(resolved.to_string_lossy().into_owned()),
        )
    }

    fn load(&self, key: &ModuleKey) -> Result<LoadedModule, ModuleLoadError> {
        let path = PathBuf::from(key.as_str());
        let text = std::fs::read_to_string(&path).map_err(|error| ModuleLoadError::Io {
            key: key.clone(),
            message: error.to_string(),
        })?;
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            // A `.json` file is a JSON module whichever way it was imported —
            // with the attribute (16.2.1.7 module-type check) or without it.
            // Neither form is implemented; see [`LoadedModuleKind`].
            return Err(ModuleLoadError::UnsupportedAttribute {
                key: "type".to_string(),
                value: "json".to_string(),
            });
        }
        Ok(LoadedModule {
            meta_url: format!("file://{}", path.display()),
            kind: LoadedModuleKind::Source(text),
            key: key.clone(),
        })
    }
}

/// Loads the transitive closure of `entry` and returns it in the shape
/// `lila-ir` consumes.
///
/// A request the loader rejects is simply left unresolved: `lila-ir` reports
/// it as a link diagnostic, which is where the `SyntaxError` a
/// `phase: resolution` negative expects comes from.
///
/// # Errors
/// Returns an error only when the *entry* module itself cannot be read.
pub fn load_module_graph(
    entry: &ModuleEntry,
    loader: &dyn HostModuleLoader,
) -> Result<ModuleGraphSources, ModuleLoadError> {
    // The entry never passes through `resolve`, so normalize its key here or a
    // module that imports the entry back gets a second copy of it.
    let entry_key = loader.canonical_key(entry.locator());
    let entry_module = match entry {
        ModuleEntry::InMemory { source_text, .. } => ModuleSourceIr::new(
            entry_key.clone(),
            source_text.clone(),
            format!("file://{}", entry_key.as_str()),
        ),
        ModuleEntry::HostLoad { .. } => {
            let loaded = loader.load(&entry_key)?;
            ModuleSourceIr::new(loaded.key, module_text(loaded.kind), loaded.meta_url)
        }
    };

    load_module_graph_from_entry(entry_module, loader)
}

/// [`load_module_graph`] with an entry already parsed by the compilation front
/// end. Dependencies still enter through [`ModuleSourceIr::new`], while the
/// entry retains this exact syntax product instead of parsing its text again.
pub(crate) fn load_module_graph_from_parsed(
    entry_locator: &str,
    source: ParsedModule,
    loader: &dyn HostModuleLoader,
) -> Result<ModuleGraphSources, ModuleLoadError> {
    let entry_key = loader.canonical_key(entry_locator);
    let entry_module = ModuleSourceIr::from_parsed(
        entry_key.clone(),
        format!("file://{}", entry_key.as_str()),
        source,
    );
    load_module_graph_from_entry(entry_module, loader)
}

/// Script-entry counterpart of [`load_module_graph_from_parsed`]. The entry
/// keeps Script grammar and contributes only its `import()` requests; every
/// loaded target is still parsed with Module grammar.
pub(crate) fn load_module_graph_from_parsed_script(
    entry_locator: &str,
    source: ParsedScript,
    loader: &dyn HostModuleLoader,
) -> Result<ModuleGraphSources, ModuleLoadError> {
    let entry_key = loader.canonical_key(entry_locator);
    let entry_module = ModuleSourceIr::from_parsed_script(
        entry_key.clone(),
        format!("file://{}", entry_key.as_str()),
        source,
    );
    load_module_graph_from_entry(entry_module, loader)
}

fn load_module_graph_from_entry(
    entry_module: ModuleSourceIr,
    loader: &dyn HostModuleLoader,
) -> Result<ModuleGraphSources, ModuleLoadError> {
    let mut modules = vec![entry_module];
    // Both the key a request resolved to and the key its load reported map
    // here, so a loader that normalizes further on load still reads each
    // module once.
    let mut indices: BTreeMap<ModuleKey, u32> = BTreeMap::new();
    indices.insert(modules[0].key().clone(), 0);
    let mut resolutions = Vec::new();
    let mut cursor = 0usize;

    while cursor < modules.len() {
        let referrer = u32::try_from(cursor).unwrap_or(u32::MAX);
        let key = modules[cursor].key().clone();
        let requests = modules[cursor].module_requests();
        cursor += 1;

        // A rejected module produces no requests; `lila-ir` reports the
        // retained parse failure itself without trying the parser again.
        let Some(requests) = requests else {
            continue;
        };
        for request in requests {
            // A request the loader rejects stays unresolved on purpose: the
            // link stage turns it into the SyntaxError the spec asks for,
            // instead of the whole compile failing here.
            let Ok(target_key) = loader.resolve(Some(&key), &request) else {
                continue;
            };
            if let Some(index) = indices.get(&target_key).copied() {
                resolutions.push((referrer, request, index));
                continue;
            }
            let Ok(loaded) = loader.load(&target_key) else {
                continue;
            };
            let text = module_text(loaded.kind);
            // The load may report a key the graph already holds: module map
            // identity is keyed on the loaded key, not on the specifier that
            // reached it. Text that disagrees is pushed through as a duplicate
            // key so `build_graph` reports one `InconsistentLoad` rather than
            // silently running one of the two.
            let existing = indices.get(&loaded.key).copied();
            let target = match existing {
                Some(index) if modules[index as usize].source_text() == text => index,
                _ => {
                    let index = u32::try_from(modules.len()).unwrap_or(u32::MAX);
                    modules.push(ModuleSourceIr::new(
                        loaded.key.clone(),
                        text,
                        loaded.meta_url,
                    ));
                    if existing.is_none() {
                        indices.insert(loaded.key, index);
                    }
                    index
                }
            };
            indices.insert(target_key, target);
            resolutions.push((referrer, request, target));
        }
    }

    Ok(ModuleGraphSources {
        modules,
        entry: 0,
        resolutions,
    })
}

/// Source text of a loaded module, whatever kind it is.
fn module_text(kind: LoadedModuleKind) -> String {
    match kind {
        LoadedModuleKind::Source(text) => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_ir::ImportAttributeIr;
    use std::fs;

    /// `base/root` is the module root; `base` itself is off-limits, which is
    /// what the traversal tests reach for.
    fn temp_base(name: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "lila-modules-{name}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(base.join("root")).unwrap();
        base
    }

    fn write_tree(root: &Path, files: &[(&str, &str)]) {
        for (name, text) in files {
            let path = root.join(name);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(path, text).unwrap();
        }
    }

    fn loader_at(root: &Path) -> FilesystemModuleLoader {
        let root = root.to_string_lossy().into_owned();
        FilesystemModuleLoader::new(Some(root.as_str()), None).unwrap()
    }

    fn entry_at(path: &Path) -> ModuleEntry {
        ModuleEntry::HostLoad {
            locator: path.to_string_lossy().into_owned(),
        }
    }

    #[test]
    fn a_host_entry_loads_its_source_through_the_loader() {
        let base = temp_base("host-entry");
        let root = base.join("root");
        write_tree(&root, &[("entry.js", "export const origin = 'host';")]);
        let loader = loader_at(&root);

        let sources = load_module_graph(&entry_at(&root.join("entry.js")), &loader).unwrap();
        assert_eq!(
            sources.modules[0].source_text(),
            "export const origin = 'host';"
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn an_in_memory_entry_does_not_ask_the_host_to_load_it() {
        let entry = ModuleEntry::InMemory {
            locator: "virtual/entry.js".to_string(),
            source_text: "export const origin = 'memory';".to_string(),
        };

        let sources = load_module_graph(&entry, &RejectAllModuleLoader).unwrap();
        assert_eq!(sources.modules[0].key().as_str(), "virtual/entry.js");
        assert_eq!(
            sources.modules[0].source_text(),
            "export const origin = 'memory';"
        );
    }

    #[test]
    fn reject_all_loader_never_resolves_or_loads() {
        let loader = RejectAllModuleLoader;
        let referrer = ModuleKey::from_host("entry.js");
        let request = ModuleRequestKeyIr::plain("./ambient.js");
        assert!(matches!(
            loader.resolve(Some(&referrer), &request),
            Err(ModuleLoadError::Denied { .. })
        ));
        assert!(matches!(
            loader.load(&ModuleKey::from_host("ambient.js")),
            Err(ModuleLoadError::Denied { .. })
        ));
    }

    #[test]
    fn parent_traversal_out_of_the_root_is_denied() {
        let base = temp_base("traversal");
        let root = base.join("root");
        write_tree(&base, &[("outside.js", "export const secret = 1;")]);
        write_tree(&root, &[("entry.js", "import '../outside.js';")]);
        let loader = loader_at(&root);

        let referrer = ModuleKey::from_host(root.join("entry.js").to_string_lossy().into_owned());
        let denied = loader.resolve(Some(&referrer), &ModuleRequestKeyIr::plain("../outside.js"));
        assert!(
            matches!(denied, Err(ModuleLoadError::Denied { .. })),
            "{denied:?}"
        );

        // The graph still loads: the rejected request is simply unresolved, and
        // becomes a link error rather than a failed compile.
        let sources = load_module_graph(&entry_at(&root.join("entry.js")), &loader).unwrap();
        assert_eq!(sources.modules.len(), 1);
        assert!(sources.resolutions.is_empty());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn an_absolute_specifier_outside_the_root_is_denied() {
        let base = temp_base("absolute");
        let root = base.join("root");
        write_tree(&base, &[("outside.js", "export const secret = 1;")]);
        write_tree(&root, &[("entry.js", "")]);
        let loader = loader_at(&root);

        let outside = base.join("outside.js").to_string_lossy().into_owned();
        let referrer = ModuleKey::from_host(root.join("entry.js").to_string_lossy().into_owned());
        let denied = loader.resolve(Some(&referrer), &ModuleRequestKeyIr::plain(outside));
        assert!(
            matches!(denied, Err(ModuleLoadError::Denied { .. })),
            "{denied:?}"
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_pointing_out_of_the_root_is_denied() {
        let base = temp_base("symlink");
        let root = base.join("root");
        write_tree(&base, &[("outside.js", "export const secret = 1;")]);
        write_tree(&root, &[("entry.js", "")]);
        std::os::unix::fs::symlink(base.join("outside.js"), root.join("link.js")).unwrap();
        let loader = loader_at(&root);

        let referrer = ModuleKey::from_host(root.join("entry.js").to_string_lossy().into_owned());
        let denied = loader.resolve(Some(&referrer), &ModuleRequestKeyIr::plain("./link.js"));
        assert!(
            matches!(denied, Err(ModuleLoadError::Denied { .. })),
            "{denied:?}"
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn parent_components_never_pop_the_filesystem_root() {
        // Without the floor, this would normalize to the *relative* `etc`,
        // which the confinement check would then re-anchor at the root it is
        // supposed to be guarding.
        assert_eq!(
            normalize(Path::new("/a/../../etc/passwd")),
            PathBuf::from("/etc/passwd")
        );
        assert_eq!(normalize(Path::new("/a/b/../c")), PathBuf::from("/a/c"));
    }

    #[test]
    fn two_specifiers_naming_one_file_load_one_module() {
        let base = temp_base("module-map");
        let root = base.join("root");
        write_tree(
            &root,
            &[
                (
                    "entry.js",
                    "import { x } from './dep.js';\nimport * as ns from './sub/../dep.js';\nx; ns;",
                ),
                ("dep.js", "export const x = 1;"),
            ],
        );
        let loader = loader_at(&root);

        let sources = load_module_graph(&entry_at(&root.join("entry.js")), &loader).unwrap();
        assert_eq!(sources.modules.len(), 2);
        assert_eq!(sources.resolutions.len(), 2);
        assert!(sources
            .resolutions
            .iter()
            .all(|(referrer, _, target)| *referrer == 0 && *target == 1));
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn phase_variants_share_one_filesystem_resolution() {
        let base = temp_base("phase-free-resolution");
        let root = base.join("root");
        write_tree(
            &root,
            &[
                (
                    "entry.js",
                    "import './dep.js';\n\
                     import defer * as deferred from './dep.js';\n\
                     import source artifact from './dep.js';\n\
                     deferred; artifact;",
                ),
                ("dep.js", "export const value = 1;"),
            ],
        );
        let loader = loader_at(&root);

        let sources = load_module_graph(&entry_at(&root.join("entry.js")), &loader).unwrap();
        assert_eq!(sources.modules.len(), 2);
        assert_eq!(
            sources.resolutions,
            vec![(0, ModuleRequestKeyIr::plain("./dep.js"), 1)]
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn an_entry_that_imports_itself_stays_one_module() {
        let base = temp_base("self-import");
        let root = base.join("root");
        write_tree(
            &root,
            // The self-import has to be aliased. Importing your own export
            // under its own name declares it twice in one module environment -
            // the import binding and the `export const` - so
            // `import { x } from './entry.js'; export const x = 1;` is a
            // SyntaxError (16.2.1.2) rather than a graph shape, and the parse
            // fails before any request is scanned.
            &[(
                "entry.js",
                "import { x as y } from './entry.js';\nexport const x = 1;\ny;",
            )],
        );
        let loader = loader_at(&root);

        // A key spelled differently from what `resolve` produces: normalizing
        // it is what keeps the entry from being loaded a second time.
        let entry = entry_at(&root.join(".").join("entry.js"));
        let sources = load_module_graph(&entry, &loader).unwrap();
        assert_eq!(sources.modules.len(), 1);
        assert_eq!(sources.resolutions.len(), 1);
        assert_eq!(sources.resolutions[0].2, 0);
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn a_missing_module_is_left_unresolved() {
        let base = temp_base("missing");
        let root = base.join("root");
        write_tree(&root, &[("entry.js", "import { x } from './nope.js';\nx;")]);
        let loader = loader_at(&root);

        let referrer = ModuleKey::from_host(root.join("entry.js").to_string_lossy().into_owned());
        let missing = loader.resolve(Some(&referrer), &ModuleRequestKeyIr::plain("./nope.js"));
        assert!(
            matches!(missing, Err(ModuleLoadError::NotFound { .. })),
            "{missing:?}"
        );

        let sources = load_module_graph(&entry_at(&root.join("entry.js")), &loader).unwrap();
        assert_eq!(sources.modules.len(), 1);
        assert!(sources.resolutions.is_empty());
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn an_unsupported_import_attribute_is_rejected() {
        let base = temp_base("attributes");
        let root = base.join("root");
        write_tree(&root, &[("dep.js", "export const x = 1;")]);
        let loader = loader_at(&root);

        let request = ModuleRequestKeyIr::try_new(
            "./dep.js",
            vec![ImportAttributeIr {
                key: "type".to_string(),
                value: "css".to_string(),
            }],
        )
        .expect("the test attribute key is unique");
        let referrer = ModuleKey::from_host(root.join("dep.js").to_string_lossy().into_owned());
        let rejected = loader.resolve(Some(&referrer), &request);
        assert!(
            matches!(rejected, Err(ModuleLoadError::UnsupportedAttribute { .. })),
            "{rejected:?}"
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn a_json_module_is_rejected_rather_than_parsed_as_ecmascript() {
        let base = temp_base("json");
        let root = base.join("root");
        write_tree(&root, &[("data.json", "{\"a\": 1}")]);
        let loader = loader_at(&root);

        // With the attribute: `AllImportAttributesSupported` fails.
        let request = ModuleRequestKeyIr::try_new(
            "./data.json",
            vec![ImportAttributeIr {
                key: "type".to_string(),
                value: "json".to_string(),
            }],
        )
        .expect("the test attribute key is unique");
        let referrer = ModuleKey::from_host(root.join("entry.js").to_string_lossy().into_owned());
        assert!(
            matches!(
                loader.resolve(Some(&referrer), &request),
                Err(ModuleLoadError::UnsupportedAttribute { .. })
            ),
            "type=json must not resolve while JSON modules are unimplemented"
        );

        // Without it: the load refuses, so the JSON text never reaches the
        // ECMAScript parser as a module body.
        let key = ModuleKey::from_host(root.join("data.json").to_string_lossy().into_owned());
        assert!(
            matches!(
                loader.load(&key),
                Err(ModuleLoadError::UnsupportedAttribute { .. })
            ),
            "a .json file must not load as a Source Text Module Record"
        );
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn a_transitive_closure_loads_every_module_once() {
        let base = temp_base("closure");
        let root = base.join("root");
        write_tree(
            &root,
            &[
                ("entry.js", "import './a.js';\nimport './b.js';"),
                ("a.js", "import './shared.js';"),
                ("b.js", "import './shared.js';"),
                ("shared.js", "export const x = 1;"),
            ],
        );
        let loader = loader_at(&root);

        let sources = load_module_graph(&entry_at(&root.join("entry.js")), &loader).unwrap();
        assert_eq!(sources.modules.len(), 4);
        let mut keys: Vec<&str> = sources
            .modules
            .iter()
            .map(|source| source.key().as_str())
            .collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), 4);
        assert_eq!(sources.resolutions.len(), 4);
        let _ = fs::remove_dir_all(&base);
    }
}
