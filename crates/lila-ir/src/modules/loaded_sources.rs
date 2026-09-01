use lila_front::{ParseGoal, ParsedModule, ParsedScript, ParsedSource, SourceUnit};

use super::module_key::{ModuleKey, ANONYMOUS_MODULE_KEY};
use super::record::{
    scan_module_requests, scan_script_module_requests, ModuleRequestKeyIr, ModuleUnitId,
};

/// One already-loaded and exactly-once-parsed graph source, plus the key the
/// host resolved it under.
///
/// Every dependency is Module syntax. The distinguished entry may instead be
/// Script syntax for [`crate::lower_script_graph`]; the lowerer validates that
/// placement before graph construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSourceIr {
    key: ModuleKey,
    meta_url: String,
    pub(super) parse: ModuleParse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ModuleParse {
    Module(ParsedModule),
    ScriptEntry(ParsedScript),
    Rejected {
        source: SourceUnit,
        error: lila_front::ParseError,
    },
}

impl ModuleSourceIr {
    /// Parses one loaded module and retains either the typed syntax product or
    /// its structured rejection. There is no constructor for an unparsed
    /// module, so graph discovery and record construction must share this one
    /// parse attempt.
    #[must_use]
    pub fn new(key: ModuleKey, source_text: String, meta_url: String) -> Self {
        let options = lila_front::ParseOptions {
            goal: ParseGoal::Module,
            filename: Some(key.as_str().to_string()),
        };
        let parse = match lila_front::parse(source_text.clone(), options) {
            Ok(ParsedSource::Module(source)) => ModuleParse::Module(source),
            Ok(ParsedSource::Script(_)) => {
                unreachable!("Module parse options cannot produce Script syntax")
            }
            Err(error) => ModuleParse::Rejected {
                source: SourceUnit {
                    goal: ParseGoal::Module,
                    filename: Some(key.as_str().to_string()),
                    source_text,
                },
                error,
            },
        };
        Self {
            key,
            meta_url,
            parse,
        }
    }

    /// Builds a graph entry from a module already parsed by the compilation
    /// front end. This is the route that prevents the entry module from being
    /// parsed again merely because it participates in a graph.
    #[must_use]
    pub fn from_parsed(key: ModuleKey, meta_url: String, source: ParsedModule) -> Self {
        Self {
            key,
            meta_url,
            parse: ModuleParse::Module(source),
        }
    }

    /// Builds the distinguished entry of a Script graph from its original
    /// Script-goal parse. Only `import()` requests are visible from this shape;
    /// static module declarations are impossible in Script syntax.
    #[doc(hidden)]
    #[must_use]
    pub fn from_parsed_script(key: ModuleKey, meta_url: String, source: ParsedScript) -> Self {
        Self {
            key,
            meta_url,
            parse: ModuleParse::ScriptEntry(source),
        }
    }

    #[must_use]
    pub fn key(&self) -> &ModuleKey {
        &self.key
    }

    #[must_use]
    pub fn source_text(&self) -> &str {
        match &self.parse {
            ModuleParse::Module(source) => &source.source_text,
            ModuleParse::ScriptEntry(source) => &source.source_text,
            ModuleParse::Rejected { source, .. } => &source.source_text,
        }
    }

    #[must_use]
    pub fn meta_url(&self) -> &str {
        &self.meta_url
    }

    /// Requests needed by host graph discovery, derived from the retained AST.
    /// `None` means the one parse attempt was rejected and must not be retried.
    #[must_use]
    pub fn module_requests(&self) -> Option<Vec<ModuleRequestKeyIr>> {
        match &self.parse {
            ModuleParse::Module(source) => Some(scan_module_requests(source)),
            ModuleParse::ScriptEntry(source) => Some(scan_script_module_requests(source)),
            ModuleParse::Rejected { .. } => None,
        }
    }

    #[must_use]
    pub fn goal(&self) -> ParseGoal {
        match &self.parse {
            ModuleParse::Module(_) | ModuleParse::Rejected { .. } => ParseGoal::Module,
            ModuleParse::ScriptEntry(_) => ParseGoal::Script,
        }
    }
}

/// The loaded transitive closure of an entry module.
///
/// `resolutions` is the host's `HostResolveImportedModule` result table: for
/// each `(referrer, request key)` pair it names the unit that request resolves
/// to. Phase is occurrence metadata and does not participate in host identity.
/// A request with no entry here is an unresolved-module link error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleGraphSources {
    /// Every module in the closure. Index is the [`ModuleUnitId`].
    pub modules: Vec<ModuleSourceIr>,
    /// Index of the entry module in `modules`.
    pub entry: ModuleUnitId,
    /// `(referrer, request key) -> target` resolutions the host produced.
    ///
    /// ```compile_fail
    /// use lila_ir::{ModuleGraphSources, ModuleRequestIr};
    ///
    /// let mut sources = ModuleGraphSources {
    ///     modules: Vec::new(),
    ///     entry: 0,
    ///     resolutions: Vec::new(),
    /// };
    /// sources
    ///     .resolutions
    ///     .push((0, ModuleRequestIr::plain("./m.js"), 1));
    /// ```
    pub resolutions: Vec<(ModuleUnitId, ModuleRequestKeyIr, ModuleUnitId)>,
}

impl ModuleGraphSources {
    /// A one-node graph: a module that requests nothing, or whose requests the
    /// host could not resolve.
    #[must_use]
    pub fn single(source: &ParsedModule) -> Self {
        let key = source
            .filename
            .clone()
            .unwrap_or_else(|| ANONYMOUS_MODULE_KEY.to_string());
        Self {
            modules: vec![ModuleSourceIr::from_parsed(
                ModuleKey::from_host(key.clone()),
                key,
                source.clone(),
            )],
            entry: 0,
            resolutions: Vec::new(),
        }
    }
}
