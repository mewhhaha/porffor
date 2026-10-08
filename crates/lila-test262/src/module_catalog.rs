//! The Test262 host's complete module catalog for Wasm-AOT runs.
//!
//! Lila compiles ahead of time, so every module a program can load is declared
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

use lila_engine::{
    EmbeddedModuleEntryInput, EmbeddedModuleGoal, EmbeddedModuleGraph, EmbeddedModuleInput,
    EmbeddedModuleReferrer, EmbeddedModuleResolutionInput, EmbeddedModuleSourceInput,
};
use lila_ir::{ModuleKey, ModuleRequestKeyIr, ModuleSourceIr};

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

/// What a source text can ask the host for, found without a second parser.
struct ModuleLoadingSyntax {
    /// Any `import(...)`, `import.defer(...)`, `import.source(...)` or
    /// `importValue(` appears.
    loads_modules: bool,
    /// Some dynamic import specifier is not a plain string literal.
    computed_specifier: bool,
}

impl ModuleLoadingSyntax {
    fn scan(text: &str) -> Self {
        let bytes = text.as_bytes();
        let mut syntax = Self {
            loads_modules: text.contains("importValue("),
            computed_specifier: false,
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
            cursor = skip_whitespace(bytes, cursor + 1);
            if !literal_specifier_at(bytes, cursor) {
                syntax.computed_specifier = true;
            }
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

/// True for `'...'` or `"..."` followed by `)` or `,`.
fn literal_specifier_at(bytes: &[u8], start: usize) -> bool {
    let Some(&quote) = bytes.get(start) else {
        return false;
    };
    if quote != b'\'' && quote != b'"' {
        return false;
    }
    let mut cursor = start + 1;
    while let Some(&byte) = bytes.get(cursor) {
        match byte {
            b'\\' => cursor += 2,
            _ if byte == quote => {
                let after = skip_whitespace(bytes, cursor + 1);
                return matches!(bytes.get(after), Some(b')' | b','));
            }
            _ => cursor += 1,
        }
    }
    false
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

struct Catalog {
    root: PathBuf,
    entry_path: PathBuf,
    goal: EmbeddedModuleGoal,
    /// Every readable file some row targets, by canonical path.
    texts: BTreeMap<PathBuf, String>,
    edges: BTreeMap<RequestRow, PathBuf>,
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
            for (specifier, attributes) in &rows {
                if let Some(target) = self.resolve(&base, specifier, attributes) {
                    if let Some(parent) = target.parent() {
                        referenced_dirs.insert(parent.to_path_buf());
                    }
                }
            }
            if scan.computed_specifier {
                for directory in &referenced_dirs {
                    rows.extend(self.fixture_rows(&base, directory));
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
        let is_entry_module = self.goal == EmbeddedModuleGoal::Module && target == self.entry_path;
        if !is_entry_module {
            let text = self.texts[&target].clone();
            pending.push_back((Node::Module(target.clone()), text));
        }
        self.edges.insert(key, target);
    }

    fn scan_source(&self, node: &Node, text: &str) -> ScannedSource {
        let syntax = ModuleLoadingSyntax::scan(text);
        let source = match node {
            Node::Realm => None,
            Node::ScriptEntry => {
                match lila_front::parse(text, lila_front::ParseOptions::script()) {
                    Ok(lila_front::ParsedSource::Script(script)) => {
                        Some(ModuleSourceIr::from_parsed_script(
                            ModuleKey::from_host("entry"),
                            String::new(),
                            script,
                        ))
                    }
                    _ => None,
                }
            }
            Node::Module(path) if is_json(path) => Some(ModuleSourceIr::json(
                ModuleKey::from_host("module"),
                text.to_owned(),
                String::new(),
            )),
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
            computed_specifier: syntax.computed_specifier,
        }
    }

    /// Rows naming each `*_FIXTURE*` file of `directory`, as a computed
    /// specifier written relative to `base` would spell them.
    fn fixture_rows(&self, base: &Path, directory: &Path) -> Vec<(String, Vec<(String, String)>)> {
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
            let attributes = if is_json(&path) {
                vec![("type".to_owned(), "json".to_owned())]
            } else {
                Vec::new()
            };
            rows.push((specifier, attributes));
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
    ) -> Option<PathBuf> {
        let json = matches!(attributes, [(key, value)] if key == "type" && value == "json");
        if !attributes.is_empty() && !json {
            return None;
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
        if !resolved.starts_with(&self.root) || !resolved.is_file() || is_json(&resolved) != json {
            return None;
        }
        if !self.texts.contains_key(&resolved) {
            let text = fs::read_to_string(&resolved).ok()?;
            self.texts.insert(resolved.clone(), text);
        }
        Some(resolved)
    }

    fn into_graph(self, entry_source: &str) -> Result<CaseModuleCatalog, String> {
        let entry_identity = identity(&self.entry_path)?;
        let entry = EmbeddedModuleEntryInput {
            goal: self.goal,
            identity: entry_identity.clone(),
            source: entry_source.to_owned(),
            meta_url: format!("lila://{entry_identity}"),
        };
        let targets: BTreeSet<&PathBuf> = self.edges.values().collect();
        let mut modules = Vec::new();
        for path in targets {
            if self.goal == EmbeddedModuleGoal::Module && *path == self.entry_path {
                continue;
            }
            let identity = identity(path)?;
            let input = EmbeddedModuleSourceInput {
                meta_url: format!("lila://{identity}"),
                identity,
                source: self.texts[path].clone(),
            };
            modules.push(if is_json(path) {
                EmbeddedModuleInput::Json(input)
            } else {
                EmbeddedModuleInput::SourceText(input)
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
                target: identity(target)?,
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
    computed_specifier: bool,
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
