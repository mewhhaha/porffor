use lila_front::{ParseGoal, ParsedModule, ParsedScript, ParsedSource, SourceUnit};

use super::module_key::{ModuleKey, ANONYMOUS_MODULE_KEY};
use super::record::{
    loading_requests_with_dynamic_imports, scan_module_loading_requests, scan_module_requests,
    scan_script_module_requests, script_dynamic_import_sites, ModuleRequestIr, ModuleRequestKeyIr,
    ModuleUnitId,
};

/// One already-loaded and exactly-once-parsed graph source, plus the key the
/// host resolved it under.
///
/// Every dependency is a Source Text or JSON Module record. The entry may instead be
/// Script syntax for [`crate::lower_script_graph`]; the lowerer validates that
/// placement before graph construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSourceIr {
    key: ModuleKey,
    meta_url: String,
    pub(super) parse: ModuleParse,
}

/// Record syntax is part of host identity and is never inferred from equal bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleKindIr {
    SourceText,
    Json,
    Script,
}

impl ModuleKindIr {
    pub const fn code(self) -> u8 {
        match self {
            Self::SourceText => 0,
            Self::Json => 1,
            Self::Script => 2,
        }
    }
    pub fn matches_request(self, request: &ModuleRequestKeyIr) -> bool {
        match self {
            Self::SourceText => !request
                .attributes()
                .iter()
                .any(|attribute| attribute.key == "type" && attribute.value == "json"),
            Self::Json => matches!(request.attributes(), [attribute]
                if attribute.key == "type" && attribute.value == "json"),
            Self::Script => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ModuleParse {
    Module(ParsedModule),
    Json(lila_front::ParsedJson),
    JsonRejected {
        source_text: String,
        error: lila_front::JsonParseError,
    },
    ScriptEntry(ParsedScript),
    Rejected {
        source: SourceUnit,
        error: lila_front::ParseError,
    },
}

impl ModuleSourceIr {
    /// Potential finite importValue targets. Discovery does not claim that the
    /// spelled method is intrinsic and never rewrites its ordinary call.
    pub fn realm_module_requests(&self) -> Vec<ModuleRequestKeyIr> {
        match &self.parse {
            ModuleParse::Module(source) => source.with_compiler_session(|ast, interner| {
                super::realm_request::literal_realm_requests(ast, interner)
            }),
            ModuleParse::ScriptEntry(source) => source.with_compiler_session(|ast, interner| {
                super::realm_request::literal_realm_requests(ast, interner)
            }),
            ModuleParse::Rejected { .. }
            | ModuleParse::Json(_)
            | ModuleParse::JsonRejected { .. } => Vec::new(),
        }
    }

    /// Parses JSON exactly once under its own grammar, retaining the actual result.
    pub fn json(key: ModuleKey, source_text: String, meta_url: String) -> Self {
        let parse = match lila_front::parse_json(source_text.clone()) {
            Ok(parsed) => ModuleParse::Json(parsed),
            Err(error) => ModuleParse::JsonRejected { source_text, error },
        };
        Self {
            key,
            meta_url,
            parse,
        }
    }

    pub const fn kind(&self) -> ModuleKindIr {
        match &self.parse {
            ModuleParse::Module(_) | ModuleParse::Rejected { .. } => ModuleKindIr::SourceText,
            ModuleParse::Json(_) | ModuleParse::JsonRejected { .. } => ModuleKindIr::Json,
            ModuleParse::ScriptEntry(_) => ModuleKindIr::Script,
        }
    }
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
            ModuleParse::Json(source) => source.source_text(),
            ModuleParse::JsonRejected { source_text, .. } => source_text,
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
            ModuleParse::Json(_) => Some(Vec::new()),
            ModuleParse::ScriptEntry(source) => Some(scan_script_module_requests(source)),
            ModuleParse::Rejected { .. } | ModuleParse::JsonRejected { .. } => None,
        }
    }

    /// Requests consumed by the host-loading driver. Source phase loads one
    /// record; Evaluation/Defer opens its dependencies. Phase-free host keys
    /// still coalesce, with a non-source occurrence promoting an earlier source
    /// occurrence without changing first-key order or the retained record.
    #[must_use]
    pub fn module_loading_requests(&self) -> Option<Vec<ModuleRequestIr>> {
        match &self.parse {
            ModuleParse::Module(source) => Some(scan_module_loading_requests(source)),
            ModuleParse::Json(_) => Some(Vec::new()),
            ModuleParse::ScriptEntry(source) => Some(loading_requests_with_dynamic_imports(
                Vec::new(),
                &script_dynamic_import_sites(source),
            )),
            ModuleParse::Rejected { .. } | ModuleParse::JsonRejected { .. } => None,
        }
    }

    /// Whether a retained Script entry contains an actual import call.
    ///
    /// Request discovery omits computed specifiers, so an empty request table
    /// cannot decide whether the Script needs an import dispatcher. The same
    /// retained AST scan used by record construction distinguishes calls from
    /// object or class methods named `import`, without another parse attempt.
    /// `None` means this source is not a successfully parsed Script entry.
    #[must_use]
    pub fn script_has_dynamic_import_sites(&self) -> Option<bool> {
        match &self.parse {
            ModuleParse::ScriptEntry(source) => Some(
                !script_dynamic_import_sites(source).is_empty()
                    || !self.realm_module_requests().is_empty(),
            ),
            ModuleParse::Module(_)
            | ModuleParse::Rejected { .. }
            | ModuleParse::Json(_)
            | ModuleParse::JsonRejected { .. } => None,
        }
    }

    #[must_use]
    pub fn goal(&self) -> ParseGoal {
        match &self.parse {
            ModuleParse::Module(_)
            | ModuleParse::Rejected { .. }
            | ModuleParse::Json(_)
            | ModuleParse::JsonRejected { .. } => ParseGoal::Module,
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
    /// Realm-origin host requests have no source referrer. In particular they
    /// never inherit the locator of the function containing importValue.
    pub realm_requests:
        std::collections::BTreeMap<ModuleRequestKeyIr, super::RealmModuleResolutionIr>,
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
    ///     realm_requests: Default::default(),
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
            realm_requests: Default::default(),
        }
    }
}
