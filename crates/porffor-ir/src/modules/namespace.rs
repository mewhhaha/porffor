//! Module namespace exotic objects (10.4.6, `ModuleNamespaceCreate` 16.2.1.10).
//!
//! A namespace object is a static table: every export name the module provides,
//! paired with the binding that backs it. Reads are live because the binding is
//! the exporter's own; writes are rejected; the key order is fixed at compile
//! time.
//!
//! # What 10.4.6 asks for
//!
//! 10.4.6 makes a namespace object unlike any ordinary object, and every one of
//! its invariants is a compile-time constant here rather than runtime state:
//!
//! * `[[GetPrototypeOf]]` is `null` and `[[SetPrototypeOf]]` only succeeds for
//!   `null` ([`ModuleNamespaceIr::PROTOTYPE_IS_NULL`]);
//! * `[[IsExtensible]]` is `false` and `[[PreventExtensions]]` is a no-op that
//!   succeeds ([`ModuleNamespaceIr::EXTENSIBLE`]);
//! * `@@toStringTag` is the non-writable, non-enumerable, non-configurable
//!   string `"Module"` ([`ModuleNamespaceIr::TO_STRING_TAG`]);
//! * every export is a writable, enumerable, non-configurable *data* property
//!   whose value is read through the exporter's binding at access time, so the
//!   binding is live, while `[[Set]]`, `[[Delete]]` and `[[DefineOwnProperty]]`
//!   all fail;
//! * `[[OwnPropertyKeys]]` is [`ModuleNamespaceIr::exports`] in UTF-16
//!   code-unit order, then `@@toStringTag`.
//!
//! Identity is cached in one cell per module ([`ModuleNamespaceIr::cell`]), so
//! repeated `import * as ns` and repeated `import()` of the same module observe
//! the same object.
//!
//! # How one is actually materialized
//!
//! [`link`] merges a graph on *source text*: every unit's body is concatenated,
//! in evaluation order, into one Script that the ordinary single-script pipeline
//! lowers, and a cross-module read is a read of the exporter's own binding in
//! the one merged top-level scope. A namespace object is materialized the same
//! way — as generated Script text ([`ModuleNamespaceIr::source`],
//! [`namespace_prelude_source`]) placed ahead of every unit body:
//!
//! ```text
//! const $m0$namespace = Object.create(null);
//! Object.defineProperty($m0$namespace, "value", { get: () => value, enumerable: true, configurable: false });
//! Object.defineProperty($m0$namespace, Symbol.toStringTag, { value: "Module", writable: false, enumerable: false, configurable: false });
//! Object.preventExtensions($m0$namespace);
//! const ns = $m0$namespace;
//! ```
//!
//! [`link`]: super::link
//!
//! Every property is an accessor rather than a data property, and that is the
//! one deliberate deviation from 10.4.6: a namespace property is *both* a data
//! property and live, which no ordinary JavaScript object can be. Liveness is
//! the defining behaviour of a namespace binding, so it wins, and the cost is
//! that `Object.getOwnPropertyDescriptor(ns, "value")` reports `get`/`set`
//! instead of `value`/`writable`. Closing that gap needs a real exotic object in
//! the backend — see `porffor-aot-wasm::modules::emit_module_namespace`, which
//! is where that would live and why it is still a stub.
//!
//! Everything else 10.4.6 asks for survives the translation:
//!
//! | invariant | how |
//! | --- | --- |
//! | `[[GetPrototypeOf]]` is `null` | `Object.create(null)` |
//! | `[[IsExtensible]]` is `false` | `Object.preventExtensions` |
//! | `[[Set]]` fails | accessor with no setter (a `TypeError` in strict code, and module code is always strict) |
//! | `[[Delete]]` fails | `configurable: false` |
//! | `[[DefineOwnProperty]]` fails | `configurable: false` on a non-extensible object |
//! | key order | properties are defined in [`ModuleNamespaceIr::exports`] order, which is already UTF-16 code-unit order, and `@@toStringTag` is defined last |
//! | live reads | the getter body names the exporter's binding, which the merged scope holds exactly once |
//! | identity | one `const` per module |
//!
//! Because the getter bodies are *deferred*, the whole prelude can be emitted
//! before any unit body: no export has to be initialized yet, and a namespace
//! whose export is another module's namespace (`export * as inner from "m"`)
//! needs no ordering between the two declarations either.
//!
//! # Deferred namespaces and module source objects
//!
//! Two of the three module request phases are materialized here as well:
//!
//! * `import defer * as ns from "m"` gives `m` a *Deferred Module Namespace*
//!   ([`ModuleNamespaceIr::deferred`]). Its getters route through the thunk
//!   [`deferred_body_source`] wraps `m`'s body in, so the first read of any
//!   export is what evaluates `m`. porffor triggers evaluation on `[[Get]]` of
//!   an export only; the proposal also triggers it from `[[HasProperty]]`,
//!   `[[OwnPropertyKeys]]` and friends, which an accessor cannot observe.
//! * `import source src from "m"` gives `m` a module source object
//!   ([`module_source_object_source`]) and nothing else: `m` is resolved, loaded
//!   and parsed, but never instantiated and never evaluated.

use crate::*;

/// One entry of a module namespace object's export table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleNamespaceExportIr {
    /// The exported name, as seen through the namespace object.
    pub export_name: String,
    /// What `ResolveExport` produced for it.
    pub target: ResolvedBindingIr,
    /// Storage name of the exporter's cell. Reads are live; writes are
    /// rejected.
    ///
    /// This is the *IR-level* cell name (`$m0$value`), which is what a future
    /// backend that emits per-unit environments would address. The source-text
    /// linker in use today names the exporter's binding as the exporter itself
    /// spells it, so the generated namespace source reads [`Self::target`]
    /// through `namespace_target_reference` and never this field.
    pub cell: String,
}

/// A module namespace exotic object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleNamespaceIr {
    /// Module this namespace reflects.
    pub module: ModuleUnitId,
    /// Exports, sorted by UTF-16 code unit as `[[OwnPropertyKeys]]` requires,
    /// with ambiguous and unresolvable names excluded (16.2.1.10).
    pub exports: Vec<ModuleNamespaceExportIr>,
    /// Storage name of the identity-cached namespace object cell.
    ///
    /// Under the source-text linker this doubles as the merged script's binding
    /// name for the object, which is safe because
    /// [`module_namespace_cell_name`] mints it from the unit id rather than
    /// from source.
    pub cell: String,
    /// `true` when this is a *Deferred* Module Namespace: the module is only
    /// reached through `import defer`, so reading any export of this object
    /// evaluates the module first.
    pub deferred: bool,
    /// Merged-script source that materializes this object, or the reason it
    /// cannot be expressed as Script text.
    ///
    /// Filled by `ensure_namespace`, so it is a pure function of the linked
    /// graph and costs nothing when no importer observes the namespace.
    pub source: Result<String, String>,
}

impl ModuleNamespaceIr {
    /// `[[GetPrototypeOf]]` of a namespace object is always `null` (10.4.6.1).
    pub const PROTOTYPE_IS_NULL: bool = true;
    /// A namespace object is never extensible (10.4.6.3).
    pub const EXTENSIBLE: bool = false;
    /// `@@toStringTag`, non-writable / non-enumerable / non-configurable
    /// (10.4.6).
    pub const TO_STRING_TAG: &'static str = "Module";

    /// `[[OwnPropertyKeys]]`, string keys only, already in UTF-16 code-unit
    /// order. `@@toStringTag` follows them.
    #[must_use]
    pub fn own_property_keys(&self) -> Vec<&str> {
        self.exports
            .iter()
            .map(|export| export.export_name.as_str())
            .collect()
    }

    /// Cell a namespace read of `export_name` resolves to.
    ///
    /// `None` means the name is not an own property of this namespace, which is
    /// what `[[Get]]` reports as `undefined` and `[[HasProperty]]` as `false`.
    #[must_use]
    pub fn cell_for(&self, export_name: &str) -> Option<&str> {
        self.exports
            .iter()
            .find(|export| export.export_name == export_name)
            .map(|export| export.cell.as_str())
    }
}

/// Sort key giving UTF-16 code-unit order, which is what `[[OwnPropertyKeys]]`
/// requires and what `String` `Ord` (UTF-8 byte order) does *not* give for
/// astral-plane and some BMP names.
fn utf16_sort_key(name: &str) -> Vec<u16> {
    name.encode_utf16().collect()
}

/// Appends `value` as a double-quoted JavaScript string literal.
///
/// Export names come from `ModuleExportName`, which is an arbitrary string
/// literal (`export { a as "any \u{10000} text" }`), so nothing about a name may
/// be assumed. Escaping runs over UTF-16 code units and emits `\uXXXX` for
/// everything outside printable ASCII, which keeps the generated source ASCII
/// and is the only encoding that survives an unpaired surrogate.
fn push_js_string_literal(out: &mut String, value: &str) {
    out.push('"');
    for unit in value.encode_utf16() {
        match unit {
            0x22 => out.push_str("\\\""),
            0x5C => out.push_str("\\\\"),
            0x08 => out.push_str("\\b"),
            0x09 => out.push_str("\\t"),
            0x0A => out.push_str("\\n"),
            0x0B => out.push_str("\\v"),
            0x0C => out.push_str("\\f"),
            0x0D => out.push_str("\\r"),
            0x20..=0x7E => out.push(char::from(unit as u8)),
            _ => out.push_str(&format!("\\u{unit:04X}")),
        }
    }
    out.push('"');
}

/// `true` when `name` can be written as an `IdentifierReference` in the merged
/// script.
///
/// The generated source names a binding directly, so a name it cannot spell has
/// to be reported rather than emitted. The one such name the module system mints
/// is [`MODULE_ANONYMOUS_DEFAULT_LOCAL_NAME`] (`*default*`), which 8.2.2 chose
/// precisely because no `BindingIdentifier` can spell it.
fn is_binding_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '$' || first == '_' || first.is_alphabetic()) {
        return false;
    }
    chars.all(|ch| ch == '$' || ch == '_' || ch.is_alphanumeric())
}

/// Expression the merged script evaluates to read a resolved binding.
///
/// `None` means the binding cannot be named as Script text: an ambiguous or
/// missing export (which 16.2.1.10 already excludes from the namespace), or a
/// local name no `IdentifierReference` can spell.
#[must_use]
pub fn namespace_target_reference(target: &ResolvedBindingIr) -> Option<String> {
    match target {
        ResolvedBindingIr::Resolved {
            module,
            binding: ModuleBindingNameIr::Namespace,
        } => Some(module_namespace_cell_name(*module)),
        ResolvedBindingIr::Resolved {
            module,
            binding: ModuleBindingNameIr::ModuleSource,
        } => Some(module_source_cell_name(*module)),
        ResolvedBindingIr::Resolved {
            binding: ModuleBindingNameIr::Name(name),
            ..
        } => is_binding_identifier(name).then(|| name.clone()),
        ResolvedBindingIr::Ambiguous | ResolvedBindingIr::NotFound => None,
    }
}

/// Merged-script statements that build one namespace object.
///
/// Property definition order is `exports` order, which `ensure_namespace`
/// already sorted into UTF-16 code-unit order, and `@@toStringTag` is defined
/// last, so `[[OwnPropertyKeys]]` comes out right without the backend sorting
/// anything. `Object.preventExtensions` runs after every definition because a
/// non-extensible object refuses new properties.
fn namespace_object_source(namespace: &ModuleNamespaceIr) -> Result<String, String> {
    let binding = namespace.cell.as_str();
    let mut text = String::new();

    text.push_str("const ");
    text.push_str(binding);
    text.push_str(" = ");
    text.push_str(OBJECT_NAME);
    text.push_str(".create(null);\n");

    for export in &namespace.exports {
        let reference = namespace_target_reference(&export.target).ok_or_else(|| {
            format!(
                "export `{}` resolves to a binding the merged script cannot name",
                export.export_name
            )
        })?;
        text.push_str(OBJECT_NAME);
        text.push_str(".defineProperty(");
        text.push_str(binding);
        text.push_str(", ");
        push_js_string_literal(&mut text, &export.export_name);
        // An accessor, not a data property: see the module docs. No setter, so
        // `[[Set]]` throws in the strict code every module unit is.
        text.push_str(", { get: () => ");
        if namespace.deferred {
            // A deferred module's bindings live in its thunk's scope, not in
            // the merged one, so the getter goes through the export table the
            // thunk publishes — and calling the thunk is what makes the first
            // read of any export evaluate the module.
            text.push_str(&module_defer_evaluate_function_name(namespace.module));
            text.push_str("()[");
            push_js_string_literal(&mut text, &export.export_name);
            text.push_str("]()");
        } else {
            text.push_str(&reference);
        }
        text.push_str(", enumerable: true, configurable: false });\n");
    }

    text.push_str(OBJECT_NAME);
    text.push_str(".defineProperty(");
    text.push_str(binding);
    text.push_str(", ");
    text.push_str(SYMBOL_NAME);
    text.push_str(".toStringTag, { value: ");
    push_js_string_literal(&mut text, ModuleNamespaceIr::TO_STRING_TAG);
    text.push_str(", writable: false, enumerable: false, configurable: false });\n");

    text.push_str(OBJECT_NAME);
    text.push_str(".preventExtensions(");
    text.push_str(binding);
    text.push_str(");\n");

    Ok(text)
}

/// Merged-script prelude declaring every observed namespace object and binding
/// every `import * as ns` local name to one.
///
/// Emitted ahead of every unit body. That is safe in both directions: the
/// getters are deferred, so no export needs to be initialized yet, and the
/// aliases are plain object references, so copying one is not a lost live
/// binding the way copying an exported `let` would be.
///
/// # Errors
/// Returns one diagnostic per reason the prelude cannot be expressed as Script
/// text, so a graph that cannot be linked says exactly what stopped it instead
/// of emitting source that binds the wrong thing.
pub fn namespace_prelude_source(graph: &ModuleGraphIr) -> Result<String, Vec<IrDiagnostic>> {
    let has_source_import = graph.units.iter().any(|unit| {
        unit.record
            .import_entries
            .iter()
            .any(|entry| entry.request.phase == ImportPhaseIr::Source)
    });
    if graph.units.iter().all(|unit| unit.namespace.is_none()) && !has_source_import {
        return Ok(String::new());
    }

    let mut diagnostics = Vec::new();
    report_shadowed_namespace_globals(graph, &mut diagnostics);
    let aliases = collect_namespace_aliases(graph, &mut diagnostics);
    let (source_modules, source_aliases) = collect_module_source_aliases(graph, &mut diagnostics);

    // Unit order is unit-id order, which is stable across runs and independent
    // of evaluation order — the getters are deferred, so no namespace has to be
    // declared before another.
    let mut text = String::new();
    // The deferred export tables first: a deferred namespace's getter calls a
    // thunk that assigns to one, and the thunk is a hoisted `function` that any
    // unit body can reach before its own declaration is stepped over.
    for unit in &graph.units {
        if graph.evaluation_mode(unit.record.id) == ModuleEvaluationModeIr::Deferred {
            text.push_str(&deferred_cells_declaration(unit.record.id));
        }
    }
    for module in &source_modules {
        text.push_str(&module_source_object_source(*module));
    }
    for unit in &graph.units {
        let Some(namespace) = unit.namespace.as_ref() else {
            continue;
        };
        match &namespace.source {
            Ok(source) => text.push_str(source),
            Err(reason) => diagnostics.push(namespace_unsupported(&unit.record.key, reason)),
        }
    }
    for (local, object) in aliases.iter().chain(source_aliases.iter()) {
        text.push_str("const ");
        text.push_str(local);
        text.push_str(" = ");
        text.push_str(object);
        text.push_str(";\n");
    }

    if diagnostics.is_empty() {
        Ok(text)
    } else {
        Err(diagnostics)
    }
}

/// Reports every unit that shadows a global the generated namespace source
/// spells.
///
/// `Object` and `Symbol` are ordinary global bindings, so a unit that declares
/// either at top level shadows it for the whole merged scope — and, for
/// `let`/`const`/`class`, poisons it with a TDZ that no placement of the prelude
/// can dodge.
fn report_shadowed_namespace_globals(graph: &ModuleGraphIr, diagnostics: &mut Vec<IrDiagnostic>) {
    for unit in &graph.units {
        // A unit that does not evaluate inline declares nothing in the merged
        // scope: a deferred body is a function body, and a source-phase-only
        // module has no body in the artifact at all.
        if graph.evaluation_mode(unit.record.id) != ModuleEvaluationModeIr::Eager {
            continue;
        }
        for shadowed in unit
            .record
            .environment
            .iter()
            .filter(|binding| binding.kind != ModuleBindingKindIr::Import)
            .filter(|binding| matches!(binding.name.as_str(), OBJECT_NAME | SYMBOL_NAME))
        {
            diagnostics.push(namespace_unsupported(
                &unit.record.key,
                &format!(
                    "namespace objects are built from `{}`, which this module shadows at top level",
                    shadowed.name
                ),
            ));
        }
    }
}

/// `(local name, namespace object binding)` for every `import * as ns`.
///
/// Unlike an imported *value*, a namespace alias is a fresh `const` in the
/// merged scope rather than a share of the exporter's cell, so it has to be
/// checked for collisions the way an ordinary top-level declaration is —
/// `check_linkable`'s collision map deliberately skips import bindings, so
/// nothing else catches one.
fn collect_namespace_aliases(
    graph: &ModuleGraphIr,
    diagnostics: &mut Vec<IrDiagnostic>,
) -> Vec<(String, String)> {
    // Every top-level name a unit owns outright. Import bindings are excluded
    // for the same reason `check_linkable` excludes them: they are deliberately
    // the exporting unit's cell.
    let mut declared: BTreeMap<&str, &str> = BTreeMap::new();
    for unit in &graph.units {
        // Same reason as `report_shadowed_namespace_globals`: only an eagerly
        // evaluated unit puts its top-level bindings in the merged scope.
        if graph.evaluation_mode(unit.record.id) != ModuleEvaluationModeIr::Eager {
            continue;
        }
        for binding in &unit.record.environment {
            if binding.kind != ModuleBindingKindIr::Import {
                declared.insert(binding.name.as_str(), unit.record.key.as_str());
            }
        }
    }

    let mut aliases = Vec::new();
    let mut owners: BTreeMap<&str, &str> = BTreeMap::new();
    for unit in &graph.units {
        let key = unit.record.key.as_str();
        for (index, entry) in unit.record.import_entries.iter().enumerate() {
            if entry.import_name != ImportNameIr::Namespace {
                continue;
            }
            let local = entry.local_name.as_str();
            let Some(ResolvedBindingIr::Resolved {
                module,
                binding: ModuleBindingNameIr::Namespace,
            }) = unit.resolved_imports.get(index)
            else {
                diagnostics.push(namespace_unsupported(
                    key,
                    &format!("`import * as {local}` did not resolve to a module namespace"),
                ));
                continue;
            };
            if !is_binding_identifier(local) {
                diagnostics.push(namespace_unsupported(
                    key,
                    &format!("namespace binding `{local}` is not spellable"),
                ));
            } else if matches!(local, OBJECT_NAME | SYMBOL_NAME) {
                // The same hazard `report_shadowed_namespace_globals` catches
                // for ordinary declarations, which it cannot see here: an
                // import binding is `ModuleBindingKindIr::Import`, so that
                // filter skips it, yet a namespace alias is emitted as a real
                // `const` in the merged scope. `const Object = ...` would put
                // `Object` in TDZ for the whole script, and the prelude's very
                // first statement is `Object.create(null)`.
                diagnostics.push(namespace_unsupported(
                    key,
                    &format!(
                        "namespace objects are built from `{local}`, which this module binds as a namespace alias"
                    ),
                ));
            } else if let Some(previous) = owners.insert(local, key) {
                diagnostics.push(namespace_unsupported(
                    key,
                    &format!("namespace binding `{local}` is already bound by module {previous}"),
                ));
            } else if let Some(owner) = declared.get(local) {
                diagnostics.push(namespace_unsupported(
                    key,
                    &format!(
                        "namespace binding `{local}` collides with a top-level declaration in module {owner}"
                    ),
                ));
            } else {
                aliases.push((local.to_string(), module_namespace_cell_name(*module)));
            }
        }
    }
    aliases
}

/// Merged-script text for a deferred module's body: a thunk that evaluates it
/// once, plus the export table its namespace object reads through.
///
/// `body` is the unit's already-stripped, already-rewritten body text, exactly
/// what an eager unit would contribute.
///
/// # Why the body moves into a function
///
/// `import defer` needs the body to run on first touch rather than in place,
/// and JavaScript has no way to make top-level statements lazy. A function is
/// the only construct that both delays them and keeps them in one scope.
///
/// The cost is that the module's top-level bindings leave the merged scope, so
/// a namespace getter can no longer name them. The thunk therefore publishes an
/// export table of *accessor closures* built inside its own scope, before the
/// body runs so that a binding still in TDZ is captured rather than read:
///
/// ```text
/// function $m1$defer$evaluate() {
///   if ($m1$defer$cells !== undefined) return $m1$defer$cells;
///   $m1$defer$cells = { __proto__: null, ["v"]: () => v };
///   const v = 10;
///   return $m1$defer$cells;
/// }
/// ```
///
/// Reads stay live: the closure names the binding, it does not copy it. Keys
/// are computed (`["v"]`) rather than literal so that an export named
/// `__proto__` defines a property instead of setting the prototype.
///
/// # Deviation
///
/// The table is published *before* the body, so a module whose body throws
/// leaves a table of bindings in TDZ behind: the first touch propagates the
/// error, as the spec requires, but a second touch raises a `ReferenceError`
/// from the TDZ instead of rethrowing the original. Storing the completion
/// would need the body inside a `try`, which would make its `let`s and `const`s
/// block-scoped and invisible to the table.
///
/// # Errors
/// Returns the reason an export cannot be named as Script text, the same way
/// [`namespace_object_source`] does.
pub(crate) fn deferred_body_source(
    graph: &ModuleGraphIr,
    module: ModuleUnitId,
    body: &str,
) -> Result<String, String> {
    let cells = module_defer_cells_cell_name(module);
    let evaluate = module_defer_evaluate_function_name(module);
    // Never `unwrap_or_default`: an empty table would compile to a namespace
    // whose every export reads `undefined` instead of saying what went wrong.
    // `collect_observed_namespaces` always builds one for a deferred module,
    // because being deferred means an `import defer * as ns` resolved to it.
    let exports = graph
        .units
        .get(module as usize)
        .and_then(|unit| unit.namespace.as_ref())
        .map(|namespace| namespace.exports.as_slice())
        .ok_or_else(|| {
            "deferred module has no namespace object to publish its exports through".to_string()
        })?;

    let mut text = String::new();
    text.push_str("function ");
    text.push_str(&evaluate);
    text.push_str("() {\n");
    text.push_str("if (");
    text.push_str(&cells);
    text.push_str(" !== undefined) return ");
    text.push_str(&cells);
    text.push_str(";\n");
    text.push_str(&cells);
    text.push_str(" = { __proto__: null");
    for export in exports {
        let reference = namespace_target_reference(&export.target).ok_or_else(|| {
            format!(
                "deferred export `{}` resolves to a binding the merged script cannot name",
                export.export_name
            )
        })?;
        text.push_str(", [");
        push_js_string_literal(&mut text, &export.export_name);
        text.push_str("]: () => ");
        text.push_str(&reference);
    }
    text.push_str(" };\n");
    text.push_str(body);
    text.push_str("\n;\nreturn ");
    text.push_str(&cells);
    text.push_str(";\n}\n");
    Ok(text)
}

/// Merged-script declaration of the cell a deferred module's export table lives
/// in.
///
/// Separate from [`deferred_body_source`] because it has to run before any
/// getter can call the thunk, while the thunk itself is a hoisted `function`
/// declaration that can sit wherever the unit's body would have gone.
#[must_use]
pub(crate) fn deferred_cells_declaration(module: ModuleUnitId) -> String {
    format!("let {};\n", module_defer_cells_cell_name(module))
}

/// Merged-script statements building one module source object, and the
/// `import source` locals bound to it.
///
/// # What this object is, and is not
///
/// The source-phase-imports proposal gives a module source object the
/// `%AbstractModuleSource%.prototype` prototype and a `@@toStringTag` accessor
/// reporting the source's class name. porffor has no `%AbstractModuleSource%`
/// intrinsic and no concrete module source type for ECMAScript modules, so what
/// is emitted here is an ordinary null-prototype object carrying an own
/// `@@toStringTag`. It is a distinct, identity-stable handle on a module that
/// was resolved, loaded and parsed but never instantiated — which is the part of
/// the proposal that is observable from the module system — and it is *not* a
/// spec-shaped `AbstractModuleSource`.
fn module_source_object_source(module: ModuleUnitId) -> String {
    let binding = module_source_cell_name(module);
    let mut text = String::new();
    text.push_str("const ");
    text.push_str(&binding);
    text.push_str(" = ");
    text.push_str(OBJECT_NAME);
    text.push_str(".create(null);\n");
    text.push_str(OBJECT_NAME);
    text.push_str(".defineProperty(");
    text.push_str(&binding);
    text.push_str(", ");
    text.push_str(SYMBOL_NAME);
    text.push_str(".toStringTag, { value: ");
    push_js_string_literal(&mut text, MODULE_SOURCE_TO_STRING_TAG);
    text.push_str(", writable: false, enumerable: false, configurable: false });\n");
    text.push_str(OBJECT_NAME);
    text.push_str(".preventExtensions(");
    text.push_str(&binding);
    text.push_str(");\n");
    text
}

/// `@@toStringTag` of a module source object. See
/// [`module_source_object_source`] for why this is porffor's own choice rather
/// than a spec value.
pub const MODULE_SOURCE_TO_STRING_TAG: &str = "Module Source";

/// Every module an `import source` request names, and the local each request
/// binds.
///
/// Collision checking mirrors [`collect_namespace_aliases`]: a source binding is
/// a fresh `const` in the merged scope, not a share of an exporter's cell.
fn collect_module_source_aliases(
    graph: &ModuleGraphIr,
    diagnostics: &mut Vec<IrDiagnostic>,
) -> (BTreeSet<ModuleUnitId>, Vec<(String, String)>) {
    let mut modules = BTreeSet::new();
    let mut aliases = Vec::new();
    for unit in &graph.units {
        let key = unit.record.key.as_str();
        for (index, entry) in unit.record.import_entries.iter().enumerate() {
            if entry.request.phase != ImportPhaseIr::Source {
                continue;
            }
            let local = entry.local_name.as_str();
            let Some(ResolvedBindingIr::Resolved {
                module,
                binding: ModuleBindingNameIr::ModuleSource,
            }) = unit.resolved_imports.get(index)
            else {
                diagnostics.push(namespace_unsupported(
                    key,
                    &format!("`import source {local}` did not resolve to a module"),
                ));
                continue;
            };
            if !is_binding_identifier(local) {
                diagnostics.push(namespace_unsupported(
                    key,
                    &format!("module source binding `{local}` is not spellable"),
                ));
                continue;
            }
            if matches!(local, OBJECT_NAME | SYMBOL_NAME) {
                diagnostics.push(namespace_unsupported(
                    key,
                    &format!(
                        "module source objects are built from `{local}`, which this module binds as a source alias"
                    ),
                ));
                continue;
            }
            modules.insert(*module);
            aliases.push((local.to_string(), module_source_cell_name(*module)));
        }
    }
    (modules, aliases)
}

fn namespace_unsupported(key: &str, reason: &str) -> IrDiagnostic {
    IrDiagnostic::unsupported(format!(
        "unsupported in porffor wasm-aot: module {key}: {reason}"
    ))
}

/// `ModuleNamespaceCreate` (16.2.1.10). Idempotent per module.
///
/// Returns the storage name of the cell holding the identity-cached namespace
/// object, so repeated `import * as ns` and `import()` of the same module
/// observe the same object.
pub(crate) fn ensure_namespace(graph: &mut ModuleGraphIr, module: ModuleUnitId) -> String {
    let cell = module_namespace_cell_name(module);
    let Some(index) = usize::try_from(module)
        .ok()
        .filter(|index| *index < graph.units.len())
    else {
        return cell;
    };
    if graph.units[index].namespace.is_some() {
        return cell;
    }

    let mut exports: Vec<ModuleNamespaceExportIr> = graph
        .exported_names(module)
        .into_iter()
        .filter_map(|export_name| {
            let target = graph.resolve_export(module, &export_name);
            // Ambiguous and unresolvable names are excluded from the namespace.
            let cell = graph.cell_name(&target)?;
            Some(ModuleNamespaceExportIr {
                export_name,
                target,
                cell,
            })
        })
        .collect();
    exports.sort_by(|left, right| {
        utf16_sort_key(&left.export_name).cmp(&utf16_sort_key(&right.export_name))
    });

    let mut namespace = ModuleNamespaceIr {
        module,
        exports,
        cell: cell.clone(),
        deferred: graph.evaluation_mode(module) == ModuleEvaluationModeIr::Deferred,
        source: Ok(String::new()),
    };
    namespace.source = namespace_object_source(&namespace);
    graph.units[index].namespace = Some(namespace);
    cell
}

/// Materializes a namespace object for every module an importer or an
/// `import()` component observes.
///
/// Runs to a fixed point: `export * as ns from "m"` makes one namespace's
/// export resolve to *another* module's namespace, and that module needs an
/// object too even though nobody imports it directly.
pub(crate) fn collect_observed_namespaces(graph: &mut ModuleGraphIr) {
    let mut observed = BTreeSet::new();
    for unit in &graph.units {
        // `import * as ns from "m"`, and `export * as ns from "m"` re-exported
        // onward. Both hand `module_namespace_cell_name(m)` to a reader, so
        // both make `m`'s namespace object observable.
        for binding in unit
            .resolved_imports
            .iter()
            .chain(unit.resolved_indirect_exports.iter())
        {
            if let ResolvedBindingIr::Resolved {
                module,
                binding: ModuleBindingNameIr::Namespace,
            } = binding
            {
                observed.insert(*module);
            }
        }
    }
    for component in &graph.components {
        observed.insert(component.module);
    }

    let mut pending: Vec<ModuleUnitId> = observed.iter().copied().collect();
    while let Some(module) = pending.pop() {
        ensure_namespace(graph, module);
        let Some(namespace) = usize::try_from(module)
            .ok()
            .and_then(|index| graph.units.get(index))
            .and_then(|unit| unit.namespace.as_ref())
        else {
            continue;
        };
        let nested: Vec<ModuleUnitId> = namespace
            .exports
            .iter()
            .filter_map(|export| match &export.target {
                ResolvedBindingIr::Resolved {
                    module,
                    binding: ModuleBindingNameIr::Namespace,
                } => Some(*module),
                ResolvedBindingIr::Resolved { .. }
                | ResolvedBindingIr::Ambiguous
                | ResolvedBindingIr::NotFound => None,
            })
            .collect();
        for module in nested {
            if observed.insert(module) {
                pending.push(module);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph_of(modules: &[(&str, &str)]) -> ModuleGraphIr {
        let sources = ModuleGraphSources {
            modules: modules
                .iter()
                .map(|(key, text)| ModuleSourceIr {
                    key: (*key).to_string(),
                    source_text: (*text).to_string(),
                    meta_url: (*key).to_string(),
                })
                .collect(),
            entry: 0,
            resolutions: Vec::new(),
        };
        let mut graph = super::super::build_graph(&sources).expect("graph should build");
        super::super::link(&mut graph);
        graph
    }

    /// `[[OwnPropertyKeys]]` sorts by UTF-16 code unit, which differs from the
    /// UTF-8 byte order Rust's `str: Ord` gives once a name leaves the BMP —
    /// and a string-literal export name can be anything at all.
    #[test]
    fn namespace_keys_are_in_utf16_code_unit_order() {
        let graph_source = concat!(
            "const a = 1;\n",
            "export { a as '\\u{10000}' };\n",
            "export { a as '\\uFF3A' };\n",
            "export { a as 'b' };\n",
            "export { a as 'a b' };\n",
        );
        let mut graph = graph_of(&[("m", graph_source)]);
        ensure_namespace(&mut graph, 0);
        let namespace = graph.units[0]
            .namespace
            .as_ref()
            .expect("namespace should exist");

        let keys = namespace.own_property_keys();
        // U+10000 is a surrogate pair (0xD800, 0xDC00), so in UTF-16 order it
        // sorts *before* U+FF3A even though its UTF-8 bytes sort after.
        let surrogate = keys
            .iter()
            .position(|key| key.chars().next() == Some('\u{10000}'));
        let fullwidth = keys
            .iter()
            .position(|key| key.chars().next() == Some('\u{FF3A}'));
        assert!(
            surrogate < fullwidth,
            "UTF-16 order puts a surrogate pair before U+FF3A: {keys:?}"
        );
        // A space sorts before an ASCII letter.
        assert!(
            keys.iter().position(|key| *key == "a b") < keys.iter().position(|key| *key == "b")
        );
    }

    /// Every export of the namespace names the *exporter's* cell, so a read is
    /// live rather than a snapshot taken when the object was created.
    #[test]
    fn namespace_entries_point_at_the_exporter_cell() {
        let mut graph = graph_of(&[("m", "export let value = 1;")]);
        ensure_namespace(&mut graph, 0);
        let namespace = graph.units[0]
            .namespace
            .as_ref()
            .expect("namespace should exist");
        assert_eq!(
            namespace.cell_for("value"),
            Some(format!("{}value", module_storage_prefix(0)).as_str())
        );
    }

    /// Identity is one cell per module: `import * as a` and `import * as b` of
    /// the same module must be the same object.
    #[test]
    fn namespace_identity_is_cached_in_one_cell() {
        let mut graph = graph_of(&[("m", "export let value = 1;")]);
        let first = ensure_namespace(&mut graph, 0);
        let second = ensure_namespace(&mut graph, 0);
        assert_eq!(first, second);
        assert_eq!(first, module_namespace_cell_name(0));
        assert_eq!(
            graph.units[0].namespace.as_ref().map(|ns| ns.cell.as_str()),
            Some(first.as_str())
        );
    }

    /// `default` is never provided by `export *`, so it is absent from a
    /// namespace assembled only through star exports.
    #[test]
    fn star_exports_do_not_contribute_default() {
        let mut graph = graph_of(&[("m", "export * from 'other';")]);
        ensure_namespace(&mut graph, 0);
        let namespace = graph.units[0]
            .namespace
            .as_ref()
            .expect("namespace should exist");
        assert!(!namespace
            .own_property_keys()
            .contains(&MODULE_DEFAULT_EXPORT_NAME));
    }

    fn plain(specifier: &str) -> ModuleRequestIr {
        ModuleRequestIr {
            specifier: specifier.to_string(),
            phase: ImportPhaseIr::Evaluation,
            attributes: Vec::new(),
        }
    }

    /// A graph whose entry is the *last* module listed, which is the shape the
    /// linker produces: dependencies first, importer last.
    fn linked_graph(
        modules: &[(&str, &str)],
        resolutions: Vec<(ModuleUnitId, ModuleRequestIr, ModuleUnitId)>,
    ) -> ModuleGraphIr {
        let entry = ModuleUnitId::try_from(modules.len() - 1).expect("entry index fits");
        let sources = ModuleGraphSources {
            modules: modules
                .iter()
                .map(|(key, text)| ModuleSourceIr {
                    key: (*key).to_string(),
                    source_text: (*text).to_string(),
                    meta_url: format!("file:///{key}"),
                })
                .collect(),
            entry,
            resolutions,
        };
        let mut graph = super::super::build_graph(&sources).expect("graph should build");
        super::super::link(&mut graph);
        collect_observed_namespaces(&mut graph);
        graph
    }

    fn source_of(graph: &ModuleGraphIr, module: ModuleUnitId) -> String {
        graph
            .unit(module)
            .namespace
            .as_ref()
            .expect("namespace should exist")
            .source
            .as_ref()
            .expect("namespace should be expressible")
            .clone()
    }

    /// The object is created with a null prototype and sealed against new
    /// properties, and `preventExtensions` runs *after* every definition — the
    /// other order would make every `defineProperty` fail.
    #[test]
    fn namespace_source_creates_a_null_prototype_non_extensible_object() {
        let mut graph = graph_of(&[("m", "export const value = 41;")]);
        ensure_namespace(&mut graph, 0);
        let source = source_of(&graph, 0);
        let binding = module_namespace_cell_name(0);

        assert!(
            source.starts_with(&format!("const {binding} = Object.create(null);\n")),
            "got {source}"
        );
        let prevent = source
            .find("Object.preventExtensions(")
            .expect("extensions are prevented");
        let last_define = source
            .rfind("Object.defineProperty(")
            .expect("properties are defined");
        assert!(
            last_define < prevent,
            "preventExtensions must run last: {source}"
        );
    }

    /// The getter names the *exporter's own* binding, which is what makes the
    /// read live. The IR-level `cell` name (`$m0$value`) belongs to a different,
    /// not-yet-built backend and must not leak into the merged source.
    #[test]
    fn namespace_source_reads_the_exporter_binding_rather_than_a_snapshot() {
        let mut graph = graph_of(&[("m", "export let value = 41;")]);
        ensure_namespace(&mut graph, 0);
        let source = source_of(&graph, 0);

        assert!(source.contains("get: () => value,"), "got {source}");
        // The *export* cell (`$m0$value`) must not appear. The namespace
        // object's own binding (`$m0$namespace`) shares the same prefix by
        // construction (`module_namespace_cell_name`), so asserting the prefix
        // is absent would be unsatisfiable rather than strict.
        assert!(
            !source.contains(&format!("{}value", module_storage_prefix(0))),
            "the IR cell naming scheme must not reach the merged source: {source}"
        );
    }

    /// `export { a as b }` is the case the direct-import path cannot link, and
    /// the namespace path gets it right for free: the *key* is the export name
    /// and the *getter* names the local one.
    #[test]
    fn namespace_source_separates_the_export_name_from_the_local_name() {
        let mut graph = graph_of(&[("m", "const a = 1;\nexport { a as b };")]);
        ensure_namespace(&mut graph, 0);
        let source = source_of(&graph, 0);
        assert!(source.contains("\"b\", { get: () => a,"), "got {source}");
    }

    /// `@@toStringTag` is defined last, so it follows every string key in
    /// `[[OwnPropertyKeys]]`, and it is non-writable / non-enumerable /
    /// non-configurable.
    #[test]
    fn namespace_source_defines_to_string_tag_last_and_locked_down() {
        let mut graph = graph_of(&[("m", "export const value = 1;")]);
        ensure_namespace(&mut graph, 0);
        let source = source_of(&graph, 0);

        let tag = source
            .find("Symbol.toStringTag")
            .expect("toStringTag is defined");
        let value_key = source.find("\"value\"").expect("value is defined");
        assert!(value_key < tag, "string keys come first: {source}");
        assert!(
            source.contains(
                "Symbol.toStringTag, { value: \"Module\", writable: false, enumerable: false, configurable: false });"
            ),
            "got {source}"
        );
    }

    /// Properties are defined in `exports` order, which `ensure_namespace`
    /// already sorted, so the emitted source needs no sorting of its own for
    /// `Object.keys` to come out in `[[OwnPropertyKeys]]` order.
    #[test]
    fn namespace_source_defines_keys_in_own_property_keys_order() {
        let graph_source = concat!(
            "const a = 1;\n",
            "export { a as 'b' };\n",
            "export { a as 'a b' };\n",
        );
        let mut graph = graph_of(&[("m", graph_source)]);
        ensure_namespace(&mut graph, 0);
        let source = source_of(&graph, 0);
        assert!(
            source.find("\"a b\"") < source.find("\"b\""),
            "got {source}"
        );
    }

    /// An export name is an arbitrary string literal, so the generated key has
    /// to survive quotes, backslashes, newlines and astral-plane text.
    #[test]
    fn namespace_source_escapes_arbitrary_export_names() {
        let graph_source = concat!(
            "const a = 1;\n",
            "export { a as 'quote\\\" and \\\\ and \\n' };\n",
            "export { a as '\\u{10000}' };\n",
        );
        let mut graph = graph_of(&[("m", graph_source)]);
        ensure_namespace(&mut graph, 0);
        let source = source_of(&graph, 0);

        assert!(
            source.contains(r#""quote\" and \\ and \n""#),
            "got {source}"
        );
        // U+10000 is the surrogate pair D800 DC00 in UTF-16, and escaping runs
        // over code units so each half survives on its own.
        assert!(source.contains(r"\uD800\uDC00"), "got {source}");
        assert!(source.is_ascii(), "generated source stays ASCII: {source}");
    }

    /// `*default*` cannot be written as an `IdentifierReference`, so a namespace
    /// that would have to name it says so instead of emitting broken source.
    #[test]
    fn an_unspellable_local_name_is_reported_rather_than_emitted() {
        let namespace = ModuleNamespaceIr {
            module: 0,
            exports: vec![ModuleNamespaceExportIr {
                export_name: MODULE_DEFAULT_EXPORT_NAME.to_string(),
                target: ResolvedBindingIr::Resolved {
                    module: 0,
                    binding: ModuleBindingNameIr::Name(
                        MODULE_ANONYMOUS_DEFAULT_LOCAL_NAME.to_string(),
                    ),
                },
                cell: module_namespace_cell_name(0),
            }],
            cell: module_namespace_cell_name(0),
            deferred: false,
            source: Ok(String::new()),
        };
        let error = namespace_object_source(&namespace).expect_err("`*default*` is unspellable");
        assert!(error.contains("default"), "got {error}");
    }

    /// The target behaviour of this lane: `import * as ns` binds a local name to
    /// the exporter's namespace object.
    #[test]
    fn the_prelude_binds_an_import_star_local_to_the_namespace_object() {
        let graph = linked_graph(
            &[
                ("a", "export const value = 41;"),
                ("c", "import * as ns from \"./a.mjs\";\nprint(ns.value);"),
            ],
            vec![(1, plain("./a.mjs"), 0)],
        );
        let prelude = namespace_prelude_source(&graph).expect("prelude should build");

        let binding = module_namespace_cell_name(0);
        assert!(
            prelude.contains(&format!("const {binding} = Object.create(null);")),
            "got {prelude}"
        );
        assert!(prelude.contains("get: () => value,"), "got {prelude}");
        // The alias comes after the object it names.
        let object = prelude.find(&format!("const {binding} =")).expect("object");
        let alias = prelude
            .find(&format!("const ns = {binding};"))
            .expect("alias");
        assert!(object < alias, "got {prelude}");
    }

    /// A graph nobody takes a namespace of pays nothing.
    #[test]
    fn the_prelude_is_empty_when_no_namespace_is_observed() {
        let graph = linked_graph(
            &[
                ("a", "export const value = 41;"),
                ("b", "import { value } from \"./a.mjs\";\nprint(value);"),
            ],
            vec![(1, plain("./a.mjs"), 0)],
        );
        assert_eq!(
            namespace_prelude_source(&graph).expect("prelude should build"),
            ""
        );
    }

    /// The prelude spells `Object`, so a unit that shadows it at top level would
    /// silently break every namespace in the graph.
    #[test]
    fn shadowing_object_is_reported_rather_than_mislinked() {
        let graph = linked_graph(
            &[
                ("a", "export const value = 41;"),
                (
                    "c",
                    "import * as ns from \"./a.mjs\";\nconst Object = 1;\nprint(ns.value);",
                ),
            ],
            vec![(1, plain("./a.mjs"), 0)],
        );
        let diagnostics = namespace_prelude_source(&graph).expect_err("shadowing must be reported");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("shadows at top level")),
            "got {diagnostics:?}"
        );
    }

    /// The same hazard through the one door `report_shadowed_namespace_globals`
    /// cannot see: it filters out `ModuleBindingKindIr::Import` bindings, and a
    /// namespace import is one — yet its alias is emitted as a real `const` in
    /// the merged scope, so `import * as Object` shadows `Object` just as hard
    /// as `const Object = 1` does.
    #[test]
    fn a_namespace_alias_named_object_is_reported_rather_than_mislinked() {
        let graph = linked_graph(
            &[
                ("a", "export const value = 41;"),
                (
                    "c",
                    "import * as Object from \"./a.mjs\";\nprint(Object.value);",
                ),
            ],
            vec![(1, plain("./a.mjs"), 0)],
        );
        let diagnostics =
            namespace_prelude_source(&graph).expect_err("a shadowing alias must be reported");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("binds as a namespace alias")),
            "got {diagnostics:?}"
        );
    }

    /// A namespace alias is a fresh `const` in the merged scope rather than a
    /// shared exporter cell, so two units cannot both spell one.
    #[test]
    fn two_units_binding_the_same_namespace_local_are_reported() {
        let graph = linked_graph(
            &[
                ("a", "export const value = 41;"),
                ("b", "import * as ns from \"./a.mjs\";\nprint(ns.value);"),
                ("c", "import * as ns from \"./a.mjs\";\nprint(ns.value);"),
            ],
            vec![(1, plain("./a.mjs"), 0), (2, plain("./a.mjs"), 0)],
        );
        let diagnostics = namespace_prelude_source(&graph).expect_err("collision must be reported");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("already bound by module")),
            "got {diagnostics:?}"
        );
    }

    /// A namespace alias also cannot collide with a name another unit declares
    /// outright — `check_linkable`'s collision map skips import bindings, so
    /// nothing else catches this.
    #[test]
    fn a_namespace_local_colliding_with_another_units_declaration_is_reported() {
        let graph = linked_graph(
            &[
                ("a", "export const value = 41;\nconst ns = 1;\nprint(ns);"),
                ("c", "import * as ns from \"./a.mjs\";\nprint(ns.value);"),
            ],
            vec![(1, plain("./a.mjs"), 0)],
        );
        let diagnostics = namespace_prelude_source(&graph).expect_err("collision must be reported");
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("collides with a top-level")),
            "got {diagnostics:?}"
        );
    }

    /// A deferred namespace's getter cannot name the exporter's binding — it is
    /// in the thunk's scope, not the merged one — so it reads through the export
    /// table the thunk publishes, and calling the thunk is what evaluates the
    /// module.
    #[test]
    fn a_deferred_namespace_getter_goes_through_the_thunk() {
        let graph = linked_graph(
            &[
                ("a", "export const value = 41;"),
                (
                    "c",
                    "import defer * as ns from \"./a.mjs\";\nprint(ns.value);",
                ),
            ],
            vec![(
                1,
                ModuleRequestIr {
                    specifier: "./a.mjs".to_string(),
                    phase: ImportPhaseIr::Defer,
                    attributes: Vec::new(),
                },
                0,
            )],
        );
        assert_eq!(graph.evaluation_mode(0), ModuleEvaluationModeIr::Deferred);
        let prelude = namespace_prelude_source(&graph).expect("prelude should build");

        assert!(
            prelude.contains(&format!("let {};", module_defer_cells_cell_name(0))),
            "got {prelude}"
        );
        assert!(
            prelude.contains(&format!(
                "get: () => {}()[\"value\"]()",
                module_defer_evaluate_function_name(0)
            )),
            "got {prelude}"
        );
        // The eager form would have named the exporter's binding directly.
        assert!(!prelude.contains("get: () => value,"), "got {prelude}");
    }

    /// The thunk publishes its export table *before* running the body, so a
    /// binding still in TDZ is captured rather than read, and the key is
    /// computed so that an export named `__proto__` defines a property instead
    /// of setting the prototype.
    #[test]
    fn a_deferred_body_publishes_capturing_accessors_before_it_runs() {
        let graph = linked_graph(
            &[
                ("a", "export const value = 41;"),
                (
                    "c",
                    "import defer * as ns from \"./a.mjs\";\nprint(ns.value);",
                ),
            ],
            vec![(
                1,
                ModuleRequestIr {
                    specifier: "./a.mjs".to_string(),
                    phase: ImportPhaseIr::Defer,
                    attributes: Vec::new(),
                },
                0,
            )],
        );
        let thunk = deferred_body_source(&graph, 0, "const value = 41;")
            .expect("the deferred body should be expressible");

        let cells = module_defer_cells_cell_name(0);
        assert!(
            thunk.contains(&format!("if ({cells} !== undefined) return {cells};")),
            "got {thunk}"
        );
        let table = thunk
            .find("[\"value\"]: () => value")
            .expect("export table");
        let body = thunk.find("const value = 41;").expect("body");
        assert!(table < body, "the table must be published first: {thunk}");
    }

    /// `import source` binds its local to a module source object, and that
    /// object is not a namespace: no export of the module is reachable through
    /// it, because the module was never instantiated.
    #[test]
    fn a_source_phase_import_binds_a_module_source_object() {
        let graph = linked_graph(
            &[
                ("a", "export const value = 41;"),
                (
                    "c",
                    "import source src from \"./a.mjs\";\nprint(typeof src);",
                ),
            ],
            vec![(
                1,
                ModuleRequestIr {
                    specifier: "./a.mjs".to_string(),
                    phase: ImportPhaseIr::Source,
                    attributes: Vec::new(),
                },
                0,
            )],
        );
        assert_eq!(
            graph.evaluation_mode(0),
            ModuleEvaluationModeIr::NotEvaluated
        );
        let prelude = namespace_prelude_source(&graph).expect("prelude should build");

        let cell = module_source_cell_name(0);
        assert!(
            prelude.contains(&format!("const {cell} = Object.create(null);")),
            "got {prelude}"
        );
        assert!(
            prelude.contains(&format!(
                "Symbol.toStringTag, {{ value: \"{MODULE_SOURCE_TO_STRING_TAG}\""
            )),
            "got {prelude}"
        );
        assert!(
            prelude.contains(&format!("const src = {cell};")),
            "got {prelude}"
        );
        assert!(
            !prelude.contains("\"value\""),
            "a module source exposes no exports: {prelude}"
        );
    }

    /// `export * as inner from "m"` makes one namespace's export resolve to
    /// another namespace object. The getter names that object's binding, and
    /// because getters are deferred the two declarations need no ordering.
    #[test]
    fn a_nested_namespace_export_names_the_other_namespace_binding() {
        let graph = linked_graph(
            &[
                ("a", "export const value = 41;"),
                ("b", "export * as inner from \"./a.mjs\";"),
                (
                    "c",
                    "import * as ns from \"./b.mjs\";\nprint(ns.inner.value);",
                ),
            ],
            vec![(1, plain("./a.mjs"), 0), (2, plain("./b.mjs"), 1)],
        );
        let prelude = namespace_prelude_source(&graph).expect("prelude should build");
        assert!(
            prelude.contains(&format!(
                "\"inner\", {{ get: () => {},",
                module_namespace_cell_name(0)
            )),
            "got {prelude}"
        );
    }
}
