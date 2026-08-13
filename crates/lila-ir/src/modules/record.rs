//! Source Text Module Records (ECMA-262 16.2.1.6).
//!
//! Owns `ParseModule` (16.2.1.6.1) and the static entry tables it produces:
//! `ImportEntries` (16.2.2.3), `ExportEntries` (16.2.3.2), `ModuleRequests`,
//! and the module `[[Environment]]` shape (9.1.1.5) that
//! `InitializeEnvironment` (16.2.1.6.4) would create.
//!
//! Module bodies keep their original byte offsets so span-derived
//! `FunctionId`s and `Function.prototype.toString` slices stay exact. The one
//! rewrite that lives here — [`rewrite_import_meta`] — is length-preserving and
//! line-preserving for exactly that reason.

use crate::*;

use super::early::module_early_errors;

use boa_ast::declaration::{ImportKind, ImportPhase, ReExportKind};
use boa_ast::expression::{ImportCall, ImportMeta};
use boa_ast::operations::{bound_names, var_declared_names};
use boa_ast::{Module, Position, Span};
use boa_interner::Sym;
use lila_front::SourceSpan;

/// Index of a module in [`ModuleGraphIr::units`](crate::ModuleGraphIr::units).
pub type ModuleUnitId = u32;

/// Phase of a module request (`import`, `import defer`, `import source`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ImportPhaseIr {
    /// A normal eager request.
    #[default]
    Evaluation,
    /// `import defer * as ns from "m"`.
    Defer,
    /// `import source x from "m"`.
    Source,
}

impl ImportPhaseIr {
    /// Name used in diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Evaluation => "evaluation",
            Self::Defer => "defer",
            Self::Source => "source",
        }
    }

    pub(super) const fn from_ast(phase: ImportPhase) -> Self {
        match phase {
            ImportPhase::Evaluation => Self::Evaluation,
            ImportPhase::Defer => Self::Defer,
            ImportPhase::Source => Self::Source,
        }
    }
}

/// One `with { key: value }` import attribute.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImportAttributeIr {
    /// Attribute key, as written.
    pub key: String,
    /// Attribute value, as written.
    pub value: String,
}

/// A repeated key refused by [`ModuleRequestAttributesIr`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateImportAttributeKeyIr {
    key: String,
}

impl DuplicateImportAttributeKeyIr {
    /// The duplicated attribute key.
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }
}

impl core::fmt::Display for DuplicateImportAttributeKeyIr {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "duplicate import attribute key: {}", self.key)
    }
}

impl std::error::Error for DuplicateImportAttributeKeyIr {}

/// The canonical `[[Attributes]]` list of a ModuleRequest Record.
///
/// Keys are unique and sorted by UTF-16 code-unit order. The storage is
/// private and only an immutable slice escapes, so derived request identity
/// cannot drift after a request becomes a graph-map key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ModuleRequestAttributesIr(Vec<ImportAttributeIr>);

impl ModuleRequestAttributesIr {
    /// Validates and canonicalizes one import-attribute list.
    ///
    /// # Errors
    /// Returns the duplicated key when the input contains two records with the
    /// same key. Static import syntax rejects that shape, and dynamic import
    /// obtains keys from an object's unique own-property keys.
    pub fn try_new(
        attributes: impl IntoIterator<Item = ImportAttributeIr>,
    ) -> Result<Self, DuplicateImportAttributeKeyIr> {
        let mut attributes: Vec<_> = attributes.into_iter().collect();
        attributes.sort_by(|left, right| left.key.encode_utf16().cmp(right.key.encode_utf16()));
        if let Some(pair) = attributes
            .windows(2)
            .find(|pair| pair[0].key == pair[1].key)
        {
            return Err(DuplicateImportAttributeKeyIr {
                key: pair[0].key.clone(),
            });
        }
        Ok(Self(attributes))
    }

    /// The empty attribute list.
    #[must_use]
    pub const fn empty() -> Self {
        Self(Vec::new())
    }

    /// Canonical attributes in UTF-16 key order.
    #[must_use]
    pub fn as_slice(&self) -> &[ImportAttributeIr] {
        &self.0
    }
}

/// What graph discovery can know about an `import()` call's `with` object.
///
/// `Known` includes the empty list. `Runtime` means evaluating the options
/// object, its `with` property, or its enumerable properties is required to
/// learn the list. Keeping that distinction closed prevents an absent options
/// expression from being conflated with an expression whose eventual request
/// is not statically knowable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DynamicImportAttributesIr {
    /// The exact attributes are known and sorted by UTF-16 key order.
    ///
    /// ```compile_fail
    /// use lila_ir::DynamicImportAttributesIr;
    ///
    /// let _ = DynamicImportAttributesIr::Known(Vec::new());
    /// ```
    Known(ModuleRequestAttributesIr),
    /// Runtime evaluation is required to obtain the attributes.
    Runtime,
}

/// The phase-free identity of a `ModuleRequest` Record.
///
/// `ModuleRequestsEqual` compares only `[[Specifier]]` and `[[Attributes]]`.
/// Keeping both fields private makes the canonical form immutable after this
/// value becomes a host-resolution or module-map key.
///
/// ```compile_fail
/// use lila_ir::{ModuleRequestAttributesIr, ModuleRequestKeyIr};
///
/// let _ = ModuleRequestKeyIr {
///     specifier: "./m.js".to_string(),
///     attributes: ModuleRequestAttributesIr::empty(),
/// };
/// ```
///
/// ```compile_fail
/// use lila_ir::ModuleRequestKeyIr;
///
/// let mut request = ModuleRequestKeyIr::plain("./m.js");
/// request.specifier = "./other.js".to_string();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleRequestKeyIr {
    /// `[[Specifier]]`, exactly as written.
    specifier: String,
    /// `[[Attributes]]`, sorted by key.
    attributes: ModuleRequestAttributesIr,
}

impl ModuleRequestKeyIr {
    /// Creates a key from an already canonical attribute list.
    #[must_use]
    pub fn new(specifier: impl Into<String>, attributes: ModuleRequestAttributesIr) -> Self {
        Self {
            specifier: specifier.into(),
            attributes,
        }
    }

    /// Validates raw attributes and creates their key in one operation.
    ///
    /// # Errors
    /// Returns the duplicated key when `attributes` contains one.
    pub fn try_new(
        specifier: impl Into<String>,
        attributes: impl IntoIterator<Item = ImportAttributeIr>,
    ) -> Result<Self, DuplicateImportAttributeKeyIr> {
        Ok(Self::new(
            specifier,
            ModuleRequestAttributesIr::try_new(attributes)?,
        ))
    }

    /// A request key with no attributes.
    #[must_use]
    pub fn plain(specifier: impl Into<String>) -> Self {
        Self::new(specifier, ModuleRequestAttributesIr::empty())
    }

    /// `[[Specifier]]`, exactly as written.
    #[must_use]
    pub fn specifier(&self) -> &str {
        &self.specifier
    }

    /// Canonical `[[Attributes]]` in UTF-16 key order.
    #[must_use]
    pub fn attributes(&self) -> &[ImportAttributeIr] {
        self.attributes.as_slice()
    }
}

/// One phaseful occurrence of a `ModuleRequest` Record.
///
/// Phase is needed for import dispatch and evaluation classification, but is
/// deliberately outside [`ModuleRequestKeyIr`]. Derived equality here means
/// occurrence equality; host resolution and `ModuleRequestsEqual` must use
/// [`Self::key`] instead.
///
/// ```compile_fail
/// use lila_ir::{ImportPhaseIr, ModuleRequestIr, ModuleRequestKeyIr};
///
/// let _ = ModuleRequestIr {
///     key: ModuleRequestKeyIr::plain("./m.js"),
///     phase: ImportPhaseIr::Evaluation,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleRequestIr {
    key: ModuleRequestKeyIr,
    phase: ImportPhaseIr,
}

impl ModuleRequestIr {
    /// Creates an occurrence from an already canonical attribute list.
    #[must_use]
    pub fn new(
        specifier: impl Into<String>,
        phase: ImportPhaseIr,
        attributes: ModuleRequestAttributesIr,
    ) -> Self {
        Self::from_key(ModuleRequestKeyIr::new(specifier, attributes), phase)
    }

    /// Creates an occurrence from its phase-free identity and phase.
    #[must_use]
    pub const fn from_key(key: ModuleRequestKeyIr, phase: ImportPhaseIr) -> Self {
        Self { key, phase }
    }

    /// Validates raw attributes and creates their occurrence in one operation.
    ///
    /// # Errors
    /// Returns the duplicated key when `attributes` contains one.
    pub fn try_new(
        specifier: impl Into<String>,
        phase: ImportPhaseIr,
        attributes: impl IntoIterator<Item = ImportAttributeIr>,
    ) -> Result<Self, DuplicateImportAttributeKeyIr> {
        Ok(Self::from_key(
            ModuleRequestKeyIr::try_new(specifier, attributes)?,
            phase,
        ))
    }

    /// A plain evaluation-phase occurrence with no attributes.
    #[must_use]
    pub fn plain(specifier: impl Into<String>) -> Self {
        Self::from_key(
            ModuleRequestKeyIr::plain(specifier),
            ImportPhaseIr::Evaluation,
        )
    }

    /// The phase-free identity used by host resolution and module maps.
    #[must_use]
    pub const fn key(&self) -> &ModuleRequestKeyIr {
        &self.key
    }

    /// `[[Specifier]]`, exactly as written.
    #[must_use]
    pub fn specifier(&self) -> &str {
        self.key.specifier()
    }

    /// `[[Phase]]` for this occurrence.
    #[must_use]
    pub const fn phase(&self) -> ImportPhaseIr {
        self.phase
    }

    /// Canonical `[[Attributes]]` in UTF-16 key order.
    #[must_use]
    pub fn attributes(&self) -> &[ImportAttributeIr] {
        self.key.attributes()
    }
}

/// A resolved request that participates in module evaluation.
///
/// The target field is private and the only constructor checks the request's
/// phase exhaustively, so evaluation-order and async-evaluation code cannot
/// accidentally accept a defer- or source-phase edge after resolution has
/// erased the request context.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ModuleEvaluationDependencyIr(ModuleUnitId);

impl ModuleEvaluationDependencyIr {
    /// Retains a resolved target only when the request evaluates it.
    #[must_use]
    pub(super) const fn from_resolved_request(
        request: &ModuleRequestIr,
        target: ModuleUnitId,
    ) -> Option<Self> {
        match request.phase() {
            ImportPhaseIr::Evaluation => Some(Self(target)),
            ImportPhaseIr::Defer | ImportPhaseIr::Source => None,
        }
    }

    /// The resolved unit that must evaluate before its referrer.
    #[must_use]
    pub(super) const fn target(self) -> ModuleUnitId {
        self.0
    }
}

/// `[[ImportName]]` of an import or indirect-export entry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImportNameIr {
    /// The namespace exotic object of the requested module.
    ///
    /// Produced by `import * as ns from "m"` and `export * as ns from "m"`.
    Namespace,
    /// A named export of the requested module.
    ///
    /// An [`ExportName`], not a [`LocalName`]: 16.2.1.4 defines `[[ImportName]]`
    /// as "the name under which the desired binding is exported by the module
    /// identified by `[[ModuleRequest]]`", and `ResolveExport` takes it as its
    /// `exportName` argument. It is the requested module's D2, read from this
    /// side.
    Name(ExportName),
}

/// An `ImportEntry` Record (16.2.2.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportEntryIr {
    /// `[[ModuleRequest]]`.
    pub request: ModuleRequestIr,
    /// `[[ImportName]]`.
    pub import_name: ImportNameIr,
    /// `[[LocalName]]`, the immutable indirect binding the import creates.
    pub local_name: LocalName,
    /// Span of the declaration, for diagnostics.
    pub span: Option<SourceSpan>,
}

/// A `LocalExportEntry` Record.
///
/// The two fields are the whole reason this area has a contract: 16.2.1.5 gives
/// them different definitions and `export { a as b }` makes them differ, yet
/// `export let x` makes them coincide — which is exactly the case that hides a
/// swap. They are different types, so the swap is `E0308`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalExportEntryIr {
    /// `[[LocalName]]`; [`LocalName::AnonymousDefault`] for an anonymous
    /// `export default`.
    pub local_name: LocalName,
    /// `[[ExportName]]`.
    pub export_name: ExportName,
}

/// An `IndirectExportEntry` Record.
///
/// Carries no `[[LocalName]]` at all: `export { x } from "m"` binds nothing in
/// this module, which is why ExportedBindings excludes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndirectExportEntryIr {
    /// `[[ModuleRequest]]`.
    pub request: ModuleRequestIr,
    /// `[[ImportName]]`.
    pub import_name: ImportNameIr,
    /// `[[ExportName]]`.
    pub export_name: ExportName,
}

/// A `StarExportEntry` Record: `export * from "m"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StarExportEntryIr {
    /// `[[ModuleRequest]]`.
    pub request: ModuleRequestIr,
}

/// How a module-environment binding was declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleBindingKindIr {
    /// Created by `CreateImportBinding` (9.1.1.5.5): immutable and indirect.
    Import,
    /// `var`, including `var`s nested in blocks of the module body.
    Var,
    /// `let`.
    Let,
    /// `const`.
    Const,
    /// A hoistable declaration, initialized before the module body runs.
    Function,
    /// `class`.
    Class,
}

/// One binding of the module `[[Environment]]` (9.1.1.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleEnvBindingIr {
    /// Spec binding name ([`LocalName::AnonymousDefault`] for an anonymous
    /// `export default`).
    pub name: LocalName,
    /// How the binding was declared.
    pub kind: ModuleBindingKindIr,
    /// `false` for `const` and for import bindings.
    pub mutable: bool,
    /// `true` when `InitializeEnvironment` initializes it before evaluation.
    pub initialized_before_evaluation: bool,
    /// `true` when a read before the module body initializes it is a
    /// `ReferenceError` (`let`/`const`/`class`/`export default <expr>`).
    pub in_tdz_until_evaluated: bool,
    /// Set only for import bindings: `[[Module]]`/`[[BindingName]]`.
    pub indirect: Option<(ModuleRequestIr, ImportNameIr)>,
}

/// One `import(...)` call site found in a module body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicImportSiteIr {
    /// Specifier when the argument is a string literal, so the target can be
    /// compiled into the artifact. `None` for computed specifiers.
    pub static_specifier: Option<String>,
    /// `[[Phase]]` of the call.
    pub phase: ImportPhaseIr,
    /// Compile-time knowledge of the request's `[[Attributes]]`.
    pub attributes: DynamicImportAttributesIr,
}

impl DynamicImportSiteIr {
    /// The full occurrence whose key graph discovery can ask the host to resolve.
    ///
    /// Runtime attributes deliberately discover the empty request variant.
    /// The emitted dispatcher still evaluates and validates their actual
    /// values, and will only match an exact request variant present in the AOT
    /// component registry. This can over-compile the attribute-free target but
    /// cannot make a differently attributed request resolve to it.
    pub(super) fn discovery_request(&self) -> Option<ModuleRequestIr> {
        let specifier = self.static_specifier.clone()?;
        let attributes = match &self.attributes {
            DynamicImportAttributesIr::Known(attributes) => attributes.clone(),
            DynamicImportAttributesIr::Runtime => ModuleRequestAttributesIr::empty(),
        };
        Some(ModuleRequestIr::new(specifier, self.phase, attributes))
    }
}

/// A [Source Text Module Record][spec] (16.2.1.6), minus evaluation state.
///
/// [spec]: https://tc39.es/ecma262/#sec-source-text-module-records
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTextModuleRecordIr {
    /// Index of this record in the graph.
    pub id: ModuleUnitId,
    /// Host-normalized resolution key this record is registered under.
    pub key: ModuleKey,
    /// Byte length of the module source text.
    pub source_len: usize,
    /// `[[HasTLA]]`: the module body contains a top-level `await`.
    pub has_top_level_await: bool,
    /// `[[RequestedModules]]`, source order, deduplicated by canonical key and
    /// phase.
    ///
    /// `ModuleRequestsEqual` itself is phase-free; the static-semantics list
    /// retains one occurrence for each phase of that key. Link and evaluation
    /// consume this list directly so an earlier source/defer occurrence cannot
    /// move a later evaluation occurrence ahead of another dependency.
    pub requested_modules: Vec<ModuleRequestIr>,
    /// Phase-free projection used only for host discovery and resolution.
    ///
    /// First-key order is retained for deterministic loading, but this is not
    /// the specification's phaseful `[[RequestedModules]]` list.
    pub module_resolution_requests: Vec<ModuleRequestKeyIr>,
    /// `[[ImportEntries]]`.
    pub import_entries: Vec<ImportEntryIr>,
    /// `[[LocalExportEntries]]`.
    pub local_export_entries: Vec<LocalExportEntryIr>,
    /// `[[IndirectExportEntries]]`.
    pub indirect_export_entries: Vec<IndirectExportEntryIr>,
    /// `[[StarExportEntries]]`.
    pub star_export_entries: Vec<StarExportEntryIr>,
    /// Shape of the module `[[Environment]]`.
    pub environment: Vec<ModuleEnvBindingIr>,
    /// Byte span of every `import.meta` reference in the module body, in
    /// ascending order, indexing this module's own source text.
    ///
    /// A span, not a count: the linker rewrites `import.meta` out of the body
    /// (Script text cannot spell it) and needs to know where each one is. See
    /// [`rewrite_import_meta`].
    pub import_meta_sites: Vec<SourceSpan>,
    /// Every `import(...)` call site in the module body.
    pub dynamic_import_sites: Vec<DynamicImportSiteIr>,
}

/// Shape of a module's `export default`, from the merged script's point of
/// view. See [`SourceTextModuleRecordIr::default_export_form`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultExportFormIr {
    /// The module has no `export default`.
    Absent,
    /// `export default function f() {}` / `export default class C {}`: the
    /// declaration binds `f` / `C` itself.
    Named,
    /// Every other form. `hoisted` distinguishes an anonymous
    /// `HoistableDeclaration`, which is initialized before the body runs, from
    /// a `ClassDeclaration` or `AssignmentExpression`, which is in TDZ until
    /// its own statement is reached.
    Anonymous {
        /// The declaration is a hoistable one.
        hoisted: bool,
    },
}

impl SourceTextModuleRecordIr {
    /// The merged spelling of one of *this* record's `[[LocalName]]`s.
    ///
    /// [`LocalName::merged_in`] takes a bare `ModuleUnitId`, so "which unit does
    /// this name belong to" — the exact coordinate commit `e27c01b1e` got wrong
    /// — was carried by loop structure rather than by types: passing the
    /// importer's id where the exporter's is required compiled, was silent for
    /// [`LocalName::Source`] (`merged_in` ignores the unit there) and produced
    /// the wrong `$d<unit>$` cell for [`LocalName::AnonymousDefault`].
    ///
    /// Every call site that already holds the record goes through here instead,
    /// so the id cannot be supplied independently of the name's owner. The one
    /// site that legitimately keeps `merged_in` destructures both from the same
    /// `ResolvedBindingIr::Resolved { module, binding }`, where the pairing is
    /// structural already.
    #[must_use]
    pub fn merged(&self, name: &LocalName) -> MergedName {
        name.merged_in(self.id)
    }

    /// Number of `import.meta` references in the module body.
    ///
    /// `import.meta` is created lazily and at most once per module (13.3.12
    /// step 4), so a module with zero uses needs no object at all and the
    /// linker declares none.
    #[must_use]
    pub fn import_meta_uses(&self) -> usize {
        self.import_meta_sites.len()
    }

    /// Export names declared without consulting the graph (no `export *`).
    #[must_use]
    pub fn own_exported_names(&self) -> Vec<ExportName> {
        let mut names = Vec::new();
        for entry in &self.local_export_entries {
            push_unique_name(&mut names, &entry.export_name);
        }
        for entry in &self.indirect_export_entries {
            push_unique_name(&mut names, &entry.export_name);
        }
        names
    }

    /// `true` when the module needs no other source unit to be linked.
    #[must_use]
    pub fn is_self_contained(&self) -> bool {
        self.requested_modules.is_empty()
    }

    /// How this module's `export default`, if any, has to be rewritten for the
    /// merged script (16.2.3.7).
    ///
    /// The distinction the linker needs is not the grammar production but
    /// whether the declaration already binds a name the merged scope can spell.
    /// `export default function f() {}` and `export default class C {}` do, so
    /// deleting the two keywords leaves a declaration that binds exactly what
    /// the export entry names. Every other form has `[[LocalName]]`
    /// [`LocalName::AnonymousDefault`], which nothing can spell, so the
    /// keywords have to become a declaration of a minted name instead.
    ///
    /// The `[[ExportName]]` test and the `[[LocalName]]` test below are asked
    /// of different types, because they are different domains: `default` is
    /// spellable from source (`export { x as default }`) while `*default*`
    /// never is.
    #[must_use]
    pub fn default_export_form(&self) -> DefaultExportFormIr {
        let Some(entry) = self
            .local_export_entries
            .iter()
            .find(|entry| entry.export_name.is_default())
        else {
            return DefaultExportFormIr::Absent;
        };
        if entry.local_name != LocalName::AnonymousDefault {
            return DefaultExportFormIr::Named;
        }
        // A hoistable declaration is initialized before the body runs, so it
        // must not be given the TDZ a `let` would.
        let hoisted = self
            .environment
            .iter()
            .find(|binding| binding.name == LocalName::AnonymousDefault)
            .is_some_and(|binding| binding.kind == ModuleBindingKindIr::Function);
        DefaultExportFormIr::Anonymous { hoisted }
    }

    /// Duplicate `[[ExportName]]`s, which are an early error (16.2.3.1).
    #[must_use]
    pub fn duplicate_export_names(&self) -> Vec<ExportName> {
        let mut seen = BTreeSet::new();
        let mut duplicates = Vec::new();
        let names = self
            .local_export_entries
            .iter()
            .map(|entry| &entry.export_name)
            .chain(
                self.indirect_export_entries
                    .iter()
                    .map(|entry| &entry.export_name),
            );
        for name in names {
            if !seen.insert(name.clone()) {
                push_unique_name(&mut duplicates, name);
            }
        }
        duplicates
    }
}

// -- `import.meta` (13.3.12, 16.2.1.9) ------------------------------------
//
// `import.meta` is an ordinary object, created at most once per module, that
// the host populates. Three properties have to hold and each one falls out of a
// different part of the shape below:
//
// * *identity-stable within a module* — `import.meta === import.meta`. One
//   `const` binding per module, evaluated once, so every reference reads the
//   same cell.
// * *distinct per module* — two modules never share the object. The binding is
//   named from the unit id.
// * *ordinary and extensible* — an object literal is extensible and its
//   properties are writable, enumerable and configurable, which is what
//   `CreateDataPropertyOrThrow` gives them in step 4.c. The prototype is
//   `null`: step 4.a is `OrdinaryObjectCreate(null)`, not
//   `OrdinaryObjectCreate(%Object.prototype%)`.
//
// `url` is this host's whole `HostGetImportMetaProperties` result (16.2.1.9).
// It is a normative-optional hook, so a host is free to define exactly one
// property, and `url` is the one every other host agrees on.

/// The `import.meta` object of one module, as Script source text.
///
/// Source text rather than IR because the linker merges units by concatenating
/// their bodies and lowering once — see `modules::link`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportMetaBindingIr {
    /// Binding that holds the object, and that every rewritten `import.meta`
    /// in this unit reads.
    ///
    /// A [`MergedName`] minted for [`UnitCellRole::ImportMeta`], so it cannot
    /// collide with another unit's meta object, with a namespace cell, or with
    /// an ordinary top-level binding of the merged scope — and so its byte
    /// length is bounded by const assertion V4, which is what
    /// [`rewrite_import_meta`] depends on.
    pub name: MergedName,
    /// The declaration that creates it, terminated by `;`.
    ///
    /// The linker emits this once, before any unit body runs.
    pub declaration: String,
}

/// Module `unit`'s `import.meta` object, with the host properties 16.2.1.9
/// defines for it.
///
/// `meta_url` is [`ModuleUnitIr::meta_url`](crate::ModuleUnitIr::meta_url): the
/// host resolved and normalized it while loading, and nothing here re-derives
/// it.
#[must_use]
pub fn import_meta_binding(unit: ModuleUnitId, meta_url: &str) -> ImportMetaBindingIr {
    let name = MergedName::minted(unit, UnitCellRole::ImportMeta);
    // `__proto__:` in an object literal is the prototype setter (13.2.5.5), and
    // the lowerer recognises a `null` value as "no prototype"
    // (`lowering::lower_object_literal`), so this is `OrdinaryObjectCreate(null)`
    // without needing `Object.create` — which a unit could have shadowed by
    // declaring its own top-level `Object`.
    let declaration = format!(
        "const {} = {{ __proto__: null, url: {} }};",
        name.as_str(),
        js_string_literal(meta_url)
    );
    ImportMetaBindingIr { name, declaration }
}

/// Why a module body's `import.meta` references could not be rewritten.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportMetaRewriteError {
    /// Human-readable reason, already phrased as a diagnostic message body.
    pub reason: String,
}

impl ImportMetaRewriteError {
    fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }
}

/// Replaces every `import.meta` in `source` with a read of module `record.id`'s
/// [`UnitCellRole::ImportMeta`] cell.
///
/// Script text cannot spell `import.meta` at all — boa rejects it outright
/// outside a module — so a linker that merges units into one Script *must*
/// rewrite it, and rewriting it into a binding read is also what makes the
/// object identity-stable for free.
///
/// `source` must be the unit's *original* text: `import_meta_sites` was built
/// against it. Run this first and
/// `modules::source::strip_module_syntax` second — the output below is
/// byte-for-byte the same length, it contains no `import` keyword the stripper
/// could mistake for a declaration, and `$m…$meta` lexes as the ordinary
/// identifier it is. The other order only happens to work while every byte the
/// stripper deletes is ASCII, because it blanks a deleted range one *character*
/// per space.
///
/// The rewrite preserves the source's byte length and its line terminators, so
/// span-derived `FunctionId`s and reported line numbers do not move. The
/// replacement identifier is shorter than `import.meta`, and the slack is
/// filled with the line terminators the span contained followed by spaces.
///
/// # Errors
/// Returns [`ImportMetaRewriteError`] when a site does not address the text it
/// is supposed to (which would mean `source` is not this unit's), or when the
/// binding name does not fit in the span it replaces.
///
/// The second failure is unreachable for any unit the graph will admit: the
/// narrowest `import.meta` span is 11 bytes, const
/// assertion V4 holds `MergedName::minted(u, ImportMeta)` to that width for
/// every `u <= MAX_LINKABLE_MODULE_UNIT_ID`, and `build_graph` mints no larger
/// id. It stays checked because the *span* is data from boa, not a constant.
pub fn rewrite_import_meta(
    source: &str,
    record: &SourceTextModuleRecordIr,
) -> Result<String, ImportMetaRewriteError> {
    if record.import_meta_sites.is_empty() {
        return Ok(source.to_string());
    }

    let cell = MergedName::minted(record.id, UnitCellRole::ImportMeta);
    let name = cell.as_str();
    let mut rewritten = String::with_capacity(source.len());
    let mut cursor = 0usize;
    for site in &record.import_meta_sites {
        let replaced = source
            .get(site.start..site.end)
            .filter(|_| site.start >= cursor)
            .ok_or_else(|| {
                ImportMetaRewriteError::new(format!(
                    "`import.meta` at {}..{} is not a span of this module's source text",
                    site.start, site.end
                ))
            })?;
        // A cheap proof that the span really is the meta-property and not some
        // unrelated range: the rewrite destroys whatever it covers, so it is
        // worth refusing rather than silently mangling a body.
        if !replaced.starts_with(IMPORT_META_HEAD) || !replaced.ends_with(IMPORT_META_TAIL) {
            return Err(ImportMetaRewriteError::new(format!(
                "`import.meta` span {}..{} covers `{replaced}`",
                site.start, site.end
            )));
        }
        rewritten.push_str(&source[cursor..site.start]);
        rewritten.push_str(name);
        rewritten.push_str(&padding_for(replaced, name.len()).ok_or_else(|| {
            ImportMetaRewriteError::new(format!(
                "`import.meta` binding `{name}` does not fit the {} bytes it replaces",
                replaced.len()
            ))
        })?);
        cursor = site.end;
    }
    rewritten.push_str(&source[cursor..]);
    Ok(rewritten)
}

/// Filler that pads `name_len` bytes back out to `replaced.len()` while keeping
/// every line terminator `replaced` contained.
///
/// The terminators come first and the spaces after, so the token following the
/// site keeps its line *and* the line count of the unit is unchanged. Its
/// column shifts, which is the one thing this cannot preserve: an identifier
/// cannot be split across the lines the meta-property was written on.
///
/// Emitting the terminators after the identifier rather than before it is
/// deliberate. A line terminator before it could turn the preceding token into
/// the end of a statement under ASI; after it, the only terminators that can
/// appear between this site and the next token are ones the source already had
/// inside the span, so no ASI decision changes.
fn padding_for(replaced: &str, name_len: usize) -> Option<String> {
    let terminators: String = replaced
        .chars()
        .filter(|ch| matches!(ch, '\n' | '\r' | '\u{2028}' | '\u{2029}'))
        .collect();
    let spaces = replaced
        .len()
        .checked_sub(name_len)?
        .checked_sub(terminators.len())?;
    let mut padding = terminators;
    padding.push_str(&" ".repeat(spaces));
    Some(padding)
}

/// `value` as a double-quoted JavaScript string literal.
///
/// U+2028 and U+2029 are escaped even though they are legal in a string literal
/// since ES2019: the merged source is assembled by concatenation, and an
/// unescaped one is a line terminator to the line index the record's spans are
/// built from.
fn js_string_literal(value: &str) -> String {
    let mut literal = String::with_capacity(value.len() + 2);
    literal.push('"');
    for character in value.chars() {
        match character {
            '"' => literal.push_str("\\\""),
            '\\' => literal.push_str("\\\\"),
            '\n' => literal.push_str("\\n"),
            '\r' => literal.push_str("\\r"),
            '\u{2028}' => literal.push_str("\\u2028"),
            '\u{2029}' => literal.push_str("\\u2029"),
            character if (character as u32) < 0x20 => {
                literal.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => literal.push(character),
        }
    }
    literal.push('"');
    literal
}

/// Appends `name` to `names` unless it is already present.
///
/// D2 only. Every one of its call sites — here and in `modules::graph`'s
/// `GetExportedNames` walk — is building an `ExportedNames` list, so a
/// `[[LocalName]]` reaching one is `E0308` rather than a namespace object with
/// a wrong key.
pub(crate) fn push_unique_name(names: &mut Vec<ExportName>, name: &ExportName) {
    if !names.iter().any(|existing| existing == name) {
        names.push(name.clone());
    }
}

fn push_unique_request(requests: &mut Vec<ModuleRequestIr>, request: &ModuleRequestIr) {
    if !requests.iter().any(|existing| existing == request) {
        requests.push(request.clone());
    }
}

fn push_unique_request_key(requests: &mut Vec<ModuleRequestKeyIr>, request: &ModuleRequestKeyIr) {
    if !requests.iter().any(|existing| existing == request) {
        requests.push(request.clone());
    }
}

fn resolved_name(interner: &Interner, name: Sym) -> String {
    interner.resolve_expect(name).to_string()
}

/// `[[LocalName]]` of a `BindingIdentifier` boa resolved out of the interner.
///
/// Every caller is a BoundNames position, so the classification
/// [`LocalName::from_bound_name`] performs is the right one and is total: only
/// `*default*` is not a source-spelled name, and only boa's own
/// anonymous-default marker can produce it.
fn resolved_local_name(interner: &Interner, name: Sym) -> LocalName {
    LocalName::from_bound_name(resolved_name(interner, name))
}

/// `[[ExportName]]` / `[[ImportName]]` boa resolved out of the interner.
///
/// A separate function from [`resolved_local_name`] on purpose: the two
/// accessors boa offers for a `ModuleExportName` pair are called
/// `private_name()` and `alias()` on both `ReExportKind::Named` and
/// `ExportDeclaration::List`, yet `private_name()` means `[[ImportName]]` in the
/// first and `[[LocalName]]` in the second. The conversion names the domain
/// where boa's accessor does not, and swapping the two conversions is `E0308`
/// one line later at the struct literal.
fn resolved_export_name(interner: &Interner, name: Sym) -> ExportName {
    ExportName::new(resolved_name(interner, name))
}

/// Byte offset of the start of every line, so boa's `(line, column)` positions
/// can be turned back into the byte offsets [`SourceSpan`] is expressed in.
///
/// Mirrors the lexer exactly (`boa_parser-0.21.1/src/lexer/cursor.rs`,
/// `next_char`): a line ends at `\r\n`, a bare `\r`, `\n`, U+2028 or U+2029,
/// and a column counts *code points*, not bytes and not UTF-16 code units.
#[derive(Debug)]
struct LineIndex {
    /// Byte offset of the first character of each line, in line order.
    line_starts: Vec<usize>,
    /// Byte length of the source, used as the clamp for anything out of range.
    len: usize,
}

impl LineIndex {
    fn new(source: &str) -> Self {
        let mut line_starts = vec![0usize];
        let mut characters = source.char_indices().peekable();
        while let Some((offset, character)) = characters.next() {
            let break_len = match character {
                '\r' => {
                    if characters.peek().is_some_and(|(_, next)| *next == '\n') {
                        characters.next();
                        '\r'.len_utf8() + '\n'.len_utf8()
                    } else {
                        '\r'.len_utf8()
                    }
                }
                '\n' | '\u{2028}' | '\u{2029}' => character.len_utf8(),
                _ => continue,
            };
            line_starts.push(offset + break_len);
        }
        Self {
            line_starts,
            len: source.len(),
        }
    }

    /// Byte offset of `position`, clamped to the end of its line.
    ///
    /// A `Span`'s end position is one past its last character, so a column of
    /// `line length + 1` is normal and must land on the line's last byte rather
    /// than swallowing the line break.
    fn offset(&self, source: &str, position: Position) -> usize {
        let line = position.line_number() as usize;
        let Some(&start) = self.line_starts.get(line - 1) else {
            return self.len;
        };
        let end = self.line_starts.get(line).copied().unwrap_or(self.len);
        let content = source[start..end].trim_end_matches(['\r', '\n', '\u{2028}', '\u{2029}']);
        content
            .char_indices()
            .nth(position.column_number() as usize - 1)
            .map_or(start + content.len(), |(offset, _)| start + offset)
    }

    fn source_span(&self, source: &str, span: Span) -> SourceSpan {
        let start = self.offset(source, span.start());
        SourceSpan {
            start,
            end: self.offset(source, span.end()).max(start),
        }
    }
}

fn module_request(
    interner: &Interner,
    request: &boa_ast::declaration::ModuleRequest,
) -> ModuleRequestIr {
    let attributes: Vec<ImportAttributeIr> = request
        .attributes()
        .iter()
        .map(|attribute| ImportAttributeIr {
            key: resolved_name(interner, attribute.key()),
            value: resolved_name(interner, attribute.value()),
        })
        .collect();
    let attributes = ModuleRequestAttributesIr::try_new(attributes)
        .expect("the parser rejects duplicate static import attribute keys");
    ModuleRequestIr::new(
        resolved_name(interner, request.specifier().sym()),
        ImportPhaseIr::from_ast(request.phase()),
        attributes,
    )
}

/// `ParseModule` (16.2.1.6.1): builds a module record from the front end's
/// retained parse product.
///
/// The `ParsedModule` type proves syntax parsing and Boa static semantics
/// already succeeded. This operation can therefore fail only for an early
/// error of `Module : ModuleBody` (16.2.3.1) found by validating the Lila-owned
/// entry tables it constructs.
///
/// Top-level `await` is *not* one of them. It sets `[[HasTLA]]` on the record;
/// [`ModuleGraphIr::async_evaluation`] propagates it across the graph and the
/// link stage gives the merged program an asynchronous body.
///
/// Everything else — unresolved specifiers, missing or ambiguous exports — is
/// the graph's job, not this function's, because none of it is decidable from a
/// single module.
pub fn parse_module_record(
    source: &ParsedModule,
    id: ModuleUnitId,
    key: ModuleKey,
) -> Result<SourceTextModuleRecordIr, Vec<IrDiagnostic>> {
    let record = source.with_compiler_session(|module, interner| {
        build_module_record(module, interner, source.source(), id, key)
    });

    // Early errors first: a `SyntaxError` outranks an unsupported feature, and
    // a module that is not well-formed has no meaningful `[[HasTLA]]`.
    let early_errors = module_early_errors(&record);
    if !early_errors.is_empty() {
        return Err(early_errors);
    }

    // `[[HasTLA]]` is a property of the record, never a failure of ParseModule:
    // 16.2.1.6.1 step 12 simply stores it. What it costs is decided later, by
    // the graph (which modules become `[[AsyncEvaluation]]`) and by the link
    // stage (which gives the merged program an asynchronous body).
    Ok(record)
}

fn build_module_record(
    module: &Module,
    interner: &Interner,
    source: &SourceUnit,
    id: ModuleUnitId,
    key: ModuleKey,
) -> SourceTextModuleRecordIr {
    let item_list = module.items();
    let items = item_list.items();
    let text = source.source_text.as_str();
    let lines = LineIndex::new(text);

    let mut record = SourceTextModuleRecordIr {
        id,
        key,
        source_len: source.source_text.len(),
        has_top_level_await: false,
        requested_modules: Vec::new(),
        module_resolution_requests: Vec::new(),
        import_entries: Vec::new(),
        local_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        star_export_entries: Vec::new(),
        environment: Vec::new(),
        import_meta_sites: Vec::new(),
        dynamic_import_sites: Vec::new(),
    };

    // 16.2.2.2 ModuleRequests, over *all* module items in source order. The
    // phaseful list deduplicates equal `(ModuleRequestKeyIr, ImportPhaseIr)`
    // occurrences. The parallel phase-free projection coalesces those phase
    // variants for host discovery and resolution only.
    //
    // This is its own pass because neither entry-table pass below can produce
    // source order: ImportEntries must run before ExportEntriesForModule, so
    // collecting requests as a side effect of those passes would put every
    // import ahead of every re-export regardless of how the module was written,
    // and `[[RequestedModules]]` order is what fixes dependency evaluation
    // order for otherwise independent dependencies.
    for item in items {
        let request = match item {
            ModuleItem::ImportDeclaration(import) => module_request(interner, import.request()),
            ModuleItem::ExportDeclaration(export) => match export.as_ref() {
                ExportDeclaration::ReExport { specifier, .. } => {
                    ModuleRequestIr::plain(resolved_name(interner, specifier.sym()))
                }
                _ => continue,
            },
            ModuleItem::StatementListItem(_) => continue,
        };
        push_unique_request_key(&mut record.module_resolution_requests, request.key());
        push_unique_request(&mut record.requested_modules, &request);
    }

    // 16.2.2.3 ImportEntries. Imports are collected before exports because
    // ExportEntriesForModule consults them for `export { imported }`.
    for item in items {
        let ModuleItem::ImportDeclaration(import) = item else {
            continue;
        };
        let request = module_request(interner, import.request());
        // The span of the binding the entry creates. `ImportDeclaration` itself
        // carries no span in boa 0.21.1, and the binding is the useful thing to
        // point a diagnostic at anyway.
        if let Some(default) = import.default() {
            record.import_entries.push(ImportEntryIr {
                request: request.clone(),
                import_name: ImportNameIr::Name(ExportName::default_export()),
                local_name: resolved_local_name(interner, default.sym()),
                span: Some(lines.source_span(text, default.span())),
            });
        }
        match import.kind() {
            ImportKind::DefaultOrUnnamed => {}
            ImportKind::Namespaced { binding } => {
                record.import_entries.push(ImportEntryIr {
                    request: request.clone(),
                    import_name: ImportNameIr::Namespace,
                    local_name: resolved_local_name(interner, binding.sym()),
                    span: Some(lines.source_span(text, binding.span())),
                });
            }
            ImportKind::Named { names } => {
                for name in names.iter() {
                    // `import { a as b }`: `export_name()` is the exporter's D2
                    // and `binding()` is this module's D1. Two domains, two
                    // conversions.
                    record.import_entries.push(ImportEntryIr {
                        request: request.clone(),
                        import_name: ImportNameIr::Name(resolved_export_name(
                            interner,
                            name.export_name(),
                        )),
                        local_name: resolved_local_name(interner, name.binding().sym()),
                        span: Some(lines.source_span(text, name.binding().span())),
                    });
                }
            }
        }
    }

    // 16.2.3.2 ExportEntries.
    let mut default_local: Option<LocalName> = None;
    for item in items {
        let ModuleItem::ExportDeclaration(export) = item else {
            continue;
        };
        match export.as_ref() {
            ExportDeclaration::ReExport { kind, specifier } => {
                // KNOWN GAP, same weight as `ModuleLinkErrorIr::UnsupportedPhase`
                // and tracked alongside it.
                //
                // `ExportDeclaration::ReExport` carries a bare `ModuleSpecifier`
                // rather than a `ModuleRequest`
                // (`vendor/boa_ast-0.21.1/src/declaration/export.rs:85`). boa's
                // parser *does* read `export ... from "m" with { type: "json" }`
                // and then throws the clause away —
                // `parse_ignored_import_attributes`,
                // `vendor/boa_parser-0.21.1/src/parser/statement/declaration/export.rs:226`.
                // The attributes are therefore not in the AST at all and nothing
                // here can recover them.
                //
                // Two things follow, both wrong, both invisible without this
                // note. `ModuleRequestsEqual` sees every re-export request as
                // attribute-less, so a module that both imports and re-exports
                // one specifier with the same attributes records *two* distinct
                // `[[RequestedModules]]` keys; and a host that keys its module
                // map on attributes will resolve the re-export to the wrong
                // module type. Fixing it needs a boa-side change (keep the
                // `ModuleRequest`), not a change here.
                let request = ModuleRequestIr::plain(resolved_name(interner, specifier.sym()));
                match kind {
                    ReExportKind::Namespaced { name: Some(name) } => {
                        record.indirect_export_entries.push(IndirectExportEntryIr {
                            request,
                            import_name: ImportNameIr::Namespace,
                            export_name: resolved_export_name(interner, *name),
                        });
                    }
                    ReExportKind::Namespaced { name: None } => {
                        record
                            .star_export_entries
                            .push(StarExportEntryIr { request });
                    }
                    ReExportKind::Named { names } => {
                        for entry in names.iter() {
                            // Both sides are D2 here. boa calls the first
                            // accessor `private_name()`, but for a re-export it
                            // is the *requested* module's `[[ExportName]]` —
                            // the entry binds nothing locally at all. Contrast
                            // `ExportDeclaration::List` below, where the same
                            // accessor is a `[[LocalName]]`.
                            record.indirect_export_entries.push(IndirectExportEntryIr {
                                request: request.clone(),
                                import_name: ImportNameIr::Name(resolved_export_name(
                                    interner,
                                    entry.private_name(),
                                )),
                                export_name: resolved_export_name(interner, entry.alias()),
                            });
                        }
                    }
                }
            }
            ExportDeclaration::List(names) => {
                for entry in names.iter() {
                    // Here `private_name()` really is a `[[LocalName]]`: the
                    // clause names a binding of *this* module. The two
                    // conversions differ, which is what makes swapping them a
                    // type error at the struct literals below.
                    let local = resolved_local_name(interner, entry.private_name());
                    let export_name = resolved_export_name(interner, entry.alias());
                    // ParseModule step 12. An `export { x }` clause has a null
                    // `[[ModuleRequest]]` at ExportEntries time; ParseModule
                    // then rewrites it against `[[ImportEntries]]`:
                    //
                    // * `x` is not imported     -> local entry (12.a.ii);
                    // * `x` is a namespace      -> *still* a local entry
                    //                              (12.a.iii), because what is
                    //                              exported is this module's own
                    //                              cell holding the namespace
                    //                              object, not a binding of the
                    //                              requested module;
                    // * `x` is a named import   -> indirect entry (12.a.iv),
                    //                              which is what makes the
                    //                              re-exported binding stay live
                    //                              and stay observable through
                    //                              its original module.
                    let imported = record
                        .import_entries
                        .iter()
                        .find(|import| import.local_name == local)
                        .map(|import| (import.request.clone(), import.import_name.clone()));
                    match imported {
                        Some((request, ImportNameIr::Name(import_name))) => {
                            record.indirect_export_entries.push(IndirectExportEntryIr {
                                request,
                                import_name: ImportNameIr::Name(import_name),
                                export_name,
                            });
                        }
                        Some((_, ImportNameIr::Namespace)) | None => {
                            record.local_export_entries.push(LocalExportEntryIr {
                                local_name: local,
                                export_name,
                            });
                        }
                    }
                }
            }
            ExportDeclaration::VarStatement(var) => {
                for name in bound_names(var) {
                    push_local_export(&mut record, &resolved_name(interner, name));
                }
            }
            ExportDeclaration::Declaration(declaration) => {
                for name in bound_names(declaration) {
                    push_local_export(&mut record, &resolved_name(interner, name));
                }
            }
            // The five default *declaration* forms all route through
            // `push_default_export`, which is the one place boa's `default`
            // marker is turned into `LocalName::AnonymousDefault`.
            ExportDeclaration::DefaultFunctionDeclaration(function) => {
                push_default_export(
                    &resolved_name(interner, function.name().sym()),
                    &mut record,
                    &mut default_local,
                );
            }
            ExportDeclaration::DefaultGeneratorDeclaration(generator) => {
                push_default_export(
                    &resolved_name(interner, generator.name().sym()),
                    &mut record,
                    &mut default_local,
                );
            }
            ExportDeclaration::DefaultAsyncFunctionDeclaration(function) => {
                push_default_export(
                    &resolved_name(interner, function.name().sym()),
                    &mut record,
                    &mut default_local,
                );
            }
            ExportDeclaration::DefaultAsyncGeneratorDeclaration(generator) => {
                push_default_export(
                    &resolved_name(interner, generator.name().sym()),
                    &mut record,
                    &mut default_local,
                );
            }
            ExportDeclaration::DefaultClassDeclaration(class) => {
                push_default_export(
                    &resolved_name(interner, class.name().sym()),
                    &mut record,
                    &mut default_local,
                );
            }
            ExportDeclaration::DefaultAssignmentExpression(_) => {
                record.local_export_entries.push(LocalExportEntryIr {
                    local_name: LocalName::AnonymousDefault,
                    export_name: ExportName::default_export(),
                });
                default_local = Some(LocalName::AnonymousDefault);
            }
        }
    }

    record.environment = module_environment(item_list, interner, &record, default_local.as_ref());

    let mut scan = ModuleBodyScan {
        interner: Some(interner),
        text,
        lines: Some(&lines),
        ..ModuleBodyScan::default()
    };
    item_list.visit_with(&mut scan);
    record.has_top_level_await = scan.top_level_await;
    record.import_meta_sites = scan.import_meta_sites;
    // The visitor walks an expression's operands before its neighbours rather
    // than strictly left to right, so source order is restored here instead of
    // assumed. `rewrite_import_meta` needs a non-overlapping ascending list.
    record.import_meta_sites.sort_by_key(|span| span.start);
    record.dynamic_import_sites = scan.dynamic_import_sites;

    record
}

/// `export let x` / `export var x` / `export function x`: the coincident case,
/// where the `[[LocalName]]` and the `[[ExportName]]` happen to be the same
/// text.
///
/// The one `&str` is therefore converted **twice**, through the two different
/// constructors. That is not redundancy — it is the whole point. A single
/// shared conversion no longer type-checks, so the case that used to hide a
/// `local_name`/`export_name` swap now cannot.
fn push_local_export(record: &mut SourceTextModuleRecordIr, name: &str) {
    record.local_export_entries.push(LocalExportEntryIr {
        local_name: LocalName::from_bound_name(name),
        export_name: ExportName::new(name),
    });
}

/// Records the `*default*` export of a default *declaration*.
///
/// boa names an anonymous default declaration `default`, which is a reserved
/// word and therefore unambiguous: any other name is a real
/// `BindingIdentifier`. This is the one place that marker is read, and the one
/// place a `[[LocalName]]` is decided from anything other than BoundNames.
fn push_default_export(
    declared_name: &str,
    record: &mut SourceTextModuleRecordIr,
    default_local: &mut Option<LocalName>,
) {
    let local_name = if declared_name == ExportName::DEFAULT {
        LocalName::AnonymousDefault
    } else {
        LocalName::from_bound_name(declared_name)
    };
    record.local_export_entries.push(LocalExportEntryIr {
        local_name: local_name.clone(),
        export_name: ExportName::default_export(),
    });
    *default_local = Some(local_name);
}

/// 16.2.1.6.4 `InitializeEnvironment`, as a static binding table.
fn module_environment(
    item_list: &boa_ast::ModuleItemList,
    interner: &Interner,
    record: &SourceTextModuleRecordIr,
    default_local: Option<&LocalName>,
) -> Vec<ModuleEnvBindingIr> {
    let items = item_list.items();
    let mut environment = Vec::new();
    let mut declared = BTreeSet::new();

    // Import bindings: immutable, indirect, initialized before evaluation.
    for entry in &record.import_entries {
        if declared.insert(entry.local_name.clone()) {
            environment.push(ModuleEnvBindingIr {
                name: entry.local_name.clone(),
                kind: ModuleBindingKindIr::Import,
                mutable: false,
                initialized_before_evaluation: true,
                in_tdz_until_evaluated: false,
                indirect: Some((entry.request.clone(), entry.import_name.clone())),
            });
        }
    }

    for item in items {
        match item {
            ModuleItem::ImportDeclaration(_) => {}
            ModuleItem::StatementListItem(StatementListItem::Declaration(declaration)) => {
                push_declaration_bindings(
                    declaration.as_ref(),
                    interner,
                    &mut environment,
                    &mut declared,
                );
            }
            ModuleItem::StatementListItem(StatementListItem::Statement(statement)) => {
                if let Statement::Var(var) = statement.as_ref() {
                    push_var_bindings(var, interner, &mut environment, &mut declared);
                }
            }
            ModuleItem::ExportDeclaration(export) => match export.as_ref() {
                ExportDeclaration::VarStatement(var) => {
                    push_var_bindings(var, interner, &mut environment, &mut declared);
                }
                ExportDeclaration::Declaration(declaration) => {
                    push_declaration_bindings(
                        declaration,
                        interner,
                        &mut environment,
                        &mut declared,
                    );
                }
                ExportDeclaration::DefaultFunctionDeclaration(_)
                | ExportDeclaration::DefaultGeneratorDeclaration(_)
                | ExportDeclaration::DefaultAsyncFunctionDeclaration(_)
                | ExportDeclaration::DefaultAsyncGeneratorDeclaration(_) => {
                    push_default_binding(
                        default_local,
                        ModuleBindingKindIr::Function,
                        &mut environment,
                        &mut declared,
                    );
                }
                ExportDeclaration::DefaultClassDeclaration(_) => {
                    push_default_binding(
                        default_local,
                        ModuleBindingKindIr::Class,
                        &mut environment,
                        &mut declared,
                    );
                }
                ExportDeclaration::DefaultAssignmentExpression(_) => {
                    push_default_binding(
                        default_local,
                        ModuleBindingKindIr::Let,
                        &mut environment,
                        &mut declared,
                    );
                }
                ExportDeclaration::List(_) | ExportDeclaration::ReExport { .. } => {}
            },
        }
    }

    // VarDeclaredNames also reaches `var`s nested in blocks and loops of the
    // module body; those are module-environment bindings too.
    let mut nested = BTreeSet::new();
    for name in var_declared_names(item_list) {
        nested.insert(resolved_local_name(interner, name));
    }
    for name in nested {
        if declared.insert(name.clone()) {
            environment.push(ModuleEnvBindingIr {
                name,
                kind: ModuleBindingKindIr::Var,
                mutable: true,
                initialized_before_evaluation: true,
                in_tdz_until_evaluated: false,
                indirect: None,
            });
        }
    }

    environment
}

fn push_declaration_bindings(
    declaration: &Declaration,
    interner: &Interner,
    environment: &mut Vec<ModuleEnvBindingIr>,
    declared: &mut BTreeSet<LocalName>,
) {
    let kind = match declaration {
        Declaration::FunctionDeclaration(_)
        | Declaration::GeneratorDeclaration(_)
        | Declaration::AsyncFunctionDeclaration(_)
        | Declaration::AsyncGeneratorDeclaration(_) => ModuleBindingKindIr::Function,
        Declaration::ClassDeclaration(_) => ModuleBindingKindIr::Class,
        Declaration::Lexical(LexicalDeclaration::Const(_)) => ModuleBindingKindIr::Const,
        Declaration::Lexical(_) => ModuleBindingKindIr::Let,
    };
    for name in bound_names(declaration) {
        let name = resolved_local_name(interner, name);
        if declared.insert(name.clone()) {
            environment.push(ModuleEnvBindingIr {
                name,
                kind,
                mutable: kind != ModuleBindingKindIr::Const,
                initialized_before_evaluation: kind == ModuleBindingKindIr::Function,
                in_tdz_until_evaluated: kind != ModuleBindingKindIr::Function,
                indirect: None,
            });
        }
    }
}

fn push_var_bindings(
    var: &VarDeclaration,
    interner: &Interner,
    environment: &mut Vec<ModuleEnvBindingIr>,
    declared: &mut BTreeSet<LocalName>,
) {
    for name in bound_names(var) {
        let name = resolved_local_name(interner, name);
        if declared.insert(name.clone()) {
            environment.push(ModuleEnvBindingIr {
                name,
                kind: ModuleBindingKindIr::Var,
                mutable: true,
                initialized_before_evaluation: true,
                in_tdz_until_evaluated: false,
                indirect: None,
            });
        }
    }
}

fn push_default_binding(
    default_local: Option<&LocalName>,
    kind: ModuleBindingKindIr,
    environment: &mut Vec<ModuleEnvBindingIr>,
    declared: &mut BTreeSet<LocalName>,
) {
    let Some(name) = default_local else {
        return;
    };
    if declared.insert(name.clone()) {
        environment.push(ModuleEnvBindingIr {
            name: name.clone(),
            kind,
            mutable: true,
            initialized_before_evaluation: kind == ModuleBindingKindIr::Function,
            in_tdz_until_evaluated: kind != ModuleBindingKindIr::Function,
            indirect: None,
        });
    }
}

/// Collects the module-body facts that need a full expression walk.
#[derive(Debug, Default)]
struct ModuleBodyScan<'a> {
    interner: Option<&'a Interner>,
    /// The module source text, needed to turn boa positions into byte offsets.
    text: &'a str,
    lines: Option<&'a LineIndex>,
    top_level_await: bool,
    import_meta_sites: Vec<SourceSpan>,
    dynamic_import_sites: Vec<DynamicImportSiteIr>,
    function_depth: usize,
}

/// Statically recovers the request attributes of the literal subset whose
/// final own properties are fixed by syntax alone.
///
/// This is only a graph-discovery optimization. The generated dispatcher
/// always performs the real `Get(options, "with")`, enumerable-own-property
/// walk, and value reads at runtime, so getters, Proxies, computed names and
/// spreads retain their observable behaviour.
fn dynamic_import_attributes(
    node: &ImportCall,
    interner: Option<&Interner>,
) -> DynamicImportAttributesIr {
    let Some(options) = node.options() else {
        return DynamicImportAttributesIr::Known(ModuleRequestAttributesIr::empty());
    };
    let (Expression::ObjectLiteral(options), Some(interner)) = (options, interner) else {
        return DynamicImportAttributesIr::Runtime;
    };

    let mut with_value = None;
    for property in options.properties() {
        let PropertyDefinition::Property(PropertyName::Literal(name), value) = property else {
            return DynamicImportAttributesIr::Runtime;
        };
        let key = resolved_name(interner, name.sym());
        if key == "__proto__" {
            // This changes the options object's prototype. An inherited
            // `with` property is observable through Get(options, "with"), so
            // even a fully literal prototype belongs to the runtime case.
            return DynamicImportAttributesIr::Runtime;
        }
        if key == "with" {
            // Object-literal data properties overwrite an earlier property of
            // the same name, so the last one is what `Get(options, "with")`
            // observes.
            with_value = Some(value);
        }
    }

    let Some(with_value) = with_value else {
        return DynamicImportAttributesIr::Known(ModuleRequestAttributesIr::empty());
    };
    let Expression::ObjectLiteral(with_object) = with_value else {
        return DynamicImportAttributesIr::Runtime;
    };

    let mut attributes: Vec<ImportAttributeIr> = Vec::new();
    for property in with_object.properties() {
        let PropertyDefinition::Property(PropertyName::Literal(name), value) = property else {
            return DynamicImportAttributesIr::Runtime;
        };
        let key = resolved_name(interner, name.sym());
        if key == "__proto__" {
            // `__proto__: value` is the object-literal prototype setter, not
            // an own data property, so EnumerableOwnProperties never reports
            // it. A computed `["__proto__"]` is an ordinary property, but the
            // conservative computed-name arm above classifies that as Runtime.
            continue;
        }
        let Expression::Literal(value) = value else {
            return DynamicImportAttributesIr::Runtime;
        };
        let LiteralKind::String(value) = value.kind() else {
            return DynamicImportAttributesIr::Runtime;
        };
        let value = resolved_name(interner, *value);
        if let Some(attribute) = attributes.iter_mut().find(|attribute| attribute.key == key) {
            // As above, a later object-literal data property overwrites the
            // earlier value before EnumerableOwnProperties observes it.
            attribute.value = value;
        } else {
            attributes.push(ImportAttributeIr { key, value });
        }
    }
    let attributes = ModuleRequestAttributesIr::try_new(attributes)
        .expect("object-literal property replacement leaves unique attribute keys");
    DynamicImportAttributesIr::Known(attributes)
}

impl<'ast> Visitor<'ast> for ModuleBodyScan<'_> {
    type BreakTy = core::convert::Infallible;

    fn visit_import_meta(&mut self, node: &'ast ImportMeta) -> ControlFlow<Self::BreakTy> {
        // boa spans the whole `import.meta` meta-property, from the `import`
        // keyword to the end of `meta`
        // (`vendor/boa_parser-0.21.1/src/parser/expression/left_hand_side/member.rs:112`),
        // so the span is exactly the text the linker has to replace — including
        // any whitespace or comment written between the two tokens.
        if let Some(lines) = self.lines {
            self.import_meta_sites
                .push(lines.source_span(self.text, node.span()));
        }
        ControlFlow::Continue(())
    }

    fn visit_import_call(&mut self, node: &'ast ImportCall) -> ControlFlow<Self::BreakTy> {
        let static_specifier = match node.argument() {
            Expression::Literal(literal) => match (literal.kind(), self.interner) {
                (LiteralKind::String(sym), Some(interner)) => Some(resolved_name(interner, *sym)),
                _ => None,
            },
            _ => None,
        };
        self.dynamic_import_sites.push(DynamicImportSiteIr {
            static_specifier,
            phase: ImportPhaseIr::from_ast(node.phase()),
            attributes: dynamic_import_attributes(node, self.interner),
        });
        node.visit_with(self)
    }

    fn visit_await(
        &mut self,
        node: &'ast boa_ast::expression::Await,
    ) -> ControlFlow<Self::BreakTy> {
        if self.function_depth == 0 {
            self.top_level_await = true;
        }
        node.visit_with(self)
    }

    /// `for await (... of ...)` is the other half of `[[HasTLA]]`.
    ///
    /// It is *not* an `Await` expression in the AST — boa records it as a flag
    /// on the loop — so the `visit_await` arm above cannot see it and a module
    /// whose only top-level `await` is a `for await` would otherwise be
    /// reported as having none.
    fn visit_for_of_loop(&mut self, node: &'ast ForOfLoop) -> ControlFlow<Self::BreakTy> {
        if self.function_depth == 0 && node.r#await() {
            self.top_level_await = true;
        }
        node.visit_with(self)
    }

    fn visit_function_body(&mut self, node: &'ast FunctionBody) -> ControlFlow<Self::BreakTy> {
        self.function_depth += 1;
        let result = node.visit_with(self);
        self.function_depth -= 1;
        result
    }
}

/// Projects `[[RequestedModules]]` to phase-free host-resolution keys and adds
/// string-literal `import(...)` keys, without lowering anything.
///
/// This is what the host loader driver calls to discover a module's
/// dependencies before the graph is assembled.
pub fn scan_module_requests(source: &ParsedModule) -> Vec<ModuleRequestKeyIr> {
    let record = source.with_compiler_session(|module, interner| {
        build_module_record(
            module,
            interner,
            source.source(),
            0,
            ModuleKey::from_host(ANONYMOUS_MODULE_KEY),
        )
    });
    requests_with_dynamic_imports(
        record.module_resolution_requests,
        &record.dynamic_import_sites,
    )
}

/// Reads the statically resolvable `import(...)` requests in a Script without
/// changing its parse goal.
///
/// A Script has no `[[RequestedModules]]` table, so only dynamic imports can
/// contribute. This exists specifically so graph discovery never reparses a
/// Script as Module code (which would both violate parse-once and reject legal
/// sloppy Script forms).
pub fn scan_script_module_requests(source: &ParsedScript) -> Vec<ModuleRequestKeyIr> {
    let sites = script_dynamic_import_sites(source);
    requests_with_dynamic_imports(Vec::new(), &sites)
}

pub(crate) fn script_entry_record(
    source: &ParsedScript,
    id: ModuleUnitId,
    key: ModuleKey,
) -> SourceTextModuleRecordIr {
    SourceTextModuleRecordIr {
        id,
        key,
        source_len: source.source_text.len(),
        has_top_level_await: false,
        requested_modules: Vec::new(),
        module_resolution_requests: Vec::new(),
        import_entries: Vec::new(),
        local_export_entries: Vec::new(),
        indirect_export_entries: Vec::new(),
        star_export_entries: Vec::new(),
        environment: Vec::new(),
        import_meta_sites: Vec::new(),
        dynamic_import_sites: script_dynamic_import_sites(source),
    }
}

pub(super) fn script_dynamic_import_sites(source: &ParsedScript) -> Vec<DynamicImportSiteIr> {
    source.with_compiler_session(|script, interner| {
        let mut scan = ModuleBodyScan {
            interner: Some(interner),
            text: &source.source_text,
            ..ModuleBodyScan::default()
        };
        let _ = script.visit_with(&mut scan);
        scan.dynamic_import_sites
    })
}

fn requests_with_dynamic_imports(
    mut requests: Vec<ModuleRequestKeyIr>,
    sites: &[DynamicImportSiteIr],
) -> Vec<ModuleRequestKeyIr> {
    for site in sites {
        let Some(request) = site.discovery_request() else {
            continue;
        };
        push_unique_request_key(&mut requests, request.key());
    }
    requests
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source_unit(source: &str) -> ParsedModule {
        let parsed = lila_front::parse(source, lila_front::ParseOptions::module())
            .expect("module fixture should parse");
        let ParsedSource::Module(module) = parsed else {
            unreachable!("module options produce a module")
        };
        module
    }

    fn record_of(source: &str) -> SourceTextModuleRecordIr {
        parse_module_record(&source_unit(source), 0, ModuleKey::from_host("main.mjs"))
            .expect("module record should build")
    }

    /// The entry tables are compared without spans everywhere except the one
    /// test that is *about* spans, so a change in boa's position reporting
    /// cannot fail nine unrelated assertions.
    fn without_spans(entries: &[ImportEntryIr]) -> Vec<ImportEntryIr> {
        entries
            .iter()
            .map(|entry| ImportEntryIr {
                span: None,
                ..entry.clone()
            })
            .collect()
    }

    /// A `[[LocalName]]`, spelled the way the module's own text spells it.
    fn local(name: &str) -> LocalName {
        LocalName::from_bound_name(name)
    }

    fn import(specifier: &str, import_name: ImportNameIr, local_name: &str) -> ImportEntryIr {
        ImportEntryIr {
            request: ModuleRequestIr::plain(specifier),
            import_name,
            local_name: local(local_name),
            span: None,
        }
    }

    fn named(name: &str) -> ImportNameIr {
        ImportNameIr::Name(ExportName::new(name))
    }

    /// The compile-time witness for the coincident case.
    ///
    /// It used to take two `&str`s, which made the *field initialisers* inside
    /// it unswappable while leaving every **call site** free to swap:
    /// `local_export("x", "y")` and `local_export("y", "x")` both compiled. The
    /// parameters are typed now, so the swap is `E0308` where it would actually
    /// be written.
    fn local_export(local_name: LocalName, export_name: ExportName) -> LocalExportEntryIr {
        LocalExportEntryIr {
            local_name,
            export_name,
        }
    }

    fn indirect_export(
        specifier: &str,
        import_name: ImportNameIr,
        export_name: &str,
    ) -> IndirectExportEntryIr {
        IndirectExportEntryIr {
            request: ModuleRequestIr::plain(specifier),
            import_name,
            export_name: ExportName::new(export_name),
        }
    }

    fn attribute(key: &str, value: &str) -> ImportAttributeIr {
        ImportAttributeIr {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    fn attributes(attributes: Vec<ImportAttributeIr>) -> ModuleRequestAttributesIr {
        ModuleRequestAttributesIr::try_new(attributes).expect("test attributes are unique")
    }

    fn request_key(specifier: &str) -> ModuleRequestKeyIr {
        ModuleRequestKeyIr::plain(specifier)
    }

    fn attributed_key(
        specifier: &str,
        request_attributes: Vec<ImportAttributeIr>,
    ) -> ModuleRequestKeyIr {
        ModuleRequestKeyIr::try_new(specifier, request_attributes)
            .expect("test attributes are unique")
    }

    fn attributed_request(
        specifier: &str,
        phase: ImportPhaseIr,
        request_attributes: Vec<ImportAttributeIr>,
    ) -> ModuleRequestIr {
        ModuleRequestIr::try_new(specifier, phase, request_attributes)
            .expect("test attributes are unique")
    }

    fn known_attributes(attributes: Vec<ImportAttributeIr>) -> DynamicImportAttributesIr {
        DynamicImportAttributesIr::Known(self::attributes(attributes))
    }

    // -- 16.2.2.3 ImportEntries -------------------------------------------

    #[test]
    fn import_entries_cover_default_named_and_namespace_clauses() {
        let record = record_of(
            "import alpha, { beta, gamma as delta } from \"./m.mjs\";\n\
             import * as epsilon from \"./n.mjs\";\n\
             import \"./side.mjs\";\n",
        );

        assert_eq!(
            without_spans(&record.import_entries),
            vec![
                // ImportEntriesForModule emits the default binding before the
                // named ones of the same clause.
                import("./m.mjs", named(MODULE_DEFAULT_EXPORT_NAME), "alpha"),
                import("./m.mjs", named("beta"), "beta"),
                import("./m.mjs", named("gamma"), "delta"),
                import("./n.mjs", ImportNameIr::Namespace, "epsilon"),
            ]
        );
        // `import "./side.mjs"` binds nothing but is still a requested module.
        assert_eq!(
            record.requested_modules,
            vec![
                ModuleRequestIr::plain("./m.mjs"),
                ModuleRequestIr::plain("./n.mjs"),
                ModuleRequestIr::plain("./side.mjs"),
            ]
        );
    }

    #[test]
    fn import_entry_spans_point_at_the_local_binding() {
        let source = "import alpha from \"./m.mjs\";\nimport { beta as gamma } from \"./n.mjs\";\n";
        let record = record_of(source);

        let expected: Vec<Option<SourceSpan>> = ["alpha", "gamma"]
            .iter()
            .map(|name| {
                let start = source.find(name).expect("binding appears in the source");
                Some(SourceSpan {
                    start,
                    end: start + name.len(),
                })
            })
            .collect();
        let actual: Vec<Option<SourceSpan>> = record
            .import_entries
            .iter()
            .map(|entry| entry.span)
            .collect();
        assert_eq!(actual, expected);
    }

    // -- 16.2.2.2 ModuleRequests ------------------------------------------

    #[test]
    fn requested_modules_dedup_by_key_within_one_phase() {
        let record = record_of(
            "import { alpha } from \"./m.mjs\" with { type: \"json\" };\n\
             import { beta } from \"./m.mjs\" with { type: \"json\" };\n\
             import { gamma } from \"./m.mjs\";\n",
        );

        // ModuleRequestsEqual compares specifier and attributes. Within this
        // shared evaluation phase the two attributed requests collapse and
        // the bare key does not.
        assert_eq!(
            record.requested_modules,
            vec![
                attributed_request(
                    "./m.mjs",
                    ImportPhaseIr::Evaluation,
                    vec![attribute("type", "json")],
                ),
                ModuleRequestIr::plain("./m.mjs"),
            ]
        );
    }

    #[test]
    fn request_attributes_are_sorted_so_keys_implement_module_requests_equal() {
        let first = record_of("import \"./m.mjs\" with { type: \"json\", charset: \"utf8\" };\n");
        let second = record_of("import \"./m.mjs\" with { charset: \"utf8\", type: \"json\" };\n");

        assert_eq!(
            first.requested_modules[0].attributes(),
            attributes(vec![
                attribute("charset", "utf8"),
                attribute("type", "json")
            ])
            .as_slice()
        );
        assert_eq!(
            first.requested_modules[0].key(),
            second.requested_modules[0].key()
        );

        // U+10000 starts with UTF-16 code unit D800, which sorts before E000
        // even though Rust's scalar-value String ordering puts it after.
        let utf16 = record_of(
            "import \"./m.mjs\" with { \"\u{10000}\": \"astral\", \"\u{e000}\": \"bmp\" };\n",
        );
        assert_eq!(
            utf16.requested_modules[0].attributes(),
            attributes(vec![
                attribute("\u{10000}", "astral"),
                attribute("\u{e000}", "bmp")
            ])
            .as_slice()
        );
    }

    #[test]
    fn request_key_constructor_canonicalizes_reverse_utf16_attribute_order() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Rust String ordering puts E000 before U+10000, while ECMAScript
        // UTF-16 ordering sees D800 first and therefore puts U+10000 first.
        let scalar_order = vec![
            attribute("\u{e000}", "bmp"),
            attribute("\u{10000}", "astral"),
        ];
        let utf16_order = vec![
            attribute("\u{10000}", "astral"),
            attribute("\u{e000}", "bmp"),
        ];
        let first = attributed_key("./m.mjs", scalar_order);
        let second = attributed_key("./m.mjs", utf16_order.clone());

        assert_eq!(first, second);
        assert_eq!(first.cmp(&second), core::cmp::Ordering::Equal);
        assert_eq!(first.attributes(), utf16_order.as_slice());
        let mut first_hash = DefaultHasher::new();
        first.hash(&mut first_hash);
        let mut second_hash = DefaultHasher::new();
        second.hash(&mut second_hash);
        assert_eq!(first_hash.finish(), second_hash.finish());

        let evaluation = ModuleRequestIr::from_key(first.clone(), ImportPhaseIr::Evaluation);
        let source = ModuleRequestIr::from_key(first, ImportPhaseIr::Source);
        assert_ne!(evaluation, source, "occurrences retain their phase");
        assert_eq!(
            evaluation.key(),
            source.key(),
            "host identity ignores phase"
        );
    }

    #[test]
    fn request_attributes_reject_duplicate_keys() {
        let error = ModuleRequestAttributesIr::try_new(vec![
            attribute("type", "json"),
            attribute("type", "css"),
        ])
        .expect_err("duplicate keys cannot inhabit the canonical list");

        assert_eq!(error.key(), "type");
    }

    #[test]
    fn requested_modules_follow_source_order_across_item_kinds() {
        // A re-export written before an import must load and evaluate first,
        // which only holds if ModuleRequests is one source-order pass.
        let record = record_of("export * from \"./first.mjs\";\nimport \"./second.mjs\";\n");
        assert_eq!(
            record.requested_modules,
            vec![
                ModuleRequestIr::plain("./first.mjs"),
                ModuleRequestIr::plain("./second.mjs"),
            ]
        );
    }

    #[test]
    fn requested_modules_retain_phases_while_resolution_requests_coalesce_them() {
        let record = record_of(
            "import './m.mjs';\n\
             import './m.mjs';\n\
             import defer * as deferred from './m.mjs';\n\
             import source artifact from './m.mjs';\n",
        );

        assert_eq!(
            record
                .requested_modules
                .iter()
                .map(ModuleRequestIr::phase)
                .collect::<Vec<_>>(),
            vec![
                ImportPhaseIr::Evaluation,
                ImportPhaseIr::Defer,
                ImportPhaseIr::Source,
            ]
        );
        assert!(record
            .requested_modules
            .iter()
            .all(|request| request.key() == &record.module_resolution_requests[0]));
        assert_eq!(
            record.module_resolution_requests,
            vec![request_key("./m.mjs")]
        );
    }

    #[test]
    fn requested_modules_keep_phaseful_order_after_an_earlier_key_projection() {
        let record = record_of(
            "import source artifact from './m.mjs';\n\
             import './n.mjs';\n\
             import './m.mjs';\n\
             artifact;",
        );

        assert_eq!(
            record.requested_modules,
            vec![
                ModuleRequestIr::from_key(request_key("./m.mjs"), ImportPhaseIr::Source,),
                ModuleRequestIr::plain("./n.mjs"),
                ModuleRequestIr::plain("./m.mjs"),
            ]
        );
        assert_eq!(
            record.module_resolution_requests,
            vec![request_key("./m.mjs"), request_key("./n.mjs")]
        );
    }

    // -- 16.2.3.2 ExportEntries / 16.2.3.3 ExportEntriesForModule ---------

    #[test]
    fn export_entries_split_into_local_indirect_and_star_tables() {
        let record = record_of(
            "import { imported } from \"./imp.mjs\";\n\
             import * as namespaced from \"./nsimp.mjs\";\n\
             const thing = 1;\n\
             export { thing };\n\
             export { thing as renamed };\n\
             export var counted = 2;\n\
             export function fn() {}\n\
             export * from \"./star.mjs\";\n\
             export * as ns from \"./nsmod.mjs\";\n\
             export { other as aliased } from \"./re.mjs\";\n\
             export { imported };\n\
             export { namespaced };\n",
        );

        assert_eq!(
            record.local_export_entries,
            vec![
                local_export(local("thing"), ExportName::new("thing")),
                local_export(local("thing"), ExportName::new("renamed")),
                local_export(local("counted"), ExportName::new("counted")),
                local_export(local("fn"), ExportName::new("fn")),
                // ParseModule 12.a.iii: re-exporting a namespace *import* is a
                // local entry, because the exported cell is this module's own.
                local_export(local("namespaced"), ExportName::new("namespaced")),
            ]
        );
        assert_eq!(
            record.indirect_export_entries,
            vec![
                indirect_export("./nsmod.mjs", ImportNameIr::Namespace, "ns"),
                indirect_export("./re.mjs", named("other"), "aliased"),
                // ParseModule 12.a.iv: `export { imported }` where `imported` is
                // a *named* import is an indirect entry, not a local one.
                indirect_export("./imp.mjs", named("imported"), "imported"),
            ]
        );
        assert_eq!(
            record.star_export_entries,
            vec![StarExportEntryIr {
                request: ModuleRequestIr::plain("./star.mjs"),
            }]
        );
    }

    #[test]
    fn exporting_a_default_import_is_indirect_on_the_name_default() {
        let record = record_of("import alpha from \"./m.mjs\";\nexport { alpha as beta };\n");

        assert_eq!(record.local_export_entries, Vec::new());
        assert_eq!(
            record.indirect_export_entries,
            vec![indirect_export(
                "./m.mjs",
                named(MODULE_DEFAULT_EXPORT_NAME),
                "beta"
            )]
        );
    }

    // -- 16.2.3.7 `export default` ----------------------------------------

    #[test]
    fn named_default_declarations_keep_their_own_local_name() {
        for source in [
            "export default function alpha() {}",
            "export default function* alpha() {}",
            "export default async function alpha() {}",
            "export default async function* alpha() {}",
            "export default class alpha {}",
        ] {
            let record = record_of(source);
            assert_eq!(
                record.local_export_entries,
                vec![local_export(
                    local("alpha"),
                    ExportName::new(MODULE_DEFAULT_EXPORT_NAME)
                )],
                "{source}"
            );
        }
    }

    #[test]
    fn anonymous_default_forms_bind_the_spec_name_and_rewrite_nothing() {
        for (source, kind) in [
            (
                "export default function () {}",
                ModuleBindingKindIr::Function,
            ),
            (
                "export default function* () {}",
                ModuleBindingKindIr::Function,
            ),
            (
                "export default async function () {}",
                ModuleBindingKindIr::Function,
            ),
            (
                "export default async function* () {}",
                ModuleBindingKindIr::Function,
            ),
            ("export default class {}", ModuleBindingKindIr::Class),
            ("export default 1 + 1;", ModuleBindingKindIr::Let),
        ] {
            let record = record_of(source);
            assert_eq!(
                record.local_export_entries,
                vec![local_export(
                    local(MODULE_ANONYMOUS_DEFAULT_LOCAL_NAME),
                    ExportName::new(MODULE_DEFAULT_EXPORT_NAME)
                )],
                "{source}"
            );
            assert_eq!(
                record.environment,
                vec![ModuleEnvBindingIr {
                    name: LocalName::AnonymousDefault,
                    kind,
                    mutable: true,
                    initialized_before_evaluation: kind == ModuleBindingKindIr::Function,
                    in_tdz_until_evaluated: kind != ModuleBindingKindIr::Function,
                    indirect: None,
                }],
                "{source}"
            );
            // The record never rewrites source text; the body keeps its exact
            // byte length so span-derived `FunctionId`s stay valid.
            assert_eq!(record.source_len, source.len(), "{source}");
        }
    }

    // -- 16.2.1.6.4 InitializeEnvironment ---------------------------------

    #[test]
    fn module_environment_records_kinds_mutability_and_tdz() {
        let record = record_of(
            "import { imported } from \"./m.mjs\";\n\
             var v = 1;\n\
             let l = 2;\n\
             const c = 3;\n\
             function f() {}\n\
             class K {}\n\
             { var nested = 4; }\n",
        );

        assert_eq!(
            record.environment,
            vec![
                ModuleEnvBindingIr {
                    name: local("imported"),
                    kind: ModuleBindingKindIr::Import,
                    mutable: false,
                    initialized_before_evaluation: true,
                    in_tdz_until_evaluated: false,
                    indirect: Some((ModuleRequestIr::plain("./m.mjs"), named("imported"))),
                },
                ModuleEnvBindingIr {
                    name: local("v"),
                    kind: ModuleBindingKindIr::Var,
                    mutable: true,
                    initialized_before_evaluation: true,
                    in_tdz_until_evaluated: false,
                    indirect: None,
                },
                ModuleEnvBindingIr {
                    name: local("l"),
                    kind: ModuleBindingKindIr::Let,
                    mutable: true,
                    initialized_before_evaluation: false,
                    in_tdz_until_evaluated: true,
                    indirect: None,
                },
                ModuleEnvBindingIr {
                    name: local("c"),
                    kind: ModuleBindingKindIr::Const,
                    mutable: false,
                    initialized_before_evaluation: false,
                    in_tdz_until_evaluated: true,
                    indirect: None,
                },
                ModuleEnvBindingIr {
                    name: local("f"),
                    kind: ModuleBindingKindIr::Function,
                    mutable: true,
                    initialized_before_evaluation: true,
                    in_tdz_until_evaluated: false,
                    indirect: None,
                },
                ModuleEnvBindingIr {
                    name: local("K"),
                    kind: ModuleBindingKindIr::Class,
                    mutable: true,
                    initialized_before_evaluation: false,
                    in_tdz_until_evaluated: true,
                    indirect: None,
                },
                // VarDeclaredNames reaches `var`s nested in blocks; they are
                // module-environment bindings too, and they are appended after
                // the source-order walk.
                ModuleEnvBindingIr {
                    name: local("nested"),
                    kind: ModuleBindingKindIr::Var,
                    mutable: true,
                    initialized_before_evaluation: true,
                    in_tdz_until_evaluated: false,
                    indirect: None,
                },
            ]
        );
    }

    // -- `[[HasTLA]]` ------------------------------------------------------

    #[test]
    fn top_level_await_sets_has_tla_rather_than_failing_parse_module() {
        for source in [
            "await Promise.resolve(1);\n",
            "for await (const value of []) { value; }\n",
        ] {
            let record =
                parse_module_record(&source_unit(source), 0, ModuleKey::from_host("main.mjs"))
                    .unwrap_or_else(|diagnostics| {
                        panic!("top-level await must parse, got {diagnostics:?} for {source}")
                    });
            assert!(record.has_top_level_await, "{source}");
        }
    }

    #[test]
    fn await_inside_a_function_is_not_top_level_await() {
        let record = record_of(
            "async function alpha() { await 1; }\n\
             const beta = async () => { for await (const v of []) { v; } };\n",
        );
        assert!(!record.has_top_level_await);
    }

    // -- `import.meta` and `import()` --------------------------------------

    #[test]
    fn import_meta_and_dynamic_import_sites_are_collected() {
        let record = record_of(
            "import.meta.url;\n\
             const alpha = import(\"./static.mjs\");\n\
             const beta = import(computed);\n\
             function inner() { return import(\"./nested.mjs\"); }\n\
             import.meta.resolve;\n",
        );

        assert_eq!(record.import_meta_uses(), 2);
        assert_eq!(
            record.dynamic_import_sites,
            vec![
                DynamicImportSiteIr {
                    static_specifier: Some("./static.mjs".to_string()),
                    phase: ImportPhaseIr::Evaluation,
                    attributes: known_attributes(Vec::new()),
                },
                // A computed specifier is not statically discoverable, so the
                // target cannot be compiled into the artifact.
                DynamicImportSiteIr {
                    static_specifier: None,
                    phase: ImportPhaseIr::Evaluation,
                    attributes: known_attributes(Vec::new()),
                },
                // Nesting inside a function does not hide the site: the whole
                // point is that the target is reachable at run time.
                DynamicImportSiteIr {
                    static_specifier: Some("./nested.mjs".to_string()),
                    phase: ImportPhaseIr::Evaluation,
                    attributes: known_attributes(Vec::new()),
                },
            ]
        );
    }

    fn record_with_id(source: &str, id: ModuleUnitId) -> SourceTextModuleRecordIr {
        parse_module_record(&source_unit(source), id, ModuleKey::from_host("main.mjs"))
            .expect("module record should build")
    }

    /// Every slice a site addresses, so a test can say what was found without
    /// hard-coding byte offsets.
    fn site_slices<'a>(source: &'a str, record: &SourceTextModuleRecordIr) -> Vec<&'a str> {
        record
            .import_meta_sites
            .iter()
            .map(|site| &source[site.start..site.end])
            .collect()
    }

    #[test]
    fn import_meta_sites_span_the_whole_meta_property() {
        let source = "print(import.meta.url);\nconst m = import.meta;\n";
        let record = record_of(source);
        assert_eq!(
            site_slices(source, &record),
            vec!["import.meta", "import.meta"]
        );
    }

    #[test]
    fn an_import_meta_written_across_lines_or_around_a_comment_is_one_site() {
        // The meta-property is two tokens, so anything that can sit between two
        // tokens can sit inside it — and all of it has to be replaced.
        let source = "const a = import\n  . /* here */ meta;\n";
        let record = record_of(source);
        assert_eq!(
            site_slices(source, &record),
            vec!["import\n  . /* here */ meta"]
        );
    }

    #[test]
    fn a_module_without_import_meta_declares_no_object() {
        let source = "const value = 1;\n";
        let record = record_of(source);
        assert_eq!(record.import_meta_uses(), 0);
        // Nothing to rewrite, and the text comes back untouched rather than
        // merely equivalent.
        assert_eq!(rewrite_import_meta(source, &record).as_deref(), Ok(source));
    }

    #[test]
    fn rewriting_replaces_every_site_with_this_units_binding() {
        let source = "print(import.meta.url);\nconst m = import.meta;\n";
        let record = record_with_id(source, 7);
        let rewritten = rewrite_import_meta(source, &record).expect("rewrite should succeed");

        let name = MergedName::minted(7, UnitCellRole::ImportMeta);
        assert_eq!(name.as_str(), "$m7$meta");
        assert!(
            !rewritten.contains("import"),
            "no `import.meta` may survive into Script text, got: {rewritten}"
        );
        assert_eq!(rewritten.matches(name.as_str()).count(), 2, "{rewritten}");
        // The rest of the body is untouched.
        assert!(rewritten.contains(".url);"), "{rewritten}");
    }

    #[test]
    fn two_units_get_two_distinct_import_meta_objects() {
        assert_ne!(
            MergedName::minted(0, UnitCellRole::ImportMeta),
            MergedName::minted(1, UnitCellRole::ImportMeta)
        );
        assert_ne!(
            import_meta_binding(0, "file:///a.mjs").declaration,
            import_meta_binding(1, "file:///a.mjs").declaration
        );
    }

    #[test]
    fn rewriting_preserves_byte_length_and_line_structure() {
        for source in [
            "print(import.meta.url);\n",
            "const a = import\n  . /* here */ meta;\n",
            "function f() { return import.meta; }\nf();\n",
        ] {
            let record = record_of(source);
            let rewritten = rewrite_import_meta(source, &record).expect("rewrite should succeed");
            assert_eq!(rewritten.len(), source.len(), "{source}");
            assert_eq!(
                rewritten.matches('\n').count(),
                source.matches('\n').count(),
                "{source}"
            );
        }
    }

    #[test]
    fn the_declaration_is_a_null_prototype_object_carrying_url() {
        let binding = import_meta_binding(0, "file:///root/a.mjs");
        assert_eq!(
            binding.declaration,
            "const $m0$meta = { __proto__: null, url: \"file:///root/a.mjs\" };"
        );
        assert_eq!(
            binding.name,
            MergedName::minted(0, UnitCellRole::ImportMeta)
        );
    }

    #[test]
    fn a_url_that_needs_escaping_stays_one_string_literal() {
        let binding = import_meta_binding(0, "file:///a\"b\\c\u{2028}d\n");
        assert_eq!(
            binding.declaration,
            "const $m0$meta = { __proto__: null, url: \"file:///a\\\"b\\\\c\\u2028d\\n\" };"
        );
        // Whatever the host handed over, the declaration is still one line.
        assert!(!binding.declaration.contains('\n'));
    }

    #[test]
    fn a_site_that_does_not_address_this_source_is_refused_not_applied() {
        let source = "print(import.meta.url);\n";
        let mut record = record_of(source);
        record.import_meta_sites = vec![SourceSpan { start: 0, end: 11 }];
        let error =
            rewrite_import_meta(source, &record).expect_err("a bogus span must be reported");
        assert!(error.reason.contains("covers"), "{}", error.reason);
    }

    #[test]
    fn scan_module_requests_reports_static_import_call_targets() {
        let requests = scan_module_requests(&source_unit(
            "import \"./static.mjs\";\nconst alpha = import(\"./dynamic.mjs\");\n",
        ));
        assert_eq!(
            requests,
            vec![request_key("./static.mjs"), request_key("./dynamic.mjs"),]
        );
    }

    #[test]
    fn dynamic_import_literal_attributes_reach_module_requests() {
        let source = source_unit(
            "import('./known.mjs', { with: { type: 'json', charset: 'utf8' } });\n\
             import('./runtime.mjs', options);\n\
             import('./proto.mjs', { with: { __proto__: 'not-an-attribute' } });\n\
             import('./inherited.mjs', { __proto__: { with: { type: 'json' } } });\n",
        );
        let record = parse_module_record(&source, 0, ModuleKey::from_host("main.mjs"))
            .expect("module record should build");

        assert_eq!(
            record.dynamic_import_sites[0].attributes,
            known_attributes(vec![
                attribute("charset", "utf8"),
                attribute("type", "json"),
            ])
        );
        assert_eq!(
            record.dynamic_import_sites[1].attributes,
            DynamicImportAttributesIr::Runtime
        );
        assert_eq!(
            record.dynamic_import_sites[2].attributes,
            known_attributes(Vec::new())
        );
        assert_eq!(
            record.dynamic_import_sites[3].attributes,
            DynamicImportAttributesIr::Runtime
        );
        assert_eq!(
            scan_module_requests(&source),
            vec![
                attributed_key(
                    "./known.mjs",
                    vec![attribute("charset", "utf8"), attribute("type", "json")],
                ),
                // A runtime options shape discovers the attribute-free request
                // as the AOT registry's safe baseline. Runtime matching still
                // requires the eventual attribute list to be exactly empty.
                request_key("./runtime.mjs"),
                request_key("./proto.mjs"),
                request_key("./inherited.mjs"),
            ]
        );
    }

    // -- position mapping ---------------------------------------------------

    #[test]
    fn line_index_maps_positions_to_byte_offsets() {
        // A CRLF break, a multi-byte character, then an LF break: columns count
        // code points, offsets count bytes.
        let source = "let a = 1;\r\nlet \u{e9} = 2;\nlet c = 3;";
        let lines = LineIndex::new(source);

        assert_eq!(lines.offset(source, Position::new(1, 1)), 0);
        // "\r\n" is one line break, so line 2 starts after both bytes.
        assert_eq!(lines.offset(source, Position::new(2, 1)), 12);
        assert_eq!(lines.offset(source, Position::new(2, 5)), 16);
        // The character before this one is two bytes wide.
        assert_eq!(lines.offset(source, Position::new(2, 6)), 18);
        assert_eq!(lines.offset(source, Position::new(3, 1)), 24);
        // A `Span` end sits one past its last character; it must clamp to the
        // end of the line's content rather than swallow the line break.
        assert_eq!(lines.offset(source, Position::new(3, 11)), source.len());
    }
}
