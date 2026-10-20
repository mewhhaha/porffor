//! The Test262 host's complete module catalog for Wasm-AOT runs.
//!
//! Scriptw compiles ahead of time, so every module a program can load is declared
//! before compilation. The host policy is: the modules available to a case are
//! the `*_FIXTURE*` files beside it plus everything reachable from the case and
//! from those modules by relative specifier. Each resolution row mirrors what
//! `FilesystemModuleLoader::resolve` accepts (same lexical normalization, root
//! confinement, JSON/source split and attribute rule); a request the filesystem
//! would reject has no row, so the runtime rejects it as a missing file would.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use scriptw_engine::{
    EmbeddedModuleBytesInput, EmbeddedModuleContents, EmbeddedModuleEntryInput, EmbeddedModuleGoal,
    EmbeddedModuleGraph, EmbeddedModuleInput, EmbeddedModuleKind, EmbeddedModuleReferrer,
    EmbeddedModuleResolutionInput, EmbeddedModuleSourceInput, EmbeddedModuleSourceValueInput,
};
use scriptw_ir::{
    DynamicImportAttributesIr, DynamicImportSiteIr, ModuleKey, ModuleRequestKeyIr, ModuleSourceIr,
};

#[cfg(test)]
mod tests;

/// The declared graph and the virtual identity the entry must compile under.
pub(crate) struct CaseModuleCatalog {
    pub(crate) graph: Arc<EmbeddedModuleGraph>,
}

/// Declares the catalog for a case, or `None` when it cannot load modules.
///
/// `entry_source` is the exact text the engine will compile; `original_source`
/// is the test body used for request discovery.
pub(crate) fn declare(
    case_path: &Path,
    goal: EmbeddedModuleGoal,
    entry_source: &str,
    original_source: &str,
) -> Result<Option<CaseModuleCatalog>, String> {
    let syntax = ModuleLoadingSyntax::scan(original_source);
    if goal == EmbeddedModuleGoal::Script && !syntax.loads_modules {
        return Ok(None);
    }
    let entry_path = case_path
        .canonicalize()
        .map_err(|error| format!("cannot canonicalize {}: {error}", case_path.display()))?;
    let root = entry_path
        .parent()
        .ok_or_else(|| format!("{} has no parent directory", entry_path.display()))?
        .to_path_buf();
    let mut catalog = Catalog {
        root,
        entry_path,
        goal,
        texts: BTreeMap::new(),
        edges: BTreeMap::new(),
    };
    catalog.discover(original_source)?;
    catalog.into_graph(entry_source).map(Some)
}

/// Keep costly finite-source lowering lazy behind its necessary AST condition.
/// The discovery callback still owns the intrinsic and source-context proof.
fn include_prepared_script_imports_if_needed(
    source: &mut ModuleSourceIr,
    discover: impl FnOnce() -> Vec<DynamicImportSiteIr>,
) {
    if source.may_contain_direct_eval_call() {
        source.include_prepared_script_import_sites(&discover());
    }
}

/// What a source text can ask the host for, found without a second parser.
struct ModuleLoadingSyntax {
    /// Any `import(...)`, `import.defer(...)`, `import.source(...)` or
    /// `importValue(` appears.
    loads_modules: bool,
}

impl ModuleLoadingSyntax {
    fn scan(text: &str) -> Self {
        let bytes = text.as_bytes();
        let mut syntax = Self {
            loads_modules: text.contains("importValue("),
        };
        let mut from = 0;
        while let Some(found) = text[from..].find("import") {
            let start = from + found;
            let end = start + "import".len();
            from = end;
            let ident = |byte: u8| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$');
            if start > 0 && (ident(bytes[start - 1]) || bytes[start - 1] == b'.') {
                continue;
            }
            let mut cursor = skip_whitespace(bytes, end);
            if bytes.get(cursor) == Some(&b'.') {
                cursor = skip_whitespace(bytes, cursor + 1);
                let rest = &text[cursor..];
                let Some(phase) = ["defer", "source"]
                    .into_iter()
                    .find(|phase| rest.starts_with(phase))
                else {
                    continue;
                };
                cursor = skip_whitespace(bytes, cursor + phase.len());
            }
            if bytes.get(cursor) != Some(&b'(') {
                continue;
            }
            syntax.loads_modules = true;
        }
        syntax
    }
}

fn skip_whitespace(bytes: &[u8], mut cursor: usize) -> usize {
    while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
        cursor += 1;
    }
    cursor
}

/// A resolution source: the entry, a loaded module, or the Realm base.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Node {
    /// A Script entry. A Module entry is `Module(entry_path)`.
    ScriptEntry,
    Module(PathBuf),
    Realm,
}

type RequestRow = (Node, String, Vec<(String, String)>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum ModuleTargetLocator {
    File(PathBuf),
    HostModuleSource,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ModuleTarget {
    locator: ModuleTargetLocator,
    kind: EmbeddedModuleKind,
}
impl ModuleTarget {
    fn file_path(&self) -> Option<&Path> {
        match &self.locator {
            ModuleTargetLocator::File(path) => Some(path),
            ModuleTargetLocator::HostModuleSource => None,
        }
    }
    fn identity(&self) -> Result<String, String> {
        match &self.locator {
            ModuleTargetLocator::File(path) => identity(path),
            ModuleTargetLocator::HostModuleSource => Ok("host/module-source".into()),
        }
    }
}

struct Catalog {
    root: PathBuf,
    entry_path: PathBuf,
    goal: EmbeddedModuleGoal,
    /// Every readable file some row targets, by canonical path.
    texts: BTreeMap<ModuleTarget, EmbeddedModuleContents>,
    edges: BTreeMap<RequestRow, ModuleTarget>,
}

impl Catalog {
    fn discover(&mut self, original_source: &str) -> Result<(), String> {
        let entry_node = match self.goal {
            EmbeddedModuleGoal::Script => Node::ScriptEntry,
            EmbeddedModuleGoal::Module => Node::Module(self.entry_path.clone()),
        };
        let mut pending = VecDeque::from([(entry_node.clone(), original_source.to_owned())]);
        let mut expanded = BTreeSet::new();
        while let Some((node, text)) = pending.pop_front() {
            if !expanded.insert(node.clone()) {
                continue;
            }
            let base = match &node {
                Node::ScriptEntry | Node::Realm => self.root.clone(),
                Node::Module(path) => path.parent().unwrap_or(&self.root).to_path_buf(),
            };
            let scan = self.scan_source(&node, &text);
            let mut referenced_dirs = BTreeSet::from([base.clone()]);
            let mut rows = Vec::new();
            for request in &scan.requests {
                rows.push((request.specifier().to_owned(), attributes_of(request)));
            }
            for site in &scan.dynamic_imports {
                let DynamicImportAttributesIr::Runtime = &site.attributes else {
                    continue;
                };
                let Some(specifier) = &site.static_specifier else {
                    continue;
                };
                // A literal locator and runtime attributes are independent.
                // Declare the supported exact kinds for this locator without
                // pretending that options have already been evaluated.
                for kind in [
                    EmbeddedModuleKind::Json,
                    EmbeddedModuleKind::Bytes,
                    EmbeddedModuleKind::Text,
                ] {
                    let request = request_for_kind(specifier, kind);
                    if site.permits_catalog_key(&request) {
                        rows.push((specifier.clone(), attributes_of(&request)));
                    }
                }
            }
            for (specifier, attributes) in &rows {
                if let Some(target) = self.resolve(&base, specifier, attributes) {
                    if let Some(parent) = target.file_path().and_then(Path::parent) {
                        referenced_dirs.insert(parent.to_path_buf());
                    }
                }
            }
            if scan
                .dynamic_imports
                .iter()
                .any(|site| site.static_specifier.is_none())
            {
                for directory in &referenced_dirs {
                    for request in self.fixture_rows(&base, directory) {
                        if scan.dynamic_imports.iter().any(|site| {
                            site.static_specifier.is_none() && site.permits_catalog_key(&request)
                        }) {
                            rows.push((request.specifier().to_owned(), attributes_of(&request)));
                        }
                    }
                }
            }
            for (specifier, attributes) in rows {
                self.add_edge(&node, &base, specifier, attributes, &mut pending);
            }
            // ShadowRealm `importValue` resolves against the entry directory,
            // independent of the active module, like `loader.resolve(None, ..)`.
            for request in scan.realm_requests {
                let (specifier, attributes) =
                    (request.specifier().to_owned(), attributes_of(&request));
                let root = self.root.clone();
                self.add_edge(&Node::Realm, &root, specifier, attributes, &mut pending);
            }
        }
        Ok(())
    }

    fn add_edge(
        &mut self,
        node: &Node,
        base: &Path,
        specifier: String,
        attributes: Vec<(String, String)>,
        pending: &mut VecDeque<(Node, String)>,
    ) {
        let key = (node.clone(), specifier, attributes);
        if self.edges.contains_key(&key) {
            return;
        }
        let Some(target) = self.resolve(base, &key.1, &key.2) else {
            return;
        };
        let is_entry_module = self.goal == EmbeddedModuleGoal::Module
            && target.file_path() == Some(self.entry_path.as_path())
            && target.kind == EmbeddedModuleKind::SourceText;
        if !is_entry_module {
            if let EmbeddedModuleContents::SourceText(text) = &self.texts[&target] {
                pending.push_back((
                    Node::Module(
                        target
                            .file_path()
                            .expect("SourceText target is a file")
                            .to_path_buf(),
                    ),
                    text.clone(),
                ));
            }
        }
        self.edges.insert(key, target);
    }

    fn scan_source(&self, node: &Node, text: &str) -> ScannedSource {
        let source = match node {
            Node::Realm => None,
            Node::ScriptEntry => {
                match scriptw_front::parse(text, scriptw_front::ParseOptions::script()) {
                    Ok(scriptw_front::ParsedSource::Script(script)) => {
                        let mut source = ModuleSourceIr::from_parsed_script(
                            ModuleKey::from_host("entry"),
                            String::new(),
                            script.clone(),
                        );
                        include_prepared_script_imports_if_needed(&mut source, || {
                            let session = scriptw_ir::LoweringSession::default();
                            // Reuse this Script parse and the compiler's finite-source
                            // proof only when the AST could call direct eval.
                            // This creates no module-loading authority by itself.
                            session.lower_source(
                                &scriptw_front::ParsedSource::Script(script),
                                scriptw_ir::HostSurfacePolicy::Test262,
                            );
                            session.prepared_script_import_sites()
                        });
                        Some(source)
                    }
                    _ => None,
                }
            }
            Node::Module(_) => Some(ModuleSourceIr::new(
                ModuleKey::from_host("module"),
                text.to_owned(),
                String::new(),
            )),
        };
        let Some(source) = source else {
            return ScannedSource::default();
        };
        ScannedSource {
            requests: source
                .module_loading_requests()
                .unwrap_or_default()
                .iter()
                .map(|request| request.key().clone())
                .collect(),
            realm_requests: source.realm_module_requests(),
            dynamic_imports: source.dynamic_import_sites().unwrap_or_default(),
        }
    }

    /// Rows naming each `*_FIXTURE*` file of `directory`, as a computed
    /// specifier written relative to `base` would spell them.
    fn fixture_rows(&self, base: &Path, directory: &Path) -> Vec<ModuleRequestKeyIr> {
        let Ok(entries) = fs::read_dir(directory) else {
            return Vec::new();
        };
        let mut rows = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            let is_fixture = path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.contains("_FIXTURE"));
            if !is_fixture || !path.is_file() {
                continue;
            }
            let Some(specifier) = relative_specifier(base, &path) else {
                continue;
            };
            let default_kind = if is_json(&path) {
                EmbeddedModuleKind::Json
            } else {
                EmbeddedModuleKind::SourceText
            };
            for kind in [
                default_kind,
                EmbeddedModuleKind::Bytes,
                EmbeddedModuleKind::Text,
            ] {
                rows.push(request_for_kind(&specifier, kind));
            }
        }
        rows.sort();
        rows
    }

    /// `FilesystemModuleLoader::resolve` plus the read `load` performs, for a
    /// referrer whose directory is `base`. `None` is a rejected request.
    fn resolve(
        &mut self,
        base: &Path,
        specifier: &str,
        attributes: &[(String, String)],
    ) -> Option<ModuleTarget> {
        let json = matches!(attributes, [(key, value)] if key == "type" && value == "json");
        let bytes = matches!(attributes, [(key, value)] if key == "type" && value == "bytes");
        let text = matches!(attributes, [(key, value)] if key == "type" && value == "text");
        if !attributes.is_empty() && !json && !bytes && !text {
            return None;
        }
        // Test262's implementation-defined host module-source contract. This
        // virtual module is declared here, never recognized by the compiler.
        if specifier == "<module source>" && attributes.is_empty() {
            let target = ModuleTarget {
                locator: ModuleTargetLocator::HostModuleSource,
                kind: EmbeddedModuleKind::ModuleSource,
            };
            self.texts.entry(target.clone()).or_insert_with(|| {
                EmbeddedModuleContents::ModuleSource {
                    source_tag: "ModuleSource".into(),
                }
            });
            return Some(target);
        }
        let candidate = if Path::new(specifier).is_absolute() {
            PathBuf::from(specifier)
        } else if specifier.starts_with("./") || specifier.starts_with("../") {
            base.join(specifier)
        } else {
            self.root.join(specifier)
        };
        let normalized = normalize(&candidate);
        let resolved = normalized.canonicalize().unwrap_or(normalized);
        if !resolved.starts_with(&self.root)
            || !resolved.is_file()
            || (!bytes && !text && is_json(&resolved) != json)
        {
            return None;
        }
        let target = ModuleTarget {
            locator: ModuleTargetLocator::File(resolved),
            kind: if bytes {
                EmbeddedModuleKind::Bytes
            } else if text {
                EmbeddedModuleKind::Text
            } else if json {
                EmbeddedModuleKind::Json
            } else {
                EmbeddedModuleKind::SourceText
            },
        };
        if !self.texts.contains_key(&target) {
            let contents = match target.kind {
                EmbeddedModuleKind::SourceText => EmbeddedModuleContents::SourceText(
                    fs::read_to_string(target.file_path().expect("filesystem target")).ok()?,
                ),
                EmbeddedModuleKind::Json => EmbeddedModuleContents::Json(
                    fs::read_to_string(target.file_path().expect("filesystem target")).ok()?,
                ),
                EmbeddedModuleKind::Text => EmbeddedModuleContents::Text(
                    fs::read_to_string(target.file_path().expect("filesystem target")).ok()?,
                ),
                EmbeddedModuleKind::Bytes => EmbeddedModuleContents::Bytes(
                    fs::read(target.file_path().expect("filesystem target")).ok()?,
                ),
                EmbeddedModuleKind::ModuleSource => {
                    unreachable!("virtual host source returned before filesystem loading")
                }
            };
            self.texts.insert(target.clone(), contents);
        }
        Some(target)
    }

    fn into_graph(self, entry_source: &str) -> Result<CaseModuleCatalog, String> {
        let entry_identity = identity(&self.entry_path)?;
        let entry = EmbeddedModuleEntryInput {
            goal: self.goal,
            identity: entry_identity.clone(),
            source: entry_source.to_owned(),
            meta_url: format!("scriptw://{entry_identity}"),
        };
        let targets: BTreeSet<&ModuleTarget> = self.edges.values().collect();
        let mut modules = Vec::new();
        for target in targets {
            if self.goal == EmbeddedModuleGoal::Module
                && target.file_path() == Some(self.entry_path.as_path())
                && target.kind == EmbeddedModuleKind::SourceText
            {
                continue;
            }
            let identity = target.identity()?;
            let meta_url = format!("scriptw://{identity}");
            modules.push(match &self.texts[target] {
                EmbeddedModuleContents::SourceText(text) => {
                    EmbeddedModuleInput::SourceText(EmbeddedModuleSourceInput {
                        identity,
                        meta_url,
                        source: text.clone(),
                    })
                }
                EmbeddedModuleContents::Json(text) => {
                    EmbeddedModuleInput::Json(EmbeddedModuleSourceInput {
                        identity,
                        meta_url,
                        source: text.clone(),
                    })
                }
                EmbeddedModuleContents::Text(text) => {
                    EmbeddedModuleInput::Text(EmbeddedModuleSourceInput {
                        identity,
                        meta_url,
                        source: text.clone(),
                    })
                }
                EmbeddedModuleContents::ModuleSource { source_tag } => {
                    EmbeddedModuleInput::ModuleSource(EmbeddedModuleSourceValueInput {
                        identity,
                        source_tag: source_tag.clone(),
                        meta_url,
                    })
                }
                EmbeddedModuleContents::Bytes(bytes) => {
                    EmbeddedModuleInput::Bytes(EmbeddedModuleBytesInput {
                        identity,
                        meta_url,
                        bytes: bytes.clone(),
                    })
                }
            });
        }
        let mut resolutions = Vec::new();
        for ((node, specifier, attributes), target) in &self.edges {
            let referrer = match node {
                Node::ScriptEntry => EmbeddedModuleReferrer::Script(entry_identity.clone()),
                Node::Module(path) => EmbeddedModuleReferrer::Module(identity(path)?),
                Node::Realm => EmbeddedModuleReferrer::Realm,
            };
            resolutions.push(EmbeddedModuleResolutionInput {
                referrer,
                specifier: specifier.clone(),
                attributes: attributes.clone(),
                target: target.identity()?,
            });
        }
        EmbeddedModuleGraph::try_new_typed(entry, modules, resolutions)
            .map(|graph| CaseModuleCatalog { graph })
            .map_err(|error| format!("invalid Test262 module catalog: {error}"))
    }
}

#[derive(Default)]
struct ScannedSource {
    requests: Vec<ModuleRequestKeyIr>,
    realm_requests: Vec<ModuleRequestKeyIr>,
    dynamic_imports: Vec<DynamicImportSiteIr>,
}

fn request_for_kind(specifier: &str, kind: EmbeddedModuleKind) -> ModuleRequestKeyIr {
    let type_value = match kind {
        EmbeddedModuleKind::SourceText | EmbeddedModuleKind::ModuleSource => None,
        EmbeddedModuleKind::Json => Some("json"),
        EmbeddedModuleKind::Bytes => Some("bytes"),
        EmbeddedModuleKind::Text => Some("text"),
    };
    ModuleRequestKeyIr::try_new(
        specifier,
        type_value
            .into_iter()
            .map(|value| scriptw_ir::ImportAttributeIr {
                key: "type".into(),
                value: value.into(),
            }),
    )
    .expect("one supported kind attribute has no duplicate key")
}

fn attributes_of(request: &ModuleRequestKeyIr) -> Vec<(String, String)> {
    request
        .attributes()
        .iter()
        .map(|attribute| (attribute.key.clone(), attribute.value.clone()))
        .collect()
}

fn is_json(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "json")
}

/// Lexical `.`/`..` folding, as the engine loader does before canonicalizing.
fn normalize(path: &Path) -> PathBuf {
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

/// `./name` or `../..../name` from directory `base` to `target`.
fn relative_specifier(base: &Path, target: &Path) -> Option<String> {
    let base: Vec<_> = base.components().collect();
    let target: Vec<_> = target.components().collect();
    let common = base.iter().zip(&target).take_while(|(a, b)| a == b).count();
    let ups = base.len() - common;
    let mut specifier = if ups == 0 {
        "./".to_owned()
    } else {
        "../".repeat(ups)
    };
    let rest: Option<Vec<&str>> = target[common..]
        .iter()
        .map(|component| component.as_os_str().to_str())
        .collect();
    specifier.push_str(&rest?.join("/"));
    Some(specifier)
}

/// Injective map from an absolute path to a catalog identity, which admits
/// only lowercase ASCII, digits and `-_/.`: every other byte, and `_` itself,
/// becomes `_xx`.
fn identity(path: &Path) -> Result<String, String> {
    let mut components = Vec::new();
    for component in path.components() {
        if let Component::Normal(name) = component {
            let name = name
                .to_str()
                .ok_or_else(|| format!("non-UTF-8 module path {}", path.display()))?;
            let mut encoded = String::with_capacity(name.len());
            for byte in name.bytes() {
                if byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'.')
                {
                    encoded.push(char::from(byte));
                } else {
                    encoded.push_str(&format!("_{byte:02x}"));
                }
            }
            components.push(encoded);
        }
    }
    Ok(components.join("/"))
}
