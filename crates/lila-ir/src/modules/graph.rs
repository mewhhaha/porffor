//! The module graph: loaded sources in, linked records out.
//!
//! Owns `GetExportedNames` (16.2.1.6.2), `ResolveExport` (16.2.1.6.3) and the
//! `InnerModuleEvaluation` DFS (16.2.1.5.3) that fixes evaluation order and
//! identifies strongly-connected components (cycles).
//!
//! Loading is *not* here. `lila-ir` performs no IO: the host resolves and
//! reads every source, then hands the closure over as [`ModuleGraphSources`].

use crate::*;

use super::record::{push_unique_name, ModuleEvaluationDependencyIr};

/// A stable module identity minted by the host's resolution boundary.
///
/// This is deliberately distinct from [`ModuleRequestIr::specifier`]: the
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
    parse: ModuleParse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ModuleParse {
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

/// Key a one-node graph uses when the caller supplied no filename.
///
/// Not a path, and never resolvable: a caller with no filename also has no
/// directory to resolve relative specifiers against.
pub const ANONYMOUS_MODULE_KEY: &str = "<entry>";

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

/// The `[[BindingName]]` half of a `ResolvedBinding` Record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleBindingNameIr {
    /// `namespace`, produced by `export * as ns from "m"`.
    Namespace,
    /// A concrete binding of the resolved module's environment.
    ///
    /// A `[[LocalName]]` **of the resolving module**, not of the module that
    /// asked: 16.2.1.6.2 step 4.a.i takes it from `e.[[LocalName]]` of whichever
    /// module the recursion ended in, which is why
    /// [`ResolvedBindingIr::Resolved`] carries the module alongside it.
    Name(LocalName),
    /// The module source object of the resolved module.
    ///
    /// Produced only by `import source x from "m"`. Not a binding of `m` at
    /// all: `m` is loaded and parsed but never instantiated, so there is no
    /// environment to name.
    ModuleSource,
}

/// Result of `ResolveExport` (16.2.1.6.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedBindingIr {
    /// A `ResolvedBinding` Record.
    Resolved {
        /// Module owning the binding.
        module: ModuleUnitId,
        /// `[[BindingName]]`.
        binding: ModuleBindingNameIr,
    },
    /// `ambiguous`: two `export *` paths reached different bindings.
    Ambiguous,
    /// `null`: no such export, or the request was circular.
    NotFound,
}

/// A linking failure. Every variant is a `SyntaxError` reported at compile
/// time, which is what test262's `phase: resolution` negatives expect — with
/// one exception this enum does not get to make: `DuplicateExport` names a
/// 16.2.3.1 *early* error that happens to have a producer here too, and
/// `rejection_kind` reports it at `phase: parse` from both producers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleLinkErrorIr {
    /// The host could not resolve a request to a module.
    UnresolvedModule {
        /// Module that made the request.
        referrer: ModuleUnitId,
        /// The unresolved request.
        request: ModuleRequestKeyIr,
    },
    /// The requested module does not export the imported name.
    MissingExport {
        /// Module that made the request.
        referrer: ModuleUnitId,
        /// The request.
        request: ModuleRequestIr,
        /// Name that could not be resolved.
        ///
        /// An `[[ImportName]]`, which is the requested module's
        /// `[[ExportName]]` read from this side — the same domain, so the same
        /// type. Filling this from a `[[LocalName]]` is `E0308`.
        import_name: ExportName,
    },
    /// Two `export *` paths reached different bindings for one name.
    AmbiguousExport {
        /// Module whose export is ambiguous.
        module: ModuleUnitId,
        /// The ambiguous export name.
        export_name: ExportName,
    },
    /// The same `[[ExportName]]` is declared twice (16.2.3.1, an early error).
    DuplicateExport {
        /// Module declaring the duplicate.
        module: ModuleUnitId,
        /// The duplicated export name.
        export_name: ExportName,
    },
    /// One key was loaded twice with different source text.
    InconsistentLoad {
        /// The key loaded inconsistently.
        key: ModuleKey,
    },
    /// Public host rows resolved one phase-free request key to two targets.
    InconsistentResolution {
        /// Module that made the request.
        referrer: ModuleUnitId,
        /// Request whose host resolution contradicted itself.
        request: ModuleRequestKeyIr,
    },
    /// The closure holds more units than the source-text linker can name.
    ///
    /// Unit ids are spelled into two in-place rewrites whose replacements must
    /// not change a unit's byte length, which caps the decimal width of an id at
    /// four digits — see [`MAX_LINKABLE_MODULE_UNIT_ID`]. This is the runtime
    /// half of budgets B1/B2; const assertions V2 and V4 carry the format half.
    TooManyUnits {
        /// Number of module sources the host handed over.
        count: usize,
    },
    /// A phased request this stage cannot link, with the reason.
    ///
    /// `import defer` and `import source` link (see
    /// [`ModuleEvaluationModeIr`]); what remains here are the shapes the
    /// source-text linker cannot express, chiefly a deferred module whose body
    /// would have to suspend.
    UnsupportedPhase {
        /// Module making the request.
        module: ModuleUnitId,
        /// The unsupported phase.
        phase: ImportPhaseIr,
        /// Why this particular request could not be linked.
        reason: String,
    },
}

impl ModuleLinkErrorIr {
    /// The condition this failure reports, in the one closed domain.
    ///
    /// `DuplicateExport` is deliberately not special-cased here: it names the
    /// same 16.2.3.1 condition `modules::early` names, and which *stage* that
    /// condition is reported at is `rejection_kind`'s decision, not this
    /// enum's. That is what makes it impossible for the two producers to
    /// disagree about its phase — they no longer each choose one.
    #[must_use]
    pub const fn code(&self) -> EarlyErrorCode {
        match self {
            Self::UnresolvedModule { .. } => EarlyErrorCode::ModuleUnresolved,
            Self::MissingExport { .. } => EarlyErrorCode::ModuleMissingExport,
            Self::AmbiguousExport { .. } => EarlyErrorCode::ModuleAmbiguousExport,
            Self::DuplicateExport { .. } => EarlyErrorCode::ModuleDuplicateExport,
            Self::InconsistentLoad { .. } => EarlyErrorCode::ModuleInconsistentLoad,
            Self::InconsistentResolution { .. } => EarlyErrorCode::ModuleInconsistentLoad,
            Self::UnsupportedPhase { .. } => EarlyErrorCode::ModuleUnsupportedPhase,
            Self::TooManyUnits { .. } => EarlyErrorCode::ModuleTooManyUnits,
        }
    }

    /// Human-readable message.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::UnresolvedModule { request, .. } => {
                format!("unresolved module request: {}", request.specifier())
            }
            Self::MissingExport {
                request,
                import_name,
                ..
            } => format!(
                "module {} does not export {}",
                request.specifier(),
                import_name.as_str()
            ),
            Self::AmbiguousExport { export_name, .. } => {
                format!("ambiguous export name: {}", export_name.as_str())
            }
            Self::DuplicateExport { export_name, .. } => {
                format!("duplicate export name: {}", export_name.as_str())
            }
            Self::InconsistentLoad { key } => {
                format!("module loaded inconsistently: {}", key.as_str())
            }
            Self::InconsistentResolution { request, .. } => format!(
                "module request resolved inconsistently: {}",
                request.specifier()
            ),
            Self::UnsupportedPhase { phase, reason, .. } => format!(
                "unsupported in lila wasm-aot: {} phase module request: {reason}",
                phase.as_str()
            ),
            Self::TooManyUnits { count } => format!(
                "unsupported in lila wasm-aot: module graph has {count} units; the source-text \
                 linker can name unit ids up to {MAX_LINKABLE_MODULE_UNIT_ID}"
            ),
        }
    }

    /// The diagnostic this error becomes on `ProgramIr`.
    ///
    /// The kind and phase are not chosen here. `IrDiagnostic::rejected` derives
    /// them from the code, so `DuplicateExport` lands on `EarlyError`/`Early`
    /// — 16.2.3.1 makes it an early error and
    /// `test/language/module-code/early-dup-export-id.js` is `phase: parse` —
    /// while the genuine link conditions land on `LinkError`/`Resolution`.
    #[must_use]
    pub fn to_diagnostic(&self) -> IrDiagnostic {
        IrDiagnostic::rejected(self.code(), self.message(), None)
    }
}

/// When, if ever, a unit's body runs in the merged script.
///
/// Fixed by [`classify_evaluation_modes`] from the *phases* of the requests
/// that reach a unit, and consumed by `modules::link` (which body text to
/// emit) and `modules::namespace` (which object to build).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleEvaluationModeIr {
    /// The body is emitted inline, in evaluation order. The default, and what
    /// every module of an unphased graph gets.
    #[default]
    Eager,
    /// `import defer`: the body is emitted as a thunk that the module's
    /// namespace object calls on the first read of any export.
    Deferred,
    /// `import source`: the module is loaded, parsed and linked, but its body
    /// is never emitted. Only a module source object is handed out.
    NotEvaluated,
}

/// How a linked unit participates in runtime source generation.
///
/// `NotEvaluated` deliberately has no inhabitant here: a source-phase-only
/// unit stays in the loaded and linked graph, but no runtime collector may
/// receive it. Keeping this type private prevents callers from manufacturing a
/// namespace or dispatcher for a unit whose body is absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModuleMaterializationModeIr {
    /// The unit's body is emitted inline.
    Eager,
    /// The unit's body is emitted as a deferred thunk.
    Deferred,
}

impl ModuleEvaluationModeIr {
    /// Diagnostic spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eager => "eager",
            Self::Deferred => "deferred",
            Self::NotEvaluated => "not evaluated",
        }
    }

    /// Runtime source-generation participation for this evaluation mode.
    ///
    /// This is the single exhaustive crossing from graph classification into
    /// artifact materialization. A new evaluation mode must decide here
    /// whether it contributes runtime state instead of inheriting a boolean
    /// default at one of the collectors.
    #[must_use]
    const fn materialization(self) -> Option<ModuleMaterializationModeIr> {
        match self {
            Self::Eager => Some(ModuleMaterializationModeIr::Eager),
            Self::Deferred => Some(ModuleMaterializationModeIr::Deferred),
            Self::NotEvaluated => None,
        }
    }
}

/// One module of the graph: its record, its source, and the lowered artifacts
/// the link stage produces from them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleUnitIr {
    /// Static entry tables for this module.
    pub record: SourceTextModuleRecordIr,
    /// The module source text, kept so the lowerer can slice spans from it.
    pub source_text: String,
    /// Value `import.meta.url` reports.
    pub meta_url: String,
    /// `InitializeEnvironment` for this unit. Filled by the link stage.
    pub hoist: Option<BlockIr>,
    /// `ExecuteModule` for this unit. Filled by the link stage.
    pub body: Option<BlockIr>,
    /// Functions lowered from this unit, with module-prefixed ids.
    pub functions: Vec<FunctionIr>,
    /// This unit's own top-level environment bindings.
    pub owned_env_bindings: Vec<OwnedEnvBindingIr>,
    /// Set when any importer or `import()` observes this module's namespace.
    pub namespace: Option<ModuleNamespaceIr>,
    /// One entry per `record.import_entries[i]`, same index.
    pub resolved_imports: Vec<ResolvedBindingIr>,
    /// One entry per `record.indirect_export_entries[i]`, same index.
    ///
    /// `ResolveExport` runs on indirect exports at link time (16.2.1.6.4 step
    /// 2), so `export { x } from "m"` where `m` has no `x` fails before
    /// anything evaluates. Keeping the results makes the re-export's target
    /// cell addressable without resolving a second time.
    pub resolved_indirect_exports: Vec<ResolvedBindingIr>,
}

/// A linked module graph.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModuleGraphIr {
    /// Index of the entry module.
    pub entry: ModuleUnitId,
    /// Every module. Index is the [`ModuleUnitId`].
    pub units: Vec<ModuleUnitIr>,
    /// Host-normalized key to unit index.
    pub keys: BTreeMap<ModuleKey, ModuleUnitId>,
    /// `(referrer, phase-free request key) -> target`, as resolved by the host.
    pub resolutions: BTreeMap<(ModuleUnitId, ModuleRequestKeyIr), ModuleUnitId>,
    /// Post-order DFS. Members of one strongly-connected component are
    /// contiguous.
    pub evaluation_order: Vec<ModuleUnitId>,
    /// Index into `evaluation_order` at which each component starts.
    pub scc_starts: Vec<usize>,
    /// Dynamic-import components compiled into this artifact.
    ///
    /// Discovery initially sees every call site in the loaded closure because
    /// those edges participate in evaluation-mode classification. After that
    /// fixed point, components whose referrer does not materialize are removed,
    /// so every row here names a call site that can run in this artifact.
    pub components: Vec<DynamicComponentIr>,
    /// When each unit's body runs, indexed by [`ModuleUnitId`].
    ///
    /// Filled by [`link`]; empty on a graph that has not been linked, which
    /// [`Self::evaluation_mode`] reads as the default [`Eager`].
    ///
    /// [`Eager`]: ModuleEvaluationModeIr::Eager
    pub evaluation_modes: Vec<ModuleEvaluationModeIr>,
    /// Every linking failure found. Non-empty means the graph does not run.
    pub link_errors: Vec<ModuleLinkErrorIr>,
    /// The entry unit is Script text, not module code.
    ///
    /// A Script has no imports and no exports, so it is not a module of the
    /// graph in any spec sense — but `import()` is legal in Script goal
    /// (13.3.10 takes `GetActiveScriptOrModule`, which a Script satisfies), and
    /// serving it needs exactly the same compiled targets a module's `import()`
    /// needs. So the loader assembles the closure of the Script's `import()`
    /// specifiers into an ordinary graph and marks the entry with this flag,
    /// which changes three things in `modules::link`:
    ///
    /// * the entry's text is emitted verbatim — no `"use strict"` prologue is
    ///   forced on it, no module syntax is stripped from it, and its top-level
    ///   `this` stays `globalThis`, because all three are Script semantics and
    ///   the entry really is a Script;
    /// * every *other* unit's material is wrapped in one immediately-invoked
    ///   strict function, so module code stays strict (16.2.1.6.1) and its
    ///   top-level bindings stay out of the Script's scope;
    /// * the entry's `import()` dispatchers are re-exported out of that wrapper
    ///   through `var` bindings the Script can call.
    pub entry_is_script: bool,
}

impl ModuleGraphIr {
    /// Borrows a unit by id.
    ///
    /// # Panics
    /// Panics if `id` is not a unit of this graph.
    #[must_use]
    pub fn unit(&self, id: ModuleUnitId) -> &ModuleUnitIr {
        &self.units[id as usize]
    }

    /// `true` when the entry module or anything it reaches uses top-level
    /// `await`, which forces the asynchronous evaluation driver.
    #[must_use]
    pub fn has_top_level_await(&self) -> bool {
        self.units
            .iter()
            .any(|unit| unit.record.has_top_level_await)
    }

    /// `[[HasTLA]]` (16.2.1.6.1 step 12) for one unit.
    ///
    /// `false` for an id this graph does not hold, which is the same answer a
    /// module with no `await` gives and never turns a graph asynchronous.
    #[must_use]
    pub fn has_tla(&self, module: ModuleUnitId) -> bool {
        self.units
            .get(module as usize)
            .is_some_and(|unit| unit.record.has_top_level_await)
    }

    /// Index of the strongly-connected component each unit belongs to, indexed
    /// by [`ModuleUnitId`], ordered as [`evaluation_components`] orders them.
    ///
    /// A unit that no root reached — which cannot happen for a graph built by
    /// [`build_graph`], every unit of which is a Tarjan root at worst — is left
    /// in a component of its own past the end, so the propagation below still
    /// terminates on a hand-built graph.
    ///
    /// [`evaluation_components`]: crate::modules::evaluation_components
    #[must_use]
    pub fn component_of_unit(&self) -> Vec<usize> {
        let mut components = vec![usize::MAX; self.units.len()];
        let mut next = 0;
        for (position, start) in self.scc_starts.iter().copied().enumerate() {
            let end = self
                .scc_starts
                .get(position + 1)
                .copied()
                .unwrap_or(self.evaluation_order.len());
            if start >= end {
                continue;
            }
            for member in &self.evaluation_order[start..end] {
                if let Some(slot) = components.get_mut(*member as usize) {
                    *slot = next;
                }
            }
            next += 1;
        }
        for slot in &mut components {
            if *slot == usize::MAX {
                *slot = next;
                next += 1;
            }
        }
        components
    }

    /// When `module`'s body runs. [`Eager`] for an id this graph does not hold
    /// and for a graph that has not been linked.
    ///
    /// [`Eager`]: ModuleEvaluationModeIr::Eager
    #[must_use]
    pub fn evaluation_mode(&self, module: ModuleUnitId) -> ModuleEvaluationModeIr {
        self.evaluation_modes
            .get(module as usize)
            .copied()
            .unwrap_or_default()
    }

    /// Runtime source-generation mode for one unit.
    ///
    /// `None` means either that `module` is not a unit of this graph or that it
    /// is source-phase-only. An unlinked graph follows [`Self::evaluation_mode`]
    /// and therefore retains its documented eager default.
    #[must_use]
    pub(super) fn materialization_mode(
        &self,
        module: ModuleUnitId,
    ) -> Option<ModuleMaterializationModeIr> {
        let index = usize::try_from(module).ok()?;
        self.units.get(index)?;
        self.evaluation_mode(module).materialization()
    }

    /// Units allowed to contribute bodies, aliases, objects or dispatchers to
    /// the emitted artifact.
    pub(super) fn materialized_units(
        &self,
    ) -> impl Iterator<Item = (ModuleUnitId, ModuleMaterializationModeIr, &ModuleUnitIr)> {
        self.units
            .iter()
            .enumerate()
            .filter_map(move |(index, unit)| {
                let id = ModuleUnitId::try_from(index).expect(
                    "unit index is capped by build_graph, which rejects a graph with more units than MAX_LINKABLE_MODULE_UNIT_ID",
                );
                self.materialization_mode(id)
                    .map(|mode| (id, mode, unit))
            })
    }

    /// Resolved evaluation dependencies of one unit, in
    /// `[[RequestedModules]]` order.
    ///
    /// Unresolved requests are dropped rather than reported: `link` has already
    /// recorded an [`ModuleLinkErrorIr::UnresolvedModule`] for each, and
    /// evaluation order is still wanted for the units that *did* resolve.
    /// Defer- and source-phase requests are resolved and linked, but they are
    /// not dependencies of `InnerModuleEvaluation`.
    fn evaluation_dependencies_of(
        &self,
        module: ModuleUnitId,
    ) -> Vec<ModuleEvaluationDependencyIr> {
        let Some(unit) = self.units.get(module as usize) else {
            return Vec::new();
        };
        unit.record
            .requested_modules
            .iter()
            .filter_map(|request| {
                let target = self.resolve_request(module, request)?;
                ModuleEvaluationDependencyIr::from_resolved_request(request, target)
            })
            .collect()
    }

    /// `[[AsyncEvaluation]]` (16.2.1.5.2 steps 11-14) for every unit, indexed
    /// by [`ModuleUnitId`].
    ///
    /// A module evaluates asynchronously when it has its own top-level `await`
    /// *or* when `InnerModuleEvaluation` gave it a non-zero
    /// [`pending_async_dependencies`]: step 11.b.i copies a dependency's
    /// `[[AsyncEvaluation]]` upwards, so asynchrony is transitive along
    /// dependency edges and never stops at the module that wrote the `await`.
    ///
    /// A cycle is one unit for this purpose. Every member of a strongly-
    /// connected component shares one `[[TopLevelCapability]]` in the spec and
    /// they resume as a group, so one member's `await` makes the whole
    /// component asynchronous.
    ///
    /// `evaluation_order` is a reverse-topological order over components, so a
    /// single forward pass reaches every dependency before its dependent and
    /// the propagation needs no fixed point.
    ///
    /// [`pending_async_dependencies`]: Self::pending_async_dependencies
    #[must_use]
    pub fn async_evaluation(&self) -> Vec<bool> {
        let components = self.component_of_unit();
        let component_count = components.iter().copied().max().map_or(0, |max| max + 1);
        let mut component_async = vec![false; component_count];

        for member in &self.evaluation_order {
            if self.evaluation_mode(*member) != ModuleEvaluationModeIr::Eager {
                continue;
            }
            let component = components[*member as usize];
            if self.has_tla(*member) {
                component_async[component] = true;
            }
            for dependency in self.evaluation_dependencies_of(*member) {
                let dependency = dependency.target();
                let dependency_component = components[dependency as usize];
                // Same component: the cycle is settled by the `has_tla` test
                // above over all of its members, and reading its own flag mid-
                // pass would depend on member order.
                if dependency_component != component && component_async[dependency_component] {
                    component_async[component] = true;
                }
            }
        }

        components
            .iter()
            .enumerate()
            .map(|(module, component)| {
                let module = ModuleUnitId::try_from(module).expect(
                    "unit index is capped by build_graph, which rejects a graph with more units than MAX_LINKABLE_MODULE_UNIT_ID",
                );
                self.evaluation_mode(module) == ModuleEvaluationModeIr::Eager
                    && component_async[*component]
            })
            .collect()
    }

    /// `[[PendingAsyncDependencies]]` (16.2.1.5.2 step 11.b.ii) for one unit:
    /// how many of its dependencies it must wait for before its own body may
    /// resume.
    ///
    /// Counted over distinct dependency *components*, and excluding the unit's
    /// own component: a cycle member is evaluated by the same
    /// `InnerModuleEvaluation` call and is never something to wait on, and two
    /// requests that resolve to one module are one dependency.
    #[must_use]
    pub fn pending_async_dependencies(&self, module: ModuleUnitId) -> usize {
        if self.evaluation_mode(module) != ModuleEvaluationModeIr::Eager {
            return 0;
        }
        let components = self.component_of_unit();
        let asynchronous = self.async_evaluation();
        let Some(&own) = components.get(module as usize) else {
            return 0;
        };
        let mut counted: BTreeSet<usize> = BTreeSet::new();
        for dependency in self.evaluation_dependencies_of(module) {
            let dependency = dependency.target();
            let component = components[dependency as usize];
            if component != own && asynchronous[dependency as usize] {
                counted.insert(component);
            }
        }
        counted.len()
    }

    /// Target of `request` made by `referrer`, if the host resolved it.
    ///
    /// There is deliberately no fallback lookup of `request.specifier` in
    /// [`Self::keys`]. A request spelling is relative to its referrer and is
    /// not a host-normalized [`ModuleKey`]; comparing the two domains used to
    /// make an omitted host resolution silently look resolved when their raw
    /// strings happened to match.
    #[must_use]
    pub fn resolve_request(
        &self,
        referrer: ModuleUnitId,
        request: &ModuleRequestIr,
    ) -> Option<ModuleUnitId> {
        self.resolve_request_key(referrer, request.key())
    }

    /// Target of a phase-free host request key made by `referrer`.
    #[must_use]
    pub fn resolve_request_key(
        &self,
        referrer: ModuleUnitId,
        request: &ModuleRequestKeyIr,
    ) -> Option<ModuleUnitId> {
        self.resolutions.get(&(referrer, request.clone())).copied()
    }

    /// `GetExportedNames` (16.2.1.6.2). Source order, as the spec defines it;
    /// namespace `[[OwnPropertyKeys]]` sorting happens in `namespace.rs`.
    #[must_use]
    pub fn exported_names(&self, module: ModuleUnitId) -> Vec<ExportName> {
        let mut export_star_set = BTreeSet::new();
        let mut names = Vec::new();
        self.collect_exported_names(module, &mut export_star_set, &mut names);
        names
    }

    fn collect_exported_names(
        &self,
        module: ModuleUnitId,
        export_star_set: &mut BTreeSet<ModuleUnitId>,
        names: &mut Vec<ExportName>,
    ) {
        // 1-2. A module already on the star path contributes nothing more.
        if !export_star_set.insert(module) {
            return;
        }
        let Some(unit) = self.units.get(module as usize) else {
            return;
        };
        for entry in &unit.record.local_export_entries {
            push_unique_name(names, &entry.export_name);
        }
        for entry in &unit.record.indirect_export_entries {
            push_unique_name(names, &entry.export_name);
        }
        for entry in &unit.record.star_export_entries {
            let Some(target) = self.resolve_request(module, &entry.request) else {
                continue;
            };
            let mut star_names = Vec::new();
            self.collect_exported_names(target, export_star_set, &mut star_names);
            for name in star_names {
                // `export *` never re-exports `default`.
                if !name.is_default() {
                    push_unique_name(names, &name);
                }
            }
        }
    }

    /// `ResolveExport` (16.2.1.6.3).
    ///
    /// The lookup key is a D2 name, always: the specification matches it against
    /// `e.[[ExportName]]`, and the `[[ImportName]]` an importer passes in is the
    /// requested module's D2 read from the other side. Passing a `[[LocalName]]`
    /// is `E0308`.
    #[must_use]
    pub fn resolve_export(
        &self,
        module: ModuleUnitId,
        export_name: &ExportName,
    ) -> ResolvedBindingIr {
        let mut resolve_set = Vec::new();
        self.resolve_export_inner(module, export_name, &mut resolve_set)
    }

    fn resolve_export_inner(
        &self,
        module: ModuleUnitId,
        export_name: &ExportName,
        resolve_set: &mut Vec<(ModuleUnitId, ExportName)>,
    ) -> ResolvedBindingIr {
        // 1. A repeated (module, exportName) pair is a circular request.
        if resolve_set
            .iter()
            .any(|(seen, name)| *seen == module && name == export_name)
        {
            return ResolvedBindingIr::NotFound;
        }
        resolve_set.push((module, export_name.clone()));

        let Some(unit) = self.units.get(module as usize) else {
            return ResolvedBindingIr::NotFound;
        };

        // 4. Local exports resolve to this module.
        // The match is D2 = D2 and the result is D1: the whole shape of
        // 16.2.1.6.2 step 4.a, and the one place the two domains meet.
        for entry in &unit.record.local_export_entries {
            if &entry.export_name == export_name {
                return ResolvedBindingIr::Resolved {
                    module,
                    binding: ModuleBindingNameIr::Name(entry.local_name.clone()),
                };
            }
        }

        // 5. Indirect exports delegate to the requested module.
        for entry in &unit.record.indirect_export_entries {
            if &entry.export_name != export_name {
                continue;
            }
            let Some(target) = self.resolve_request(module, &entry.request) else {
                return ResolvedBindingIr::NotFound;
            };
            return match &entry.import_name {
                ImportNameIr::Namespace => ResolvedBindingIr::Resolved {
                    module: target,
                    binding: ModuleBindingNameIr::Namespace,
                },
                ImportNameIr::Name(name) => self.resolve_export_inner(target, name, resolve_set),
            };
        }

        // 6. `default` is never provided by `export *`.
        if export_name.is_default() {
            return ResolvedBindingIr::NotFound;
        }

        // 7-8. Every star path must agree, otherwise the name is ambiguous.
        let mut star_resolution = ResolvedBindingIr::NotFound;
        for entry in &unit.record.star_export_entries {
            let Some(target) = self.resolve_request(module, &entry.request) else {
                continue;
            };
            let resolution = self.resolve_export_inner(target, export_name, resolve_set);
            if matches!(resolution, ResolvedBindingIr::Ambiguous) {
                return ResolvedBindingIr::Ambiguous;
            }
            if matches!(resolution, ResolvedBindingIr::Resolved { .. }) {
                if matches!(star_resolution, ResolvedBindingIr::NotFound) {
                    star_resolution = resolution;
                } else if star_resolution != resolution {
                    return ResolvedBindingIr::Ambiguous;
                }
            }
        }
        star_resolution
    }

    // `cell_name` used to live here. It was the sole producer of a *fourth*
    // name domain — `$m{unit}$` prefixed onto a `[[LocalName]]` — which a
    // per-unit-environment backend would need and which the source-text linker
    // in use today must never emit, because the merged scope names an
    // exporter's binding exactly as the exporter spells it. Its one caller
    // filled `ModuleNamespaceExportIr::cell`, whose only reader was a test.
    //
    // Nothing produces such a name now, so nothing can leak one into generated
    // Script text. `modules::namespace::namespace_target_reference` is the
    // single authority for what a resolved binding reads as, and it returns a
    // `MergedName`. If a per-unit-environment backend is ever built, the name
    // it needs is a *different* type from `MergedName` and must be spelled as
    // one — see `modules::namespace`'s module docs.
}

/// Builds the graph from an already-loaded closure: turns every retained parse
/// product into a module record and records the host's resolutions.
///
/// A key the graph already holds is reused and never rebuilt. That is module
/// map identity (16.2.1.7 `HostLoadImportedModule`): however many importers
/// name a module, and however many times the host hands its source over, the
/// graph holds exactly one unit for it — which is what makes a module evaluate
/// once and hand out one namespace object.
pub(crate) fn build_graph(
    sources: &ModuleGraphSources,
) -> Result<ModuleGraphIr, Vec<IrDiagnostic>> {
    let mut graph = ModuleGraphIr::default();
    let mut diagnostics = Vec::new();
    // Source position -> unit id. The two differ as soon as one key arrives
    // twice, and `resolutions` is stated in source positions.
    let mut remap: Vec<ModuleUnitId> = Vec::with_capacity(sources.modules.len());
    let mut inconsistent: Vec<ModuleKey> = Vec::new();

    for source in &sources.modules {
        if let Some(&existing) = graph.keys.get(source.key()) {
            // Same key, different bytes: the host contradicted itself, and
            // there is no honest way to pick a winner.
            if graph.units[existing as usize].source_text != source.source_text()
                && !inconsistent.iter().any(|key| key == source.key())
            {
                inconsistent.push(source.key().clone());
            }
            remap.push(existing);
            continue;
        }
        // The one place a unit id is minted, and therefore the one place the
        // byte budgets B1/B2 can be enforced at run time (contract ledger R3).
        // The previous `unwrap_or(ModuleUnitId::MAX)` was worse than unchecked:
        // it saturated to a ten-digit id, which violates both budgets silently
        // and then fails downstream as a confusing `StripError`.
        let Some(id) = ModuleUnitId::try_from(graph.units.len())
            .ok()
            .filter(|id| *id <= MAX_LINKABLE_MODULE_UNIT_ID)
        else {
            diagnostics.push(
                ModuleLinkErrorIr::TooManyUnits {
                    count: sources.modules.len(),
                }
                .to_diagnostic(),
            );
            return Err(diagnostics);
        };
        remap.push(id);
        let record = match &source.parse {
            ModuleParse::Module(unit_source) => {
                parse_module_record(unit_source, id, source.key().clone())
            }
            ModuleParse::ScriptEntry(unit_source) => Ok(super::record::script_entry_record(
                unit_source,
                id,
                source.key().clone(),
            )),
            ModuleParse::Rejected { error, .. } => {
                Err(vec![super::early::module_parse_failure_diagnostic(error)])
            }
        };
        match record {
            Ok(record) => {
                let resolved_imports =
                    vec![ResolvedBindingIr::NotFound; record.import_entries.len()];
                let resolved_indirect_exports =
                    vec![ResolvedBindingIr::NotFound; record.indirect_export_entries.len()];
                graph.keys.insert(source.key().clone(), id);
                graph.units.push(ModuleUnitIr {
                    record,
                    source_text: source.source_text().to_string(),
                    meta_url: source.meta_url().to_string(),
                    hoist: None,
                    body: None,
                    functions: Vec::new(),
                    owned_env_bindings: Vec::new(),
                    namespace: None,
                    resolved_imports,
                    resolved_indirect_exports,
                });
            }
            Err(mut parse_diagnostics) => diagnostics.append(&mut parse_diagnostics),
        }
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    graph.entry = remap
        .get(sources.entry as usize)
        .copied()
        .unwrap_or_default();
    let mut inconsistent_resolutions = BTreeSet::new();
    for (referrer, request, target) in &sources.resolutions {
        let (Some(&referrer), Some(&target)) =
            (remap.get(*referrer as usize), remap.get(*target as usize))
        else {
            continue;
        };
        let identity = (referrer, request.clone());
        if inconsistent_resolutions.contains(&identity) {
            continue;
        }
        match graph.resolutions.get(&identity).copied() {
            None => {
                graph.resolutions.insert(identity, target);
            }
            Some(existing) if existing == target => {}
            Some(_) => {
                // A phase-free request has exactly one host resolution. Keep
                // no winner: last-write-wins would make public row order part
                // of module identity.
                graph.resolutions.remove(&identity);
                inconsistent_resolutions.insert(identity);
            }
        }
    }
    for key in inconsistent {
        graph
            .link_errors
            .push(ModuleLinkErrorIr::InconsistentLoad { key });
    }
    for (referrer, request) in inconsistent_resolutions {
        graph
            .link_errors
            .push(ModuleLinkErrorIr::InconsistentResolution { referrer, request });
    }
    Ok(graph)
}

/// Resolves every import entry, records link errors, and computes the
/// evaluation order and its strongly-connected components.
pub(crate) fn link(graph: &mut ModuleGraphIr) {
    let unit_count = graph.units.len();

    for module in 0..unit_count {
        let id = ModuleUnitId::try_from(module).expect("unit index is capped by build_graph, which rejects a graph with more units than MAX_LINKABLE_MODULE_UNIT_ID");

        // 16.2.3.1: duplicate export names are an early error.
        for export_name in graph.units[module].record.duplicate_export_names() {
            graph.link_errors.push(ModuleLinkErrorIr::DuplicateExport {
                module: id,
                export_name,
            });
        }

        // Requests the host could not resolve.
        let requests = graph.units[module].record.requested_modules.clone();
        let mut checked = BTreeSet::new();
        for request in requests {
            let request = request.key().clone();
            if !checked.insert(request.clone()) {
                continue;
            }
            let inconsistent = graph.link_errors.iter().any(|error| {
                matches!(
                    error,
                    ModuleLinkErrorIr::InconsistentResolution {
                        referrer,
                        request: inconsistent_request,
                    } if *referrer == id && inconsistent_request == &request
                )
            });
            if graph.resolve_request_key(id, &request).is_none() && !inconsistent {
                graph.link_errors.push(ModuleLinkErrorIr::UnresolvedModule {
                    referrer: id,
                    request,
                });
            }
        }

        // ResolveExport for every import entry.
        let entries = graph.units[module].record.import_entries.clone();
        let mut resolved = Vec::with_capacity(entries.len());
        for entry in &entries {
            let Some(target) = graph.resolve_request(id, &entry.request) else {
                resolved.push(ResolvedBindingIr::NotFound);
                continue;
            };
            // A source-phase request never consults the requested module's
            // exports: it hands out a module source object, and the module is
            // not even instantiated. `[[ImportName]]` is `default` only because
            // the grammar reuses `ImportedBinding`.
            if entry.request.phase() == ImportPhaseIr::Source {
                resolved.push(ResolvedBindingIr::Resolved {
                    module: target,
                    binding: ModuleBindingNameIr::ModuleSource,
                });
                continue;
            }
            let binding = match &entry.import_name {
                ImportNameIr::Namespace => ResolvedBindingIr::Resolved {
                    module: target,
                    binding: ModuleBindingNameIr::Namespace,
                },
                ImportNameIr::Name(name) => graph.resolve_export(target, name),
            };
            match &binding {
                ResolvedBindingIr::Resolved { .. } => {}
                ResolvedBindingIr::Ambiguous => {
                    graph.link_errors.push(ModuleLinkErrorIr::AmbiguousExport {
                        module: target,
                        export_name: import_name_text(&entry.import_name),
                    });
                }
                ResolvedBindingIr::NotFound => {
                    graph.link_errors.push(ModuleLinkErrorIr::MissingExport {
                        referrer: id,
                        request: entry.request.clone(),
                        import_name: import_name_text(&entry.import_name),
                    });
                }
            }
            resolved.push(binding);
        }
        graph.units[module].resolved_imports = resolved;

        // 16.2.1.6.4 `InitializeEnvironment` step 2: every indirect export
        // resolves at link time, so `export { x } from "m"` where `m` has no
        // `x` is a SyntaxError before anything evaluates. Nothing imports the
        // re-export for this to be caught (`instn-iee-err-*`).
        let indirect = graph.units[module].record.indirect_export_entries.clone();
        let mut resolved_indirect = Vec::with_capacity(indirect.len());
        for entry in &indirect {
            if graph.resolve_request(id, &entry.request).is_none() {
                // Already reported as `UnresolvedModule` above; a missing
                // export on top of it would say the same thing twice.
                resolved_indirect.push(ResolvedBindingIr::NotFound);
                continue;
            }
            // Resolved through *this* module, as the spec states it: a name
            // that is both re-exported and reachable through `export *` is
            // ambiguous, and only the local view sees that.
            let resolution = graph.resolve_export(id, &entry.export_name);
            match &resolution {
                ResolvedBindingIr::Resolved { .. } => {}
                ResolvedBindingIr::Ambiguous => {
                    graph.link_errors.push(ModuleLinkErrorIr::AmbiguousExport {
                        module: id,
                        export_name: entry.export_name.clone(),
                    });
                }
                ResolvedBindingIr::NotFound => {
                    graph.link_errors.push(ModuleLinkErrorIr::MissingExport {
                        referrer: id,
                        request: entry.request.clone(),
                        import_name: import_name_text(&entry.import_name),
                    });
                }
            }
            resolved_indirect.push(resolution);
        }
        graph.units[module].resolved_indirect_exports = resolved_indirect;
    }

    compute_evaluation_order(graph);
    // Discovery must precede classification: a dynamic request carries a phase
    // too, so it can make its target eager, deferred or source-only. Runtime
    // components are fixed only after that reachability fixed point; a call
    // site in a source-only referrer is link metadata, not artifact code.
    let components = super::dynamic::discover_components(graph);
    classify_evaluation_modes(graph, &components);
    let components: Vec<_> = components
        .into_iter()
        .filter(|component| graph.materialization_mode(component.referrer).is_some())
        .collect();
    graph.components = components;
    report_unlinkable_phases(graph);
}

/// Fixes [`ModuleEvaluationModeIr`] for every unit from the phases of the
/// requests that reach it.
///
/// The rule is reachability, not a per-request vote: a module evaluates when
/// something that *itself* evaluates asks for it in the evaluation phase. So a
/// module reached only through `import source` never runs, and neither does
/// anything only that module imports — which is the whole point of the source
/// phase, and what a per-request vote would get wrong.
///
/// # Roots
///
/// The entry always evaluates. So does every evaluation-phase `import()`
/// target, because `import()` resolves with an *evaluated* namespace. So does
/// every unit no request points at: a graph assembled by an embedder rather than
/// by `load_module_graph` may hold units with no importer at all, and dropping
/// their bodies would silently change what such a graph runs.
///
/// A dynamic request counts exactly as much as its static twin does, phase for
/// phase: `import.defer('m')` defers `m` the way `import defer * as ns from
/// 'm'` does, and `import.source('m')` neither evaluates nor instantiates it.
///
/// # Deviation
///
/// The import-defer proposal defers a deferred module's whole dependency
/// subgraph. Here a deferred module's own evaluation-phase dependencies are
/// eager, because the merged scope binds an import to the *exporter's* cell and
/// a thunked exporter's cell is not in the merged scope at all. The deferred
/// module itself still evaluates only on first touch; what runs early is the
/// side effects of the modules it imports.
fn classify_evaluation_modes(graph: &mut ModuleGraphIr, components: &[DynamicComponentIr]) {
    let count = graph.units.len();
    // `(referrer, phase, target)` once, so the fixed point below is a walk over
    // an edge list rather than a repeated resolve of every request.
    let mut edges: Vec<(usize, ImportPhaseIr, usize)> = Vec::new();
    let mut targeted = vec![false; count];
    for module in 0..count {
        let id = ModuleUnitId::try_from(module).expect("unit index is capped by build_graph, which rejects a graph with more units than MAX_LINKABLE_MODULE_UNIT_ID");
        for request in &graph.units[module].record.requested_modules {
            let Some(target) = graph
                .resolve_request(id, request)
                .map(|target| target as usize)
                .filter(|target| *target < count)
            else {
                continue;
            };
            targeted[target] = true;
            edges.push((module, request.phase(), target));
        }
    }
    // `import()` call sites, which are not in `[[RequestedModules]]` but reach
    // a module just as surely. Their referrer is a unit of this graph, so a
    // dynamic edge out of a module nothing evaluates opens nothing — an
    // `import()` written in a source-phase-only module never runs.
    for component in components {
        let (Ok(referrer), Ok(target)) = (
            usize::try_from(component.referrer),
            usize::try_from(component.module),
        ) else {
            continue;
        };
        if referrer >= count || target >= count {
            continue;
        }
        targeted[target] = true;
        edges.push((referrer, component.request.phase(), target));
    }

    let mut eager = vec![false; count];
    let mut deferred = vec![false; count];
    for module in 0..count {
        if !targeted[module] || ModuleUnitId::try_from(module) == Ok(graph.entry) {
            eager[module] = true;
        }
    }

    // Fixed point rather than one pass: a unit only becomes deferred through an
    // edge from a unit that itself runs, and an edge can promote an already
    // deferred unit to eager, which then opens its own outgoing edges.
    loop {
        let mut changed = false;
        for (module, phase, target) in &edges {
            if !eager[*module] && !deferred[*module] {
                continue;
            }
            match phase {
                ImportPhaseIr::Evaluation => {
                    if !eager[*target] {
                        eager[*target] = true;
                        deferred[*target] = false;
                        changed = true;
                    }
                }
                ImportPhaseIr::Defer => {
                    if !eager[*target] && !deferred[*target] {
                        deferred[*target] = true;
                        changed = true;
                    }
                }
                // A source-phase request neither evaluates nor instantiates its
                // target, so it opens no edge at all.
                ImportPhaseIr::Source => {}
            }
        }
        if !changed {
            break;
        }
    }

    graph.evaluation_modes = (0..count)
        .map(|module| {
            if eager[module] {
                ModuleEvaluationModeIr::Eager
            } else if deferred[module] {
                ModuleEvaluationModeIr::Deferred
            } else {
                ModuleEvaluationModeIr::NotEvaluated
            }
        })
        .collect();
}

/// Reports the phased requests the source-text linker still cannot express.
///
/// Both remaining cases are about a deferred body becoming a *function* body:
/// a top-level `await` in it has nothing to suspend, and a cycle through it
/// would need every member of the component thunked together.
fn report_unlinkable_phases(graph: &mut ModuleGraphIr) {
    let components = graph.component_of_unit();
    let mut component_sizes = vec![0usize; components.iter().copied().max().map_or(0, |m| m + 1)];
    for component in &components {
        if let Some(size) = component_sizes.get_mut(*component) {
            *size += 1;
        }
    }

    let mut errors = Vec::new();
    for (module, mode) in graph.evaluation_modes.iter().copied().enumerate() {
        if mode != ModuleEvaluationModeIr::Deferred {
            continue;
        }
        let id = ModuleUnitId::try_from(module).expect("unit index is capped by build_graph, which rejects a graph with more units than MAX_LINKABLE_MODULE_UNIT_ID");
        if graph.has_tla(id) {
            errors.push(ModuleLinkErrorIr::UnsupportedPhase {
                module: id,
                phase: ImportPhaseIr::Defer,
                reason: format!(
                    "module {} has a top-level await, and a deferred body is a function body with nothing to suspend",
                    graph.units[module].record.key.as_str()
                ),
            });
        }
        if component_sizes
            .get(components[module])
            .is_some_and(|size| *size > 1)
        {
            errors.push(ModuleLinkErrorIr::UnsupportedPhase {
                module: id,
                phase: ImportPhaseIr::Defer,
                reason: format!(
                    "module {} is part of a cycle, and only whole components can be deferred together",
                    graph.units[module].record.key.as_str()
                ),
            });
        }
    }
    graph.link_errors.append(&mut errors);
}

/// `[[ImportName]]` as it reads in a diagnostic.
///
/// The `Namespace` arm has no `[[ExportName]]` behind it — `import * as ns`
/// resolves to a namespace object, not to an export — so `"*"` here is a
/// diagnostic spelling standing in for one.
///
/// It is **not** a name no module could have exported under: 16.2.3.1 makes
/// `ModuleExportName : StringLiteral` legal, so `export { x as "*" } from "m"`
/// spells exactly this, and invariant E1 relies on that openness. A
/// `MissingExport`/`AmbiguousExport` diagnostic naming `*` is therefore
/// ambiguous between a namespace import and a literal `"*"` export. Diagnostic
/// text only — no miscompilation — but the claim that it could not happen was
/// false and is not repeated here.
fn import_name_text(import_name: &ImportNameIr) -> ExportName {
    match import_name {
        ImportNameIr::Namespace => ExportName::new("*"),
        ImportNameIr::Name(name) => name.clone(),
    }
}

/// `InnerModuleEvaluation` (16.2.1.5.3) order, via Tarjan's SCC algorithm.
///
/// Tarjan emits components in reverse topological order, which is exactly the
/// order dependencies must evaluate in, and it groups the members of a cycle
/// contiguously so the link stage can hoist all of them before executing any.
fn compute_evaluation_order(graph: &mut ModuleGraphIr) {
    struct Tarjan {
        index: Vec<Option<usize>>,
        low: Vec<usize>,
        on_stack: Vec<bool>,
        stack: Vec<ModuleUnitId>,
        next_index: usize,
        /// DFS *finish* order, which is the order `InnerModuleEvaluation`
        /// reaches step 13 in. Distinct from `index`, the entry order.
        finish: Vec<usize>,
        next_finish: usize,
        order: Vec<ModuleUnitId>,
        starts: Vec<usize>,
    }

    let unit_count = graph.units.len();
    let mut state = Tarjan {
        index: vec![None; unit_count],
        low: vec![0; unit_count],
        on_stack: vec![false; unit_count],
        stack: Vec::new(),
        next_index: 0,
        finish: vec![0; unit_count],
        next_finish: 0,
        order: Vec::new(),
        starts: Vec::new(),
    };

    // Explicit work stack: module graphs can nest deeper than the native stack
    // tolerates, and `unsafe_code = "forbid"` leaves no room for a guard page
    // trick.
    enum Step {
        Enter(ModuleUnitId),
        Resume(ModuleUnitId, usize),
    }

    let dependencies: Vec<Vec<ModuleEvaluationDependencyIr>> = (0..unit_count)
        .map(|module| {
            let id = ModuleUnitId::try_from(module).expect("unit index is capped by build_graph, which rejects a graph with more units than MAX_LINKABLE_MODULE_UNIT_ID");
            graph.evaluation_dependencies_of(id)
        })
        .collect();

    for root in 0..unit_count {
        if state.index[root].is_some() {
            continue;
        }
        let root_id = ModuleUnitId::try_from(root).expect("unit index is capped by build_graph, which rejects a graph with more units than MAX_LINKABLE_MODULE_UNIT_ID");
        let mut work = vec![Step::Enter(root_id)];
        while let Some(step) = work.pop() {
            match step {
                Step::Enter(id) => {
                    let module = id as usize;
                    if state.index[module].is_some() {
                        continue;
                    }
                    state.index[module] = Some(state.next_index);
                    state.low[module] = state.next_index;
                    state.next_index += 1;
                    state.stack.push(id);
                    state.on_stack[module] = true;
                    work.push(Step::Resume(id, 0));
                }
                Step::Resume(id, cursor) => {
                    let module = id as usize;
                    if let Some(&dependency) = dependencies[module].get(cursor) {
                        work.push(Step::Resume(id, cursor + 1));
                        let next = dependency.target();
                        let target = next as usize;
                        match state.index[target] {
                            None => work.push(Step::Enter(next)),
                            Some(target_index) => {
                                if state.on_stack[target] {
                                    state.low[module] = state.low[module].min(target_index);
                                }
                            }
                        }
                        continue;
                    }
                    // All dependencies visited: fold their lowlinks in.
                    for &dependency in &dependencies[module] {
                        let next = dependency.target();
                        let target = next as usize;
                        if state.on_stack[target] {
                            state.low[module] = state.low[module].min(state.low[target]);
                        }
                    }
                    state.finish[module] = state.next_finish;
                    state.next_finish += 1;
                    if state.index[module] == Some(state.low[module]) {
                        state.starts.push(state.order.len());
                        let mut component = Vec::new();
                        while let Some(member) = state.stack.pop() {
                            state.on_stack[member as usize] = false;
                            component.push(member);
                            if member == id {
                                break;
                            }
                        }
                        // The stack pops a component in reverse *entry* order,
                        // which stops matching the spec as soon as a cycle
                        // branches. Finish order is what `InnerModuleEvaluation`
                        // executes bodies in, and the component root always
                        // finishes last, so it stays the component's last body.
                        component.sort_by_key(|member| state.finish[*member as usize]);
                        state.order.extend(component);
                    }
                }
            }
        }
    }

    graph.evaluation_order = state.order;
    graph.scc_starts = state.starts;
}

#[cfg(test)]
mod tests {
    use super::*;

    // The single authority for what a resolved binding reads as in the merged
    // scope, now that this file mints no cell name of its own.
    use super::super::namespace::namespace_target_reference;

    /// Last path segment, which is what the test loader resolves on.
    fn last_segment(text: &str) -> &str {
        text.rsplit('/').next().unwrap_or(text)
    }

    /// Stands in for a host loader: a specifier resolves to the file whose key
    /// ends in the same segment, so `"./a.js"`, `"../dir/a.js"` and `"a.js"`
    /// all normalize to the one key `"/root/a.js"` — which is the shape the
    /// module map has to collapse. `files[0]` is the entry.
    fn sources_of(files: &[(&str, &str)]) -> ModuleGraphSources {
        let modules: Vec<ModuleSourceIr> = files
            .iter()
            .map(|(key, source_text)| {
                ModuleSourceIr::new(
                    ModuleKey::from_host(*key),
                    (*source_text).to_string(),
                    format!("file://{key}"),
                )
            })
            .collect();
        let mut resolutions = Vec::new();
        for (index, _) in files.iter().enumerate() {
            let requests = modules[index]
                .module_requests()
                .expect("test module parses");
            let referrer = u32::try_from(index).expect("test graph is small");
            for request in requests {
                let target = files
                    .iter()
                    .position(|(key, _)| last_segment(key) == last_segment(request.specifier()));
                if let Some(target) = target {
                    let target = u32::try_from(target).expect("test graph is small");
                    resolutions.push((referrer, request, target));
                }
            }
        }
        ModuleGraphSources {
            modules,
            entry: 0,
            resolutions,
        }
    }

    fn linked(files: &[(&str, &str)]) -> ModuleGraphIr {
        let mut graph = build_graph(&sources_of(files)).expect("test modules parse");
        link(&mut graph);
        graph
    }

    fn unit_of(graph: &ModuleGraphIr, key: &str) -> ModuleUnitId {
        *graph
            .keys
            .get(&ModuleKey::from_host(key))
            .expect("key is in the graph")
    }

    fn components(graph: &ModuleGraphIr) -> Vec<Vec<ModuleUnitId>> {
        crate::evaluation_components(graph)
    }

    #[test]
    fn rejected_delete_reference_dependencies_keep_typed_diagnostics_through_graph_build() {
        for (index, source_text, expected) in [
            (
                0,
                "export const x = 0; delete x;",
                EarlyErrorCode::StrictModeDeleteIdentifierReference,
            ),
            (
                1,
                "export class C { #x; m(o) { delete o.#x; } }",
                EarlyErrorCode::StrictModeDeletePrivateReference,
            ),
        ] {
            let dependency_key = format!("/root/delete-{index}.js");
            let dependency = ModuleSourceIr::new(
                ModuleKey::from_host(dependency_key.clone()),
                source_text.to_string(),
                format!("file://{dependency_key}"),
            );
            assert_eq!(
                dependency.module_requests(),
                None,
                "the rejected parse must be retained rather than rescanned"
            );

            let diagnostics = build_graph(&ModuleGraphSources {
                modules: vec![
                    ModuleSourceIr::new(
                        ModuleKey::from_host("/root/entry.js"),
                        format!("import './delete-{index}.js';"),
                        "file:///root/entry.js".to_string(),
                    ),
                    dependency,
                ],
                entry: 0,
                resolutions: vec![(
                    0,
                    ModuleRequestKeyIr::plain(format!("./delete-{index}.js")),
                    1,
                )],
            })
            .expect_err("the retained rejected dependency must stop graph construction");
            let [diagnostic] = diagnostics.as_slice() else {
                panic!("expected one retained parse diagnostic, got {diagnostics:?}");
            };

            assert_eq!(
                diagnostic.kind,
                IrDiagnosticKind::EarlyError,
                "{source_text:?}"
            );
            assert_eq!(
                diagnostic.phase(),
                IrDiagnosticPhase::Early,
                "{source_text:?}"
            );
            assert_eq!(diagnostic.code(), Some(expected), "{source_text:?}");
            assert_eq!(
                diagnostic.error_type(),
                Some(NativeErrorKind::SyntaxError),
                "{source_text:?}"
            );
            assert!(diagnostic.span.is_some(), "{source_text:?}: {diagnostic:?}");
        }
    }

    #[test]
    fn rejected_optional_chain_tagged_template_dependency_keeps_typed_diagnostic_through_graph_build(
    ) {
        let dependency_source = "export const value = null; value?.tag`x${1}`;";
        let dependency = ModuleSourceIr::new(
            ModuleKey::from_host("/root/optional-template.js"),
            dependency_source.to_string(),
            "file:///root/optional-template.js".to_string(),
        );
        assert_eq!(
            dependency.module_requests(),
            None,
            "the rejected parse must be retained rather than rescanned"
        );

        let diagnostics = build_graph(&ModuleGraphSources {
            modules: vec![
                ModuleSourceIr::new(
                    ModuleKey::from_host("/root/entry.js"),
                    "import './optional-template.js';".to_string(),
                    "file:///root/entry.js".to_string(),
                ),
                dependency,
            ],
            entry: 0,
            resolutions: vec![(0, ModuleRequestKeyIr::plain("./optional-template.js"), 1)],
        })
        .expect_err("the retained rejected dependency must stop graph construction");
        let [diagnostic] = diagnostics.as_slice() else {
            panic!("expected one retained parse diagnostic, got {diagnostics:?}");
        };

        assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early);
        assert_eq!(
            diagnostic.code(),
            Some(EarlyErrorCode::OptionalChainTaggedTemplate)
        );
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
        let span = diagnostic
            .span
            .expect("the retained TemplateLiteral must keep its source span");
        assert!(
            span.start < span.end,
            "{dependency_source:?}: {diagnostic:?}"
        );
    }

    #[test]
    fn rejected_for_head_body_declaration_conflict_dependency_keeps_typed_diagnostic_through_graph_build(
    ) {
        let dependency_source = "for (let x of []) { var x; }";
        let dependency = ModuleSourceIr::new(
            ModuleKey::from_host("/root/for-head-body-conflict.js"),
            dependency_source.to_string(),
            "file:///root/for-head-body-conflict.js".to_string(),
        );
        assert_eq!(
            dependency.module_requests(),
            None,
            "the rejected parse must be retained rather than rescanned"
        );

        let diagnostics = build_graph(&ModuleGraphSources {
            modules: vec![
                ModuleSourceIr::new(
                    ModuleKey::from_host("/root/entry.js"),
                    "import './for-head-body-conflict.js';".to_string(),
                    "file:///root/entry.js".to_string(),
                ),
                dependency,
            ],
            entry: 0,
            resolutions: vec![(
                0,
                ModuleRequestKeyIr::plain("./for-head-body-conflict.js"),
                1,
            )],
        })
        .expect_err("the retained rejected dependency must stop graph construction");
        let [diagnostic] = diagnostics.as_slice() else {
            panic!("expected one retained parse diagnostic, got {diagnostics:?}");
        };

        assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early);
        assert_eq!(
            diagnostic.code(),
            Some(EarlyErrorCode::ForHeadBodyDeclarationConflict)
        );
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
        let span = diagnostic
            .span
            .expect("the retained loop conflict must keep its source span");
        assert!(
            span.start < span.end,
            "{dependency_source:?}: {diagnostic:?}"
        );
    }

    #[test]
    fn rejected_for_declaration_duplicate_bound_name_dependency_keeps_typed_diagnostic_through_graph_build(
    ) {
        let dependency_source = "for (let [x, x] of []) {}";
        let dependency = ModuleSourceIr::new(
            ModuleKey::from_host("/root/for-declaration-duplicate.js"),
            dependency_source.to_string(),
            "file:///root/for-declaration-duplicate.js".to_string(),
        );
        assert_eq!(
            dependency.module_requests(),
            None,
            "the rejected parse must be retained rather than rescanned"
        );

        let diagnostics = build_graph(&ModuleGraphSources {
            modules: vec![
                ModuleSourceIr::new(
                    ModuleKey::from_host("/root/entry.js"),
                    "import './for-declaration-duplicate.js';".to_string(),
                    "file:///root/entry.js".to_string(),
                ),
                dependency,
            ],
            entry: 0,
            resolutions: vec![(
                0,
                ModuleRequestKeyIr::plain("./for-declaration-duplicate.js"),
                1,
            )],
        })
        .expect_err("the retained rejected dependency must stop graph construction");
        let [diagnostic] = diagnostics.as_slice() else {
            panic!("expected one retained parse diagnostic, got {diagnostics:?}");
        };

        assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early);
        assert_eq!(
            diagnostic.code(),
            Some(EarlyErrorCode::ForDeclarationDuplicateBoundName)
        );
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
        let span = diagnostic
            .span
            .expect("the retained duplicate loop binding must keep its source span");
        assert!(
            span.start < span.end,
            "{dependency_source:?}: {diagnostic:?}"
        );
    }

    #[test]
    fn rejected_lexical_bound_name_let_dependency_keeps_typed_diagnostic_through_graph_build() {
        let dependency_source = "for (const { value: let } of []) {}";
        let dependency = ModuleSourceIr::new(
            ModuleKey::from_host("/root/lexical-bound-name-let.js"),
            dependency_source.to_string(),
            "file:///root/lexical-bound-name-let.js".to_string(),
        );
        assert_eq!(
            dependency.module_requests(),
            None,
            "the rejected Module parse must be retained rather than rescanned"
        );

        let diagnostics = build_graph(&ModuleGraphSources {
            modules: vec![
                ModuleSourceIr::new(
                    ModuleKey::from_host("/root/entry.js"),
                    "import './lexical-bound-name-let.js';".to_string(),
                    "file:///root/entry.js".to_string(),
                ),
                dependency,
            ],
            entry: 0,
            resolutions: vec![(
                0,
                ModuleRequestKeyIr::plain("./lexical-bound-name-let.js"),
                1,
            )],
        })
        .expect_err("the retained rejected dependency must stop graph construction");
        let [diagnostic] = diagnostics.as_slice() else {
            panic!("expected one retained parse diagnostic, got {diagnostics:?}");
        };

        assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early);
        assert_eq!(diagnostic.code(), Some(EarlyErrorCode::LexicalBoundNameLet));
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
        let span = diagnostic
            .span
            .expect("the retained lexical binding must keep its source span");
        assert!(
            span.start < span.end,
            "{dependency_source:?}: {diagnostic:?}"
        );
    }

    #[test]
    fn rejected_top_level_super_dependency_keeps_its_module_code_through_graph_build() {
        let dependency_source = "() => super.value;";
        let dependency = ModuleSourceIr::new(
            ModuleKey::from_host("/root/top-level-super.js"),
            dependency_source.to_string(),
            "file:///root/top-level-super.js".to_string(),
        );
        assert_eq!(dependency.goal(), ParseGoal::Module);
        assert_eq!(
            dependency.module_requests(),
            None,
            "the rejected Module parse must be retained rather than rescanned"
        );

        let diagnostics = build_graph(&ModuleGraphSources {
            modules: vec![
                ModuleSourceIr::new(
                    ModuleKey::from_host("/root/entry.js"),
                    "import './top-level-super.js';".to_string(),
                    "file:///root/entry.js".to_string(),
                ),
                dependency,
            ],
            entry: 0,
            resolutions: vec![(0, ModuleRequestKeyIr::plain("./top-level-super.js"), 1)],
        })
        .expect_err("the retained rejected dependency must stop graph construction");
        let [diagnostic] = diagnostics.as_slice() else {
            panic!("expected one retained parse diagnostic, got {diagnostics:?}");
        };

        assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError);
        assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early);
        assert_eq!(diagnostic.code(), Some(EarlyErrorCode::ModuleTopLevelSuper));
        assert_eq!(diagnostic.error_type(), Some(NativeErrorKind::SyntaxError));
        let span = diagnostic
            .span
            .expect("the retained Module failure must keep its source span");
        assert!(
            span.start < span.end,
            "{dependency_source:?}: {diagnostic:?}"
        );
    }

    #[test]
    fn rejected_class_owned_super_call_dependencies_keep_distinct_codes_through_graph_build() {
        for (index, source_text, expected) in [
            (
                0,
                "export default class { constructor() { super(); } }",
                EarlyErrorCode::ClassBaseConstructorHasDirectSuper,
            ),
            (
                1,
                "export default class { static { super(); } }",
                EarlyErrorCode::ClassStaticBlockContainsSuperCall,
            ),
        ] {
            let dependency_key = format!("/root/class-super-{index}.js");
            let dependency = ModuleSourceIr::new(
                ModuleKey::from_host(dependency_key.clone()),
                source_text.to_string(),
                format!("file://{dependency_key}"),
            );
            assert_eq!(dependency.goal(), ParseGoal::Module);
            let ModuleParse::Rejected { error, .. } = &dependency.parse else {
                panic!("the class-owned early error must be retained as a rejected parse");
            };
            assert_eq!(
                crate::modules::early::module_parse_failure_diagnostic(error).code(),
                Some(expected),
                "{source_text:?}"
            );
            assert_eq!(
                dependency.module_requests(),
                None,
                "a rejected dependency must not be rescanned for requests"
            );

            let diagnostics = build_graph(&ModuleGraphSources {
                modules: vec![
                    ModuleSourceIr::new(
                        ModuleKey::from_host("/root/entry.js"),
                        format!("import './class-super-{index}.js';"),
                        "file:///root/entry.js".to_string(),
                    ),
                    dependency,
                ],
                entry: 0,
                resolutions: vec![(
                    0,
                    ModuleRequestKeyIr::plain(format!("./class-super-{index}.js")),
                    1,
                )],
            })
            .expect_err("the retained rejected dependency must stop graph construction");
            let [diagnostic] = diagnostics.as_slice() else {
                panic!("expected one retained parse diagnostic, got {diagnostics:?}");
            };

            assert_eq!(diagnostic.kind, IrDiagnosticKind::EarlyError, "{source_text:?}");
            assert_eq!(diagnostic.phase(), IrDiagnosticPhase::Early, "{source_text:?}");
            assert_eq!(diagnostic.code(), Some(expected), "{source_text:?}");
            assert_eq!(
                diagnostic.error_type(),
                Some(NativeErrorKind::SyntaxError),
                "{source_text:?}"
            );
            let span = diagnostic
                .span
                .expect("the retained class rejection must keep its source span");
            assert!(span.start < span.end, "{source_text:?}: {diagnostic:?}");
        }
    }

    #[test]
    fn retained_class_owned_super_dependency_builds_a_real_module_graph() {
        let dependency_source = concat!(
            "class Base {};\n",
            "export class Derived extends Base {\n",
            "  constructor() { super(); }\n",
            "  method() { return () => super.value; }\n",
            "  static { void super.value; }\n",
            "}",
        );
        let dependency = ModuleSourceIr::new(
            ModuleKey::from_host("/root/class-owned-super.js"),
            dependency_source.to_string(),
            "file:///root/class-owned-super.js".to_string(),
        );
        assert_eq!(dependency.goal(), ParseGoal::Module);
        assert_eq!(
            dependency.module_requests(),
            Some(Vec::new()),
            "valid class-owned super must remain a successfully parsed Module"
        );

        let graph = build_graph(&ModuleGraphSources {
            modules: vec![
                ModuleSourceIr::new(
                    ModuleKey::from_host("/root/entry.js"),
                    "import './class-owned-super.js';".to_string(),
                    "file:///root/entry.js".to_string(),
                ),
                dependency,
            ],
            entry: 0,
            resolutions: vec![(0, ModuleRequestKeyIr::plain("./class-owned-super.js"), 1)],
        })
        .expect("valid class-owned super must build a Module graph");
        assert_eq!(graph.units.len(), 2);
    }

    #[test]
    fn retained_import_meta_dependency_keeps_its_module_goal_through_graph_build() {
        let dependency_source = concat!(
            "export const direct = import.meta;\n",
            "export function nested() { return import.meta; }",
        );
        let dependency = ModuleSourceIr::new(
            ModuleKey::from_host("/root/import-meta.js"),
            dependency_source.to_string(),
            "file:///root/import-meta.js".to_string(),
        );
        assert_eq!(dependency.goal(), ParseGoal::Module);
        assert_eq!(
            dependency.module_requests(),
            Some(Vec::new()),
            "direct and nested ImportMeta must remain a successfully parsed Module"
        );

        let graph = build_graph(&ModuleGraphSources {
            modules: vec![
                ModuleSourceIr::new(
                    ModuleKey::from_host("/root/entry.js"),
                    "import './import-meta.js';".to_string(),
                    "file:///root/entry.js".to_string(),
                ),
                dependency,
            ],
            entry: 0,
            resolutions: vec![(0, ModuleRequestKeyIr::plain("./import-meta.js"), 1)],
        })
        .expect("a Module dependency containing ImportMeta must build without a parse rejection");

        let dependency = &graph.units[unit_of(&graph, "/root/import-meta.js") as usize];
        assert_eq!(dependency.source_text, dependency_source);
        let import_meta_text: Vec<&str> = dependency
            .record
            .import_meta_sites
            .iter()
            .map(|site| &dependency.source_text[site.start..site.end])
            .collect();
        assert_eq!(import_meta_text, vec!["import.meta", "import.meta"]);
    }

    /// The default: everything the entry reaches through an ordinary `import`
    /// evaluates inline, which is what an unphased graph has always done.
    #[test]
    fn an_unphased_graph_evaluates_every_unit_eagerly() {
        let graph = linked(&[
            ("/root/entry.js", "import { x } from './a.js';\nx;"),
            ("/root/a.js", "export const x = 1;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        assert_eq!(
            graph.evaluation_modes,
            vec![ModuleEvaluationModeIr::Eager, ModuleEvaluationModeIr::Eager]
        );
    }

    /// `import defer` is the only edge reaching the dependency, so it is linked
    /// but its body waits for the first touch of its namespace.
    #[test]
    fn a_defer_only_dependency_is_deferred() {
        let graph = linked(&[
            ("/root/entry.js", "import defer * as ns from './a.js';\nns;"),
            ("/root/a.js", "export const x = 1;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        assert_eq!(
            graph.evaluation_mode(unit_of(&graph, "/root/a.js")),
            ModuleEvaluationModeIr::Deferred
        );
    }

    /// An evaluation-phase importer wins over a deferred one: a module that
    /// something evaluates has already run by the time the deferred namespace
    /// is touched, and `import defer` of it is then indistinguishable from
    /// `import *`.
    #[test]
    fn a_module_also_imported_eagerly_is_not_deferred() {
        let graph = linked(&[
            (
                "/root/entry.js",
                "import defer * as ns from './a.js';\nimport { x } from './a.js';\nns; x;",
            ),
            ("/root/a.js", "export const x = 1;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        assert_eq!(
            graph.evaluation_mode(unit_of(&graph, "/root/a.js")),
            ModuleEvaluationModeIr::Eager
        );
    }

    /// `import source` neither evaluates nor instantiates its target — and
    /// therefore does not evaluate what that target imports either, which a
    /// per-request vote over incoming phases would get wrong.
    #[test]
    fn a_source_only_module_and_its_own_dependency_never_evaluate() {
        let graph = linked(&[
            ("/root/entry.js", "import source src from './a.js';\nsrc;"),
            ("/root/a.js", "import './b.js';\nexport const x = 1;"),
            ("/root/b.js", "globalThis.ran = true;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        assert_eq!(
            graph.evaluation_mode(unit_of(&graph, "/root/a.js")),
            ModuleEvaluationModeIr::NotEvaluated
        );
        assert_eq!(
            graph.evaluation_mode(unit_of(&graph, "/root/b.js")),
            ModuleEvaluationModeIr::NotEvaluated
        );
    }

    /// Loading/linking edges are not evaluation edges. In particular, a defer
    /// edge and a source edge pointing back at the importer do not make the two
    /// modules an `InnerModuleEvaluation` cycle.
    #[test]
    fn non_evaluation_phase_edges_do_not_form_an_evaluation_cycle() {
        let graph = linked(&[
            ("/root/entry.js", "import defer * as ns from './a.js';\nns;"),
            (
                "/root/a.js",
                "import source entry from './entry.js';\nexport const x = 1;\nentry;",
            ),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        assert_eq!(
            graph.evaluation_mode(unit_of(&graph, "/root/a.js")),
            ModuleEvaluationModeIr::Deferred
        );
        let components = components(&graph);
        assert_eq!(components.len(), 2, "{components:?}");
        assert!(
            components.iter().all(|component| component.len() == 1),
            "{components:?}"
        );
    }

    /// A source-phase target is never evaluated, so even a raw `[[HasTLA]]`
    /// record inside it cannot turn the linked graph asynchronous.
    #[test]
    fn non_evaluation_phase_tla_does_not_make_graph_async() {
        let graph = linked(&[
            ("/root/entry.js", "import source src from './a.js';\nsrc;"),
            ("/root/a.js", "export const x = await 1;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        let target = unit_of(&graph, "/root/a.js");
        assert_eq!(
            graph.evaluation_mode(target),
            ModuleEvaluationModeIr::NotEvaluated
        );
        assert!(graph.has_tla(target));
        assert!(
            graph
                .async_evaluation()
                .iter()
                .all(|asynchronous| !asynchronous),
            "source-only targets do not participate in AsyncModuleExecution"
        );
        assert_eq!(graph.pending_async_dependencies(graph.entry), 0);
        assert_eq!(graph.pending_async_dependencies(target), 0);
    }

    /// A source-phase request resolves to a module source object rather than to
    /// the `default` export its `ImportedBinding` grammar would otherwise name.
    #[test]
    fn a_source_phase_import_resolves_to_a_module_source() {
        let graph = linked(&[
            ("/root/entry.js", "import source src from './a.js';\nsrc;"),
            ("/root/a.js", "export const x = 1;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        let target = unit_of(&graph, "/root/a.js");
        assert_eq!(
            graph.units[0].resolved_imports,
            vec![ResolvedBindingIr::Resolved {
                module: target,
                binding: ModuleBindingNameIr::ModuleSource,
            }]
        );
        assert_eq!(
            namespace_target_reference(&graph.units[0].resolved_imports[0]),
            Some(MergedName::minted(target, UnitCellRole::ModuleSource))
        );
    }

    /// A deferred body becomes a function body, and a top-level `await` in a
    /// function body has nothing to suspend. Reported rather than mislinked.
    #[test]
    fn deferring_a_top_level_await_module_is_reported() {
        let graph = linked(&[
            ("/root/entry.js", "import defer * as ns from './a.js';\nns;"),
            ("/root/a.js", "export const x = await 1;"),
        ]);
        assert!(
            graph.link_errors.iter().any(|error| matches!(
                error,
                ModuleLinkErrorIr::UnsupportedPhase {
                    phase: ImportPhaseIr::Defer,
                    ..
                }
            )),
            "{:?}",
            graph.link_errors
        );
    }

    #[test]
    fn one_key_reached_through_several_specifiers_is_one_unit() {
        let graph = linked(&[
            (
                "/root/entry.js",
                "import { x } from './shared.js';\nimport * as ns from '../root/shared.js';\nx; ns;",
            ),
            ("/root/mid.js", "export { x } from 'shared.js';"),
            ("/root/shared.js", "export const x = 1;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        assert_eq!(graph.units.len(), 3);
        assert_eq!(graph.keys.len(), 3);
        // Three different specifiers, one unit behind all of them.
        let shared = unit_of(&graph, "/root/shared.js");
        assert_eq!(graph.resolutions.len(), 3);
        assert!(graph.resolutions.values().all(|target| *target == shared));
    }

    /// `ModuleGraphSources::resolutions` is a public embedder boundary. Its
    /// request may be constructed independently of the retained parse, so an
    /// opposite input order must still name the same ModuleRequest Record.
    #[test]
    fn a_public_host_resolution_row_matches_canonical_request_attributes() {
        fn attribute(key: &str, value: &str) -> ImportAttributeIr {
            ImportAttributeIr {
                key: key.to_string(),
                value: value.to_string(),
            }
        }

        let modules = vec![
            ModuleSourceIr::new(
                ModuleKey::from_host("/root/entry.js"),
                "import { value } from './dep.js' with { charset: 'utf8', type: 'text' };\n\
                 value;"
                    .to_string(),
                "file:///root/entry.js".to_string(),
            ),
            ModuleSourceIr::new(
                ModuleKey::from_host("/root/dep.js"),
                "export const value = 41;".to_string(),
                "file:///root/dep.js".to_string(),
            ),
        ];
        let host_request = ModuleRequestKeyIr::try_new(
            "./dep.js",
            // Reverse of the source's canonical order.
            vec![attribute("type", "text"), attribute("charset", "utf8")],
        )
        .expect("host attributes are unique");
        let sources = ModuleGraphSources {
            modules,
            entry: 0,
            resolutions: vec![(0, host_request, 1)],
        };

        let mut graph = build_graph(&sources).expect("test modules parse");
        link(&mut graph);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        assert_eq!(
            graph.units[0].resolved_imports,
            vec![ResolvedBindingIr::Resolved {
                module: 1,
                binding: ModuleBindingNameIr::Name(LocalName::from_bound_name("value")),
            }]
        );
    }

    #[test]
    fn eval_defer_and_source_occurrences_share_one_resolution_key() {
        let sources = ModuleGraphSources {
            modules: vec![
                ModuleSourceIr::new(
                    ModuleKey::from_host("/root/entry.js"),
                    "import './dep.js';\n\
                     import defer * as deferred from './dep.js';\n\
                     import source artifact from './dep.js';\n\
                     deferred; artifact;"
                        .to_string(),
                    "file:///root/entry.js".to_string(),
                ),
                ModuleSourceIr::new(
                    ModuleKey::from_host("/root/dep.js"),
                    "export const value = 41;".to_string(),
                    "file:///root/dep.js".to_string(),
                ),
            ],
            entry: 0,
            resolutions: vec![(0, ModuleRequestKeyIr::plain("./dep.js"), 1)],
        };

        let mut graph = build_graph(&sources).expect("test modules parse");
        assert_eq!(graph.resolutions.len(), 1);
        assert_eq!(graph.units[0].record.requested_modules.len(), 3);
        assert_eq!(graph.units[0].record.module_resolution_requests.len(), 1);
        for phase in [
            ImportPhaseIr::Evaluation,
            ImportPhaseIr::Defer,
            ImportPhaseIr::Source,
        ] {
            let request = ModuleRequestIr::from_key(ModuleRequestKeyIr::plain("./dep.js"), phase);
            assert_eq!(graph.resolve_request(0, &request), Some(1));
        }

        link(&mut graph);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        assert_eq!(graph.evaluation_mode(1), ModuleEvaluationModeIr::Eager);
    }

    #[test]
    fn conflicting_public_resolution_rows_have_no_last_write_winner() {
        let request = ModuleRequestKeyIr::plain("./dep.js");
        let sources = ModuleGraphSources {
            modules: vec![
                ModuleSourceIr::new(
                    ModuleKey::from_host("/root/entry.js"),
                    "import './dep.js';".to_string(),
                    "file:///root/entry.js".to_string(),
                ),
                ModuleSourceIr::new(
                    ModuleKey::from_host("/root/first.js"),
                    "export const first = 1;".to_string(),
                    "file:///root/first.js".to_string(),
                ),
                ModuleSourceIr::new(
                    ModuleKey::from_host("/root/second.js"),
                    "export const second = 2;".to_string(),
                    "file:///root/second.js".to_string(),
                ),
            ],
            entry: 0,
            resolutions: vec![(0, request.clone(), 1), (0, request.clone(), 2)],
        };

        let graph = build_graph(&sources).expect("test modules parse");
        assert_eq!(graph.resolve_request_key(0, &request), None);
        assert_eq!(
            graph.link_errors,
            vec![ModuleLinkErrorIr::InconsistentResolution {
                referrer: 0,
                request,
            }]
        );
    }

    #[test]
    fn a_repeated_key_with_different_text_is_one_unit_and_an_inconsistent_load() {
        let sources = ModuleGraphSources {
            modules: vec![
                ModuleSourceIr::new(
                    ModuleKey::from_host("/root/entry.js"),
                    "export const x = 1;".to_string(),
                    "file:///root/entry.js".to_string(),
                ),
                ModuleSourceIr::new(
                    ModuleKey::from_host("/root/entry.js"),
                    "export const x = 2;".to_string(),
                    "file:///root/entry.js".to_string(),
                ),
            ],
            entry: 0,
            resolutions: Vec::new(),
        };
        let graph = build_graph(&sources).expect("test modules parse");
        assert_eq!(graph.units.len(), 1);
        assert_eq!(
            graph.link_errors,
            vec![ModuleLinkErrorIr::InconsistentLoad {
                key: ModuleKey::from_host("/root/entry.js"),
            }]
        );
    }

    #[test]
    fn a_two_node_cycle_is_one_contiguous_component() {
        let graph = linked(&[
            (
                "/root/a.js",
                "import { b } from './b.js';\nexport const a = 1;\nb;",
            ),
            (
                "/root/b.js",
                "import { a } from './a.js';\nexport const b = 2;\na;",
            ),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        assert_eq!(components(&graph), vec![vec![1, 0]]);
    }

    #[test]
    fn a_three_node_cycle_is_one_contiguous_component() {
        let graph = linked(&[
            ("/root/a.js", "import './b.js';\nexport const a = 1;"),
            ("/root/b.js", "import './c.js';\nexport const b = 2;"),
            ("/root/c.js", "import './a.js';\nexport const c = 3;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        let components = components(&graph);
        assert_eq!(components.len(), 1);
        assert_eq!(components[0].len(), 3);
        // The entry is the component root, so it runs last.
        assert_eq!(components[0].last().copied(), Some(graph.entry));
    }

    // -- `[[HasTLA]]` / `[[AsyncEvaluation]]` ------------------------------

    #[test]
    fn a_graph_with_no_await_evaluates_synchronously_throughout() {
        let graph = linked(&[
            ("/root/entry.js", "import { x } from './a.js';\nx;"),
            ("/root/a.js", "export const x = 1;"),
        ]);
        assert!(!graph.has_top_level_await());
        assert_eq!(graph.async_evaluation(), vec![false, false]);
        assert_eq!(graph.pending_async_dependencies(graph.entry), 0);
    }

    /// 16.2.1.5.2 step 11.b.i: an importer inherits its dependency's
    /// `[[AsyncEvaluation]]`, and keeps inheriting it up the chain.
    #[test]
    fn async_evaluation_propagates_transitively_to_every_importer() {
        let graph = linked(&[
            ("/root/entry.js", "import { y } from './mid.js';\ny;"),
            (
                "/root/mid.js",
                "import { x } from './leaf.js';\nexport const y = x;",
            ),
            ("/root/leaf.js", "export const x = await 1;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        let leaf = unit_of(&graph, "/root/leaf.js");
        let mid = unit_of(&graph, "/root/mid.js");
        let entry = unit_of(&graph, "/root/entry.js");

        assert!(graph.has_tla(leaf));
        assert!(!graph.has_tla(mid));
        assert!(!graph.has_tla(entry));

        let asynchronous = graph.async_evaluation();
        assert!(asynchronous[leaf as usize]);
        assert!(asynchronous[mid as usize]);
        assert!(asynchronous[entry as usize]);

        assert_eq!(graph.pending_async_dependencies(leaf), 0);
        assert_eq!(graph.pending_async_dependencies(mid), 1);
        assert_eq!(graph.pending_async_dependencies(entry), 1);
    }

    /// A synchronous sibling of an asynchronous module stays synchronous: only
    /// the dependency edge carries `[[AsyncEvaluation]]`, never mere membership
    /// in the same graph.
    #[test]
    fn a_sibling_that_does_not_import_the_awaiting_module_stays_synchronous() {
        let graph = linked(&[
            (
                "/root/entry.js",
                "import { x } from './a.js';\nimport { y } from './b.js';\nx; y;",
            ),
            ("/root/a.js", "export const x = await 1;"),
            ("/root/b.js", "export const y = 2;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        let asynchronous = graph.async_evaluation();
        assert!(asynchronous[unit_of(&graph, "/root/a.js") as usize]);
        assert!(!asynchronous[unit_of(&graph, "/root/b.js") as usize]);
        assert!(asynchronous[graph.entry as usize]);
        // Two dependencies, one of them asynchronous.
        assert_eq!(graph.pending_async_dependencies(graph.entry), 1);
    }

    /// A cycle shares one `[[TopLevelCapability]]`, so one member's `await`
    /// makes every member asynchronous — and a member never waits on another
    /// member, because `InnerModuleEvaluation` evaluates them together.
    #[test]
    fn one_await_in_a_cycle_makes_the_whole_component_asynchronous() {
        let graph = linked(&[
            (
                "/root/a.js",
                "import { b } from './b.js';\nexport const a = 1;\nb;",
            ),
            (
                "/root/b.js",
                "import { a } from './a.js';\nexport const b = await 2;\na;",
            ),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        assert_eq!(graph.async_evaluation(), vec![true, true]);
        assert_eq!(graph.pending_async_dependencies(0), 0);
        assert_eq!(graph.pending_async_dependencies(1), 0);
    }

    #[test]
    fn a_self_importing_module_is_one_unit() {
        // The import has to be aliased. A self-import of an unaliased name is a
        // duplicate lexical declaration — the import binding `x` and the
        // `export const x` are two declarations of `x` in one module
        // environment — so `import { x } from './a.js'; export const x = 1;` is
        // a SyntaxError rather than a graph shape, and boa rejects it before
        // this file is reached.
        let graph = linked(&[(
            "/root/a.js",
            "import { x as y } from './a.js';\nexport const x = 1;\ny;",
        )]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        assert_eq!(graph.units.len(), 1);
        assert_eq!(components(&graph), vec![vec![0]]);
        assert_eq!(
            graph.units[0].resolved_imports,
            vec![ResolvedBindingIr::Resolved {
                module: 0,
                binding: ModuleBindingNameIr::Name(LocalName::from_bound_name("x")),
            }]
        );
    }

    #[test]
    fn an_unresolved_request_is_the_only_error_it_reports() {
        let graph = linked(&[("/root/entry.js", "import { x } from './missing.js';\nx;")]);
        assert_eq!(
            graph.link_errors,
            vec![ModuleLinkErrorIr::UnresolvedModule {
                referrer: 0,
                request: ModuleRequestKeyIr::plain("./missing.js"),
            }]
        );
    }

    #[test]
    fn dependencies_evaluate_before_the_modules_that_import_them() {
        let graph = linked(&[
            ("/root/entry.js", "import './a.js';\nimport './b.js';"),
            ("/root/a.js", "export const a = 1;"),
            ("/root/b.js", "export const b = 2;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        let a = unit_of(&graph, "/root/a.js");
        let b = unit_of(&graph, "/root/b.js");
        assert_eq!(graph.evaluation_order, vec![a, b, graph.entry]);
    }

    #[test]
    fn an_earlier_source_occurrence_does_not_reorder_later_evaluation_dependencies() {
        let graph = linked(&[
            (
                "/root/entry.js",
                "import source artifact from './m.js';\n\
                 import './n.js';\n\
                 import './m.js';\n\
                 artifact;",
            ),
            ("/root/m.js", "export const m = 1;"),
            ("/root/n.js", "export const n = 1;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        let m = unit_of(&graph, "/root/m.js");
        let n = unit_of(&graph, "/root/n.js");
        assert_eq!(graph.evaluation_order, vec![n, m, graph.entry]);
    }

    #[test]
    fn an_earlier_defer_occurrence_does_not_reorder_later_evaluation_dependencies() {
        let graph = linked(&[
            (
                "/root/entry.js",
                "import defer * as deferred from './m.js';\n\
                 import './n.js';\n\
                 import './m.js';\n\
                 deferred;",
            ),
            ("/root/m.js", "export const m = 1;"),
            ("/root/n.js", "export const n = 1;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        let m = unit_of(&graph, "/root/m.js");
        let n = unit_of(&graph, "/root/n.js");
        assert_eq!(graph.evaluation_order, vec![n, m, graph.entry]);
        assert_eq!(graph.evaluation_mode(m), ModuleEvaluationModeIr::Eager);
    }

    #[test]
    fn an_indirect_export_chain_resolves_to_the_module_that_declares_it() {
        let graph = linked(&[
            ("/root/entry.js", "import { x } from './a.js';\nx;"),
            ("/root/a.js", "export { x } from './b.js';"),
            ("/root/b.js", "export { x } from './c.js';"),
            ("/root/c.js", "export const x = 1;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        let c = unit_of(&graph, "/root/c.js");
        let resolved = ResolvedBindingIr::Resolved {
            module: c,
            binding: ModuleBindingNameIr::Name(LocalName::from_bound_name("x")),
        };
        assert_eq!(graph.units[0].resolved_imports, vec![resolved.clone()]);
        // Every link of the chain points at the one declaring cell.
        let a = unit_of(&graph, "/root/a.js");
        assert_eq!(
            graph.unit(a).resolved_indirect_exports,
            vec![resolved.clone()]
        );
        // The merged scope names the exporter's binding exactly as the
        // exporter spells it — no `$m{unit}$` prefix — which is what makes an
        // importer's read a read of the exporter's own cell.
        assert_eq!(
            namespace_target_reference(&resolved),
            Some(LocalName::from_bound_name("x").merged_in(c))
        );
        assert_eq!(
            namespace_target_reference(&resolved).map(|name| name.as_str().to_string()),
            Some("x".to_string())
        );
    }

    #[test]
    fn an_indirect_export_of_a_missing_name_fails_at_link_with_no_importer() {
        // Nothing imports `nope`; the re-export alone must fail.
        let graph = linked(&[
            ("/root/entry.js", "import './a.js';"),
            ("/root/a.js", "export { nope } from './b.js';"),
            ("/root/b.js", "export const x = 1;"),
        ]);
        let a = unit_of(&graph, "/root/a.js");
        assert_eq!(
            graph.link_errors,
            vec![ModuleLinkErrorIr::MissingExport {
                referrer: a,
                request: ModuleRequestIr::plain("./b.js"),
                import_name: ExportName::new("nope"),
            }]
        );
    }

    #[test]
    fn two_star_paths_to_different_bindings_are_ambiguous() {
        let graph = linked(&[
            ("/root/entry.js", "import { x } from './a.js';\nx;"),
            (
                "/root/a.js",
                "export * from './b.js';\nexport * from './c.js';",
            ),
            ("/root/b.js", "export const x = 1;"),
            ("/root/c.js", "export const x = 2;"),
        ]);
        let a = unit_of(&graph, "/root/a.js");
        assert_eq!(
            graph.link_errors,
            vec![ModuleLinkErrorIr::AmbiguousExport {
                module: a,
                export_name: ExportName::new("x"),
            }]
        );
        assert_eq!(
            graph.resolve_export(a, &ExportName::new("x")),
            ResolvedBindingIr::Ambiguous
        );
    }

    #[test]
    fn two_star_paths_to_the_same_binding_are_not_ambiguous() {
        let graph = linked(&[
            ("/root/entry.js", "import { x } from './a.js';\nx;"),
            (
                "/root/a.js",
                "export * from './b.js';\nexport * from './c.js';",
            ),
            ("/root/b.js", "export * from './d.js';"),
            ("/root/c.js", "export * from './d.js';"),
            ("/root/d.js", "export const x = 1;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        let d = unit_of(&graph, "/root/d.js");
        assert_eq!(
            graph.units[0].resolved_imports,
            vec![ResolvedBindingIr::Resolved {
                module: d,
                binding: ModuleBindingNameIr::Name(LocalName::from_bound_name("x")),
            }]
        );
    }

    #[test]
    fn a_cycle_of_export_stars_terminates_and_collects_both_sides() {
        let graph = linked(&[
            ("/root/entry.js", "import { x, y } from './a.js';\nx; y;"),
            ("/root/a.js", "export * from './b.js';\nexport const x = 1;"),
            ("/root/b.js", "export * from './a.js';\nexport const y = 2;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        let a = unit_of(&graph, "/root/a.js");
        let mut names = graph.exported_names(a);
        names.sort();
        assert_eq!(names, vec![ExportName::new("x"), ExportName::new("y")]);
    }

    #[test]
    fn default_is_not_reachable_through_export_star() {
        let graph = linked(&[
            ("/root/entry.js", "import d from './a.js';\nd;"),
            ("/root/a.js", "export * from './b.js';"),
            ("/root/b.js", "export default 1;\nexport const z = 3;"),
        ]);
        let a = unit_of(&graph, "/root/a.js");
        assert_eq!(graph.exported_names(a), vec![ExportName::new("z")]);
        assert_eq!(
            graph.resolve_export(a, &ExportName::default_export()),
            ResolvedBindingIr::NotFound
        );
        assert_eq!(
            graph.link_errors,
            vec![ModuleLinkErrorIr::MissingExport {
                referrer: 0,
                request: ModuleRequestIr::plain("./a.js"),
                import_name: ExportName::default_export(),
            }]
        );
    }

    #[test]
    fn a_namespace_import_resolves_to_the_namespace_cell() {
        let graph = linked(&[
            ("/root/entry.js", "import * as ns from './a.js';\nns;"),
            ("/root/a.js", "export const x = 1;"),
        ]);
        assert!(graph.link_errors.is_empty(), "{:?}", graph.link_errors);
        let a = unit_of(&graph, "/root/a.js");
        let binding = ResolvedBindingIr::Resolved {
            module: a,
            binding: ModuleBindingNameIr::Namespace,
        };
        assert_eq!(graph.units[0].resolved_imports, vec![binding.clone()]);
        assert_eq!(
            namespace_target_reference(&binding),
            Some(MergedName::minted(a, UnitCellRole::Namespace))
        );
    }

    /// Ledger R3: unit ids are spelled into two length-preserving rewrites, so
    /// a graph the linker cannot name is rejected where the id is minted rather
    /// than saturated into a ten-digit id that violates both byte budgets and
    /// fails later with a confusing message.
    #[test]
    fn a_graph_larger_than_the_unit_id_cap_is_rejected_at_the_mint_site() {
        let over_cap = usize::try_from(MAX_LINKABLE_MODULE_UNIT_ID).expect("cap fits") + 2;
        let empty =
            lila_front::parse("", lila_front::ParseOptions::module()).expect("empty module parses");
        let ParsedSource::Module(empty) = empty else {
            unreachable!("module options produce a module")
        };
        let modules: Vec<ModuleSourceIr> = (0..over_cap)
            .map(|index| {
                ModuleSourceIr::from_parsed(
                    ModuleKey::from_host(format!("/root/m{index}.js")),
                    format!("file:///root/m{index}.js"),
                    empty.clone(),
                )
            })
            .collect();
        let sources = ModuleGraphSources {
            modules,
            entry: 0,
            resolutions: Vec::new(),
        };
        let diagnostics = build_graph(&sources).expect_err("the graph exceeds the unit-id cap");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code() == Some(EarlyErrorCode::ModuleTooManyUnits)),
            "{diagnostics:?}"
        );
    }

    /// The cap admits everything up to and including itself, so the rejection
    /// above is not off by one.
    #[test]
    fn the_unit_id_cap_is_inclusive() {
        assert!(ModuleUnitId::try_from(
            usize::try_from(MAX_LINKABLE_MODULE_UNIT_ID).expect("cap fits")
        )
        .is_ok_and(|id| id <= MAX_LINKABLE_MODULE_UNIT_ID));
    }
}
