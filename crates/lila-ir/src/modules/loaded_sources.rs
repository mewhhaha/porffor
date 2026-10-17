use lila_front::{ParseGoal, ParsedModule, ParsedScript, ParsedSource, SourceUnit};

use super::module_key::{ModuleKey, ANONYMOUS_MODULE_KEY};
use super::record::{
    loading_requests_with_dynamic_imports, module_dynamic_import_sites,
    scan_module_loading_requests, scan_module_requests, scan_script_module_requests,
    script_dynamic_import_sites, ModuleRequestIr, ModuleRequestKeyIr, ModuleUnitId,
};

/// One already-loaded and exactly-once-parsed graph source, plus the key the
/// host resolved it under.
///
/// Every dependency is a Source Text, JSON, Bytes, or Text Module record. The entry may instead be
/// Script syntax for [`crate::lower_script_graph`]; the lowerer validates that
/// placement before graph construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleSourceIr {
    key: ModuleKey,
    meta_url: String,
    pub(super) parse: ModuleParse,
    pub(super) prepared_script_import_sites: Vec<super::DynamicImportSiteIr>,
}

/// Record syntax is part of host identity and is never inferred from equal bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ModuleKindIr {
    SourceText,
    Json,
    Bytes,
    Text,
    ModuleSource,
    Script,
}

impl ModuleKindIr {
    pub const fn code(self) -> u8 {
        match self {
            Self::SourceText => 0,
            Self::Json => 1,
            Self::Script => 2,
            Self::Bytes => 3,
            Self::Text => 4,
            Self::ModuleSource => 5,
        }
    }
    pub fn requested(request: &ModuleRequestKeyIr) -> Self {
        match request.attributes() {
            [attribute] if attribute.key == "type" && attribute.value == "json" => Self::Json,
            [attribute] if attribute.key == "type" && attribute.value == "bytes" => Self::Bytes,
            [attribute] if attribute.key == "type" && attribute.value == "text" => Self::Text,
            _ => Self::SourceText,
        }
    }

    pub fn matches_request(self, request: &ModuleRequestKeyIr) -> bool {
        match self {
            Self::SourceText | Self::ModuleSource => {
                !request.attributes().iter().any(|attribute| {
                    attribute.key == "type"
                        && matches!(attribute.value.as_str(), "json" | "bytes" | "text")
                })
            }
            Self::Json => matches!(request.attributes(), [attribute]
                if attribute.key == "type" && attribute.value == "json"),
            Self::Bytes => matches!(request.attributes(), [attribute]
                if attribute.key == "type" && attribute.value == "bytes"),
            Self::Text => matches!(request.attributes(), [attribute]
                if attribute.key == "type" && attribute.value == "text"),
            Self::Script => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ModuleParse {
    Module(ParsedModule),
    Json(lila_front::ParsedJson),
    Bytes(Vec<u8>),
    Text(String),
    ModuleSource {
        source_tag: String,
    },
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
    /// Whether retained Script syntax has a call that could be direct eval.
    /// This whole-tree check only avoids impossible finite-source discovery;
    /// it proves neither intrinsic identity nor source or module authority.
    #[doc(hidden)]
    pub fn may_contain_direct_eval_call(&self) -> bool {
        match &self.parse {
            ModuleParse::ScriptEntry(source) => source
                .with_compiler_session(|ast, _| crate::analysis::contains_direct_eval_call(ast)),
            _ => false,
        }
    }

    /// Attach exact sites discovered in direct evals that retain this Script
    /// Record. This source alone supplies their host referrer; syntax heads in
    /// the original entry remain separate and are still checked one-for-one.
    #[doc(hidden)]
    pub fn include_prepared_script_import_sites(
        &mut self,
        sites: &[super::DynamicImportSiteIr],
    ) -> bool {
        assert!(
            matches!(self.parse, ModuleParse::ScriptEntry(_)),
            "a prepared Script-origin import cannot claim a Module referrer"
        );
        let mut changed = false;
        for site in sites
            .iter()
            .cloned()
            .map(super::DynamicImportSiteIr::as_prepared_script_site)
        {
            if !self.prepared_script_import_sites.contains(&site) {
                self.prepared_script_import_sites.push(site);
                changed = true;
            }
        }
        changed
    }

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
            | ModuleParse::Bytes(_)
            | ModuleParse::Text(_)
            | ModuleParse::ModuleSource { .. }
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
            prepared_script_import_sites: Vec::new(),
            parse,
        }
    }

    /// Retains raw data without parsing or decoding it as JavaScript or UTF-8.
    pub fn bytes(key: ModuleKey, bytes: Vec<u8>, meta_url: String) -> Self {
        Self {
            key,
            meta_url,
            prepared_script_import_sites: Vec::new(),
            parse: ModuleParse::Bytes(bytes),
        }
    }

    /// Retains the host-decoded String for CreateTextModule without a grammar parse.
    pub fn text(key: ModuleKey, text: String, meta_url: String) -> Self {
        Self {
            key,
            meta_url,
            prepared_script_import_sites: Vec::new(),
            parse: ModuleParse::Text(text),
        }
    }

    /// One opaque host ModuleSource record. No ECMAScript parse is performed.
    pub fn module_source(key: ModuleKey, source_tag: String, meta_url: String) -> Self {
        Self {
            key,
            meta_url,
            parse: ModuleParse::ModuleSource { source_tag },
            prepared_script_import_sites: Vec::new(),
        }
    }

    pub fn record_key(&self) -> super::ModuleRecordKey {
        super::ModuleRecordKey::new(self.key.clone(), self.kind())
    }

    /// Exact loaded payload used for host identity, including invalid UTF-8.
    pub fn payload_bytes(&self) -> &[u8] {
        match &self.parse {
            ModuleParse::Bytes(bytes) => bytes,
            ModuleParse::ModuleSource { source_tag } => source_tag.as_bytes(),
            ModuleParse::Module(_)
            | ModuleParse::Json(_)
            | ModuleParse::Text(_)
            | ModuleParse::JsonRejected { .. }
            | ModuleParse::ScriptEntry(_)
            | ModuleParse::Rejected { .. } => self.source_text().as_bytes(),
        }
    }

    pub const fn kind(&self) -> ModuleKindIr {
        match &self.parse {
            ModuleParse::Module(_) | ModuleParse::Rejected { .. } => ModuleKindIr::SourceText,
            ModuleParse::Json(_) | ModuleParse::JsonRejected { .. } => ModuleKindIr::Json,
            ModuleParse::ScriptEntry(_) => ModuleKindIr::Script,
            ModuleParse::Bytes(_) => ModuleKindIr::Bytes,
            ModuleParse::Text(_) => ModuleKindIr::Text,
            ModuleParse::ModuleSource { .. } => ModuleKindIr::ModuleSource,
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
                    encoding: lila_front::SourceTextEncoding::Utf8,
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
            prepared_script_import_sites: Vec::new(),
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
            prepared_script_import_sites: Vec::new(),
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
            prepared_script_import_sites: Vec::new(),
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
            ModuleParse::Bytes(_) | ModuleParse::ModuleSource { .. } => "",
            ModuleParse::Text(text) => text,
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
            ModuleParse::Json(_)
            | ModuleParse::Bytes(_)
            | ModuleParse::Text(_)
            | ModuleParse::ModuleSource { .. } => Some(Vec::new()),
            ModuleParse::ScriptEntry(source) => {
                let mut requests = scan_script_module_requests(source);
                for request in self
                    .prepared_script_import_sites
                    .iter()
                    .filter_map(super::DynamicImportSiteIr::discovery_request)
                {
                    if !requests.contains(request.key()) {
                        requests.push(request.key().clone());
                    }
                }
                Some(requests)
            }
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
            ModuleParse::Json(_)
            | ModuleParse::Bytes(_)
            | ModuleParse::Text(_)
            | ModuleParse::ModuleSource { .. } => Some(Vec::new()),
            ModuleParse::ScriptEntry(source) => Some(loading_requests_with_dynamic_imports(
                Vec::new(),
                &script_dynamic_import_sites(source)
                    .into_iter()
                    .chain(self.prepared_script_import_sites.iter().cloned())
                    .collect::<Vec<_>>(),
            )),
            ModuleParse::Rejected { .. } | ModuleParse::JsonRejected { .. } => None,
        }
    }

    /// Retained AST metadata for actual import call sites, including literal
    /// specifiers whose options or attributes require runtime evaluation.
    /// Synthetic records contain no sites. A rejected grammar returns None.
    #[must_use]
    pub fn dynamic_import_sites(&self) -> Option<Vec<super::DynamicImportSiteIr>> {
        match &self.parse {
            ModuleParse::Module(source) => Some(module_dynamic_import_sites(source)),
            ModuleParse::ScriptEntry(source) => Some(
                script_dynamic_import_sites(source)
                    .into_iter()
                    .chain(self.prepared_script_import_sites.iter().cloned())
                    .collect(),
            ),
            ModuleParse::Json(_)
            | ModuleParse::Bytes(_)
            | ModuleParse::Text(_)
            | ModuleParse::ModuleSource { .. } => Some(Vec::new()),
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
                    || !self.prepared_script_import_sites.is_empty()
                    || !self.realm_module_requests().is_empty(),
            ),
            ModuleParse::Module(_)
            | ModuleParse::Rejected { .. }
            | ModuleParse::Json(_)
            | ModuleParse::Bytes(_)
            | ModuleParse::Text(_)
            | ModuleParse::ModuleSource { .. }
            | ModuleParse::JsonRejected { .. } => None,
        }
    }

    #[must_use]
    pub fn goal(&self) -> ParseGoal {
        match &self.parse {
            ModuleParse::Module(_)
            | ModuleParse::Rejected { .. }
            | ModuleParse::Json(_)
            | ModuleParse::Bytes(_)
            | ModuleParse::Text(_)
            | ModuleParse::ModuleSource { .. }
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
