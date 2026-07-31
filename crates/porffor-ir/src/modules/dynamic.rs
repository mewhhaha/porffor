//! `import()`: `ImportCall` lowering and the component registry.
//!
//! No interpreter is involved and no source is parsed at runtime. Every
//! statically discoverable `import()` target is compiled into the same
//! artifact and registered here under the specifier its call site wrote. At
//! runtime `import(x)` performs `ToString(x)`, looks the string up in this
//! registry, and either runs the memoised component or rejects the promise.
//!
//! This is why `import()` is *in* scope while `eval` is not: a specifier names
//! a module, it is not source text, so serving it needs a loader at compile
//! time rather than a parser at run time.
//!
//! # `EvaluateImportCall` (13.3.10.1), and what happens where
//!
//! Step 3-6 (evaluate the specifier expression, then the options expression)
//! happen *before* the promise exists, so an abrupt completion there throws
//! normally. [`lower_import_call`] therefore lowers both operands plainly and
//! deliberately inserts no `ToString` coercion.
//!
//! Step 8-9 (`ToString(specifier)`, then `IfAbruptRejectPromise`) happen
//! *after* `NewPromiseCapability`, so a `Symbol` specifier or a throwing
//! `toString` rejects the promise instead of throwing. That distinction is the
//! whole point of test262's `dynamic-import/catch/` subtree, and it is the
//! backend's job: `ExprIr::DynamicImport` carries the uncoerced operand.
//!
//! # What is memoised
//!
//! A module evaluates at most once, but *every* `import()` call produces a
//! fresh promise object — `always-create-new-promise.js` compares the results
//! of two calls with the same specifier and requires them to be distinct.
//! [`DynamicComponentIr::completion_cell`] therefore memoises the module's
//! *evaluation completion*, never a promise object; each call allocates a new
//! promise and settles it from that cell.
//!
//! # How a linked graph actually serves `import()` today
//!
//! [`link`] merges a graph on *source text*: every unit's body is concatenated,
//! in evaluation order, into one Script that the ordinary single-script pipeline
//! lowers. `import()` is served the same way, one stage earlier than the IR:
//!
//! * [`ModuleGraphIr::dynamic_import_prelude`] emits one dispatcher function
//!   per module that writes an `import()`. The namespace objects the
//!   dispatchers resolve with are *not* minted here: they are the ones
//!   `modules::namespace` already emits under
//!   [`module_namespace_cell_name`], which is what makes
//!   `import("./a.mjs")` and `import * as ns from "./a.mjs"` produce the same
//!   object (16.2.1.10 caches `[[Namespace]]` per module);
//! * [`ModuleGraphIr::rewrite_dynamic_import_calls`] rewrites the `import`
//!   keyword of every `import(` call site in a unit's text into that unit's
//!   dispatcher name, so the merged script contains an ordinary call
//!   expression and no `ImportCall` node survives to the backend;
//! * [`ModuleGraphIr::check_dynamic_import_linkable`] reports every shape this
//!   desugaring cannot express, so an unsupported graph fails with one honest
//!   diagnostic instead of being mislinked.
//!
//! [`link`]: super::link
//!
//! Because the whole thing is source in, source out, it rests only on language
//! features the backend already runs: `new Promise(executor)`, an object literal
//! with getters, and a template literal. Nothing here needs new Wasm emission.
//!
//! The generated dispatcher is a faithful reading of `EvaluateImportCall`:
//!
//! ```js
//! function $porffor$module$import$1(specifier, options) {
//!   return new Promise(function (resolve, reject) {
//!     var key = `${specifier}`;
//!     if (key === "./a.mjs") { resolve($m0$namespace); return; }
//!     reject(new TypeError("Cannot find module " + key));
//!   });
//! }
//! ```
//!
//! The specifier and the options expressions are evaluated by the *caller*, at
//! the call site, before the dispatcher is entered — so an abrupt completion
//! there throws (steps 3-6). `ToString` happens inside the executor, after the
//! capability exists, so a `Symbol` specifier or a throwing `toString` rejects
//! the promise instead of throwing (steps 8-9); the template literal is chosen
//! over `String(specifier)` precisely because `String(symbol)` does *not*
//! throw while `ToString(symbol)` does. A specifier that is not in the compiled
//! graph reaches the final `reject`, so it is a rejected promise and never a
//! trap.
//!
//! ## When the target module's body runs
//!
//! `scan_module_requests` reports a static `import()` specifier as a request, so
//! the host loads an `import()` target into the graph like any other unit, and
//! `compute_evaluation_order`'s root loop visits *every* unit — so the target's
//! body is emitted into the merged script whether or not anything statically
//! imports it. The consequence is that an `import()` target is evaluated
//! **eagerly**, as part of the merged script, rather than at the call.
//!
//! That is a deviation from 13.3.10, and it is visible two ways: a target with a
//! top-level side effect performs it even if the `import()` never runs, and it
//! performs it at its place in the merged script rather than at the call. It is
//! *not* visible through the promise, because the namespace object's getters
//! read their bindings when a property is read — which, for the `.then` callback
//! that reads them, is after the whole merged script has run.
//!
//! Tarjan's root loop starts at the entry (unit 0), so a unit reachable *only*
//! through `import()` has no edge in `requested_modules`, becomes a separate
//! root, and lands **after** the entry in `evaluation_order` — which would cost
//! the merged script the entry's completion value and run an eagerly evaluated
//! dynamic dependency after its own dependent. `modules::link::emission_order`
//! fixes both by rotating the entry's strongly-connected component to the end.
//!
//! Removing the eager evaluation altogether is what `StatementIr::ModuleUnitOnce`
//! and [`DynamicComponentIr::completion_cell`] exist for, and needs a pass that
//! hoists a unit's declarations out of its body so the body can be wrapped in a
//! guarded function without moving its bindings.
//!
//! ## Deviations this stage knowingly carries
//!
//! * The promise is settled synchronously inside the executor rather than from
//!   a later job. Reactions still run as microtasks, so only the interleaving of
//!   `import()` with other already-queued jobs can observe the difference.
//! * **Every options validation of 13.3.10.1 step 11 is skipped.** The
//!   dispatcher binds `options` and never reads it, so `import('m', 123)`
//!   fulfils instead of rejecting with a `TypeError` (11.a), a throwing `with`
//!   getter is never called (11.b-c), a malformed attribute key or value is not
//!   rejected (11.d), and `AllImportAttributesSupported` never runs, so
//!   `import('m', { with: { type: 'json' } })` silently resolves to the
//!   JavaScript namespace rather than rejecting (11.e). That is the
//!   `dynamic-import/2nd-param-*` subtree, wrong answers rather than missing
//!   ones, and it is the first follow-up this lane owes.
//! * Import attributes also do not participate in resolution:
//!   `DynamicImportSiteIr` does not record them, so `import('m', { with: ... })`
//!   and `import('m')` collapse onto one unit.
//! * A target whose top-level completes abruptly aborts the whole merged script
//!   rather than rejecting the promise (13.3.10.2 step 5), because the target's
//!   body is inlined. `import('./throws.mjs').catch(f)` cannot catch. This is a
//!   consequence of the eager evaluation above, not a separate defect.

use crate::*;

use boa_ast::declaration::ImportPhase;
use boa_ast::expression::ImportCall;

/// One module reachable through `import()`.
///
/// A component is minted per `(referrer, specifier)` pair, not per module: two
/// modules may both write `import('./m.js')` and mean different files, so the
/// specifier alone does not identify a target. The runtime match is therefore
/// on the pair, which [`ModuleGraphIr::resolve_dynamic_component`] performs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicComponentIr {
    /// Host-normalized key of the target. Diagnostics and `import.meta` use
    /// it; the runtime string is *not* matched against it, because the runtime
    /// string is the specifier as written.
    pub key: String,
    /// Specifier text as written at the call site. This is what `ToString` of
    /// the runtime argument is compared against.
    pub specifier: String,
    /// Module that wrote the call site.
    pub referrer: ModuleUnitId,
    /// Module `specifier` resolves to.
    pub module: ModuleUnitId,
    /// `[[Phase]]` of the request. Always [`ImportPhaseIr::Evaluation`]:
    /// `import.defer()` and `import.source()` are rejected during lowering.
    pub phase: ImportPhaseIr,
    /// Cell memoising this component's *evaluation completion* — the namespace
    /// object it resolved to, or the error it threw.
    ///
    /// Not a promise. Reusing one promise object across calls is observably
    /// wrong (`always-create-new-promise.js`); the module evaluates once, the
    /// promise is new every time.
    pub completion_cell: String,
}

/// Registers every statically discoverable `import()` target as a component.
///
/// A call site with a computed specifier registers nothing: it resolves at
/// runtime against whatever the registry already holds, and rejects if nothing
/// matches. That is the entire dynamic-source story — there is no fallback
/// path that reaches a parser.
pub(crate) fn collect_components(graph: &mut ModuleGraphIr) {
    let mut components: Vec<DynamicComponentIr> = Vec::new();
    for index in 0..graph.units.len() {
        let referrer = ModuleUnitId::try_from(index).unwrap_or(ModuleUnitId::MAX);
        let sites = graph.units[index].record.dynamic_import_sites.clone();
        for site in sites {
            // `import.defer()` / `import.source()` never become components:
            // lowering rejects them outright, so a component for one could
            // only ever be dead weight in the artifact.
            if site.phase != ImportPhaseIr::Evaluation {
                continue;
            }
            let Some(specifier) = site.static_specifier else {
                continue;
            };
            let request = site_request(&specifier, site.phase);
            let Some(module) = graph.resolve_request(referrer, &request) else {
                continue;
            };
            if components
                .iter()
                .any(|existing| existing.referrer == referrer && existing.specifier == specifier)
            {
                continue;
            }
            let key = graph.unit(module).record.key.clone();
            components.push(DynamicComponentIr {
                key,
                specifier,
                referrer,
                module,
                phase: site.phase,
                // Keyed by target module, not by call site: two specifiers
                // naming the same module share one evaluation.
                completion_cell: module_component_completion_cell_name(module),
            });
        }
    }
    graph.components = components;
}

/// The `ModuleRequest` a dynamic call site makes.
///
/// Equality of `ModuleRequestIr` is `ModuleRequestsEqual`, so attributes
/// participate in resolution: `import('m', { with: { type: 'json' } })` and
/// `import('m')` are distinct requests resolving to distinct units. The
/// attributes are not available here yet — `DynamicImportSiteIr` records only
/// the specifier and the phase — so every dynamic request is currently
/// attribute-free and those two forms collapse onto one unit. See the
/// registry note for the `DynamicImportSiteIr::attributes` field that fixes
/// it; this function is the single place that then changes.
fn site_request(specifier: &str, phase: ImportPhaseIr) -> ModuleRequestIr {
    ModuleRequestIr {
        specifier: specifier.to_string(),
        phase,
        attributes: Vec::new(),
    }
}

impl ModuleGraphIr {
    /// The component `import(specifier)` in `referrer` resolves to.
    ///
    /// The single authority for the runtime lookup rule, so the backend and
    /// the registry cannot disagree about it.
    ///
    /// `referrer` is `None` for an `import()` in a Script, which has no module
    /// graph and therefore no compile-time resolution context at all: such a
    /// call always rejects, which is exactly what a Script-goal test that only
    /// checks `import()` produces a Promise needs.
    #[must_use]
    pub fn resolve_dynamic_component(
        &self,
        referrer: Option<ModuleUnitId>,
        specifier: &str,
    ) -> Option<&DynamicComponentIr> {
        let referrer = referrer?;
        self.components
            .iter()
            .find(|component| component.referrer == referrer && component.specifier == specifier)
    }
}

/// `ImportPhaseIr` of a boa `ImportCall`.
///
/// A local copy of `ImportPhaseIr::from_ast`, which is private to `record`.
const fn call_phase(phase: ImportPhase) -> ImportPhaseIr {
    match phase {
        ImportPhase::Evaluation => ImportPhaseIr::Evaluation,
        ImportPhase::Defer => ImportPhaseIr::Defer,
        ImportPhase::Source => ImportPhaseIr::Source,
    }
}

/// Lowers `import(specifier, options)` to [`ExprIr::DynamicImport`].
///
/// `specifier` and `options` are the *already lowered* operands, in evaluation
/// order — the caller lowers them because only the caller owns a lowerer.
/// Neither is coerced here: `ToString` of the specifier happens after the
/// promise capability exists, so it must reject rather than throw.
///
/// `referrer` is the module the call site belongs to, or `None` in a Script.
/// `import()` is legal in Script goal, so `None` is an ordinary case and not
/// an error.
///
/// # Errors
/// Returns the diagnostic message for `import.defer()` and `import.source()`,
/// which are parsed but not implemented. An honest reject beats a silent wrong
/// answer: `defer` must not evaluate eagerly and `source` must not produce a
/// namespace, so lowering either as a plain evaluation-phase import would be
/// observably wrong.
pub(crate) fn lower_import_call(
    call: &ImportCall,
    specifier: TypedExpr,
    options: Option<TypedExpr>,
    referrer: Option<ModuleUnitId>,
) -> Result<TypedExpr, String> {
    let phase = call_phase(call.phase());
    if phase != ImportPhaseIr::Evaluation {
        return Err(format!(
            "unsupported in porffor wasm-aot: import.{}() dynamic import phase",
            phase.as_str()
        ));
    }
    // Always a Promise object, on every path including the rejecting one, so
    // the kind is a singleton and the backend emits the value directly.
    Ok(TypedExpr::from_info(
        ValueInfo::new(ValueKind::Object),
        ExprIr::DynamicImport {
            specifier: Box::new(specifier),
            options: options.map(Box::new),
            phase,
            referrer,
        },
    ))
}

/// Prefix every identifier the linker synthesizes for `import()` carries.
///
/// `$` is an identifier character in JavaScript, so these are ordinary names in
/// the merged top-level scope rather than anything the parser treats specially.
/// "Ordinary" would mean "collidable" if nothing checked, so
/// [`ModuleGraphIr::check_dynamic_import_linkable`] rejects a graph in which a
/// module declares a top-level name starting with this prefix.
pub const LINKER_NAME_PREFIX: &str = "$porffor$module$";

/// Merged-scope name of the `import()` dispatcher `unit`'s call sites call.
///
/// One per *referrer*, not one per target: two modules may both write
/// `import('./m.js')` and mean different files, so the specifier alone does not
/// identify a target and the dispatcher has to be the thing that knows which
/// module asked.
fn dispatcher_name(unit: ModuleUnitId) -> String {
    format!("{LINKER_NAME_PREFIX}import${unit}")
}

impl ModuleGraphIr {
    /// JavaScript the merged script must run before any module body, after the
    /// namespace prelude `modules::namespace` owns.
    ///
    /// This lane mints no namespace objects of its own. A dispatcher's
    /// `resolve` names [`module_namespace_cell_name`], the *same* binding
    /// `import * as ns` aliases, so `import("./a.mjs")` and
    /// `import * as ns from "./a.mjs"` hand back one object — 16.2.1.10 caches
    /// `[[Namespace]]` per module and test262's `module-code/namespace/`
    /// compares the two with `===`. The dispatcher bodies are deferred, so a
    /// `function` declaration may precede the `const` it names.
    ///
    /// Emitted as a *single line* with no interior line terminator, and empty
    /// when the graph has nothing dynamic in it. Every line of every unit body
    /// in the merged script is therefore displaced by the same fixed amount no
    /// matter how large the graph is, which is what keeps a diagnostic's line
    /// number worth reading.
    #[must_use]
    pub fn dynamic_import_prelude(&self) -> String {
        self.dynamic_import_dispatchers()
    }

    /// One `import()` dispatcher function per module that writes an `import()`.
    ///
    /// A module with call sites but no resolvable target still gets one: its
    /// dispatcher falls straight through to `reject`, which is what a computed
    /// specifier naming nothing in the compiled graph must do.
    #[must_use]
    pub fn dynamic_import_dispatchers(&self) -> String {
        let mut text = String::new();
        for (index, unit) in self.units.iter().enumerate() {
            if unit.record.dynamic_import_sites.is_empty() {
                continue;
            }
            let Ok(referrer) = ModuleUnitId::try_from(index) else {
                continue;
            };
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str("function ");
            text.push_str(&dispatcher_name(referrer));
            // `options` is bound and ignored. Binding it keeps the call site's
            // second operand an ordinary argument, so it is still evaluated in
            // source order before the promise exists (step 6).
            text.push_str("(specifier, options) { return new Promise(function (resolve, reject) {");
            // `ToString`, not `String()`: `String(symbol)` answers
            // `"Symbol(d)"` while `ToString(symbol)` throws, and step 8 wants
            // the throw so that step 9 can turn it into a rejection.
            text.push_str(" var key = `${specifier}`;");
            for component in self
                .components
                .iter()
                .filter(|component| component.referrer == referrer)
            {
                text.push_str(" if (key === ");
                text.push_str(&js_string_literal(&component.specifier));
                text.push_str(") { resolve(");
                text.push_str(&module_namespace_cell_name(component.module));
                text.push_str("); return; }");
            }
            text.push_str(" reject(new TypeError(\"Cannot find module \" + key)); }); }");
        }
        text
    }

    /// Rewrites every `import(` call site in `source` into a call to `unit`'s
    /// dispatcher.
    ///
    /// # Where this belongs in the linker's rewrite chain
    ///
    /// **Last.** `rewrite_import_meta` and
    /// [`strip_module_syntax`](super::source::strip_module_syntax) both preserve
    /// byte length, because their outputs are addressed by spans the record
    /// captured against the original text. This one does not: the dispatcher
    /// name is longer than the `import` keyword. Running it last means no
    /// span-addressed pass ever sees the shifted offsets.
    ///
    /// It is otherwise indifferent to what ran before it. A static `import`
    /// declaration is never followed by `(`, so stripping first or not changes
    /// nothing, and `rewrite_import_meta` has already turned `import.meta` into
    /// an ordinary identifier by the time this runs.
    ///
    /// This is a JavaScript lexical scan, not a substring replacement. Comments,
    /// string literals, template literals with nested `${}` substitutions and
    /// regular-expression literals are all skipped, and `import` is only a
    /// keyword when it is not preceded by `.` — so `import.meta` and
    /// `obj.import(x)` are left for their own owners, and `"import('m')"` inside
    /// a string stays a string.
    ///
    /// The replacement is longer than the `import` keyword and inserts no line
    /// terminator, so byte offsets shift within a line but the line structure of
    /// the unit survives exactly.
    ///
    /// # Errors
    /// Returns a diagnostic message body when the scanner cannot lex `source`
    /// (an unterminated string, comment, template or regular expression), and
    /// when it finds a call site in a unit whose record says the unit has none.
    /// The second case means the lexical scan and boa's parse disagree — an
    /// object literal or class body with a method literally named `import` is
    /// the way to provoke it — and failing loudly beats emitting a call to a
    /// dispatcher that was never declared.
    ///
    /// # Panics
    /// Panics if `unit` is not a unit of this graph.
    pub fn rewrite_dynamic_import_calls(
        &self,
        unit: ModuleUnitId,
        source: &str,
    ) -> Result<String, String> {
        let sites = ImportCallScanner::new(source).run()?;
        if sites.is_empty() {
            return Ok(source.to_string());
        }
        // A *count* comparison, not an emptiness one. The scanner flags any
        // `import` word not preceded by `.` and followed by `(`, which includes
        // `{ import() {} }`, `{ get import() {} }` and `class C { #import() {} }`
        // — none of which boa records as an `ImportCall`. Testing emptiness lets
        // a unit that has one real `import()` carry any number of hallucinated
        // sites through, renaming a method to a dispatcher (a runtime
        // `TypeError`) or emitting `#$porffor$module$import$0()` (a syntax error
        // in generated source).
        let recorded = self.unit(unit).record.dynamic_import_sites.len();
        if sites.len() != recorded {
            // No module key in the message: the caller in `modules::link`
            // already prefixes `module {key}:`, and saying it twice reads as a
            // bug in the diagnostic rather than in the source.
            return Err(format!(
                "found {} `import(` call site(s) but the module record lists {recorded}; \
                 a property or method named `import` cannot be told apart lexically",
                sites.len()
            ));
        }

        let name = dispatcher_name(unit);
        let mut rewritten = String::with_capacity(source.len() + sites.len() * name.len());
        let mut cursor = 0usize;
        for (start, end) in sites {
            rewritten.push_str(&source[cursor..start]);
            rewritten.push_str(&name);
            cursor = end;
        }
        rewritten.push_str(&source[cursor..]);
        Ok(rewritten)
    }

    /// Every reason this graph's `import()` usage cannot be desugared.
    ///
    /// Empty means [`Self::dynamic_import_prelude`] and
    /// [`Self::rewrite_dynamic_import_calls`] describe the program exactly.
    ///
    /// Deliberately scoped to what `import()` reaches: a namespace observed only
    /// by a static `import * as ns` is somebody else's report to make, and
    /// duplicating it would print the same problem twice.
    #[must_use]
    pub fn check_dynamic_import_linkable(&self) -> Vec<IrDiagnostic> {
        let mut diagnostics = Vec::new();

        for unit in &self.units {
            let key = &unit.record.key;
            for site in &unit.record.dynamic_import_sites {
                // `import.defer()` must not evaluate eagerly and
                // `import.source()` must not produce a namespace, so neither can
                // be served by a dispatcher that resolves to one.
                if site.phase != ImportPhaseIr::Evaluation {
                    diagnostics.push(IrDiagnostic::unsupported(format!(
                        "unsupported in porffor wasm-aot: module {key}: import.{}() dynamic import phase",
                        site.phase.as_str()
                    )));
                }
            }
            for binding in &unit.record.environment {
                if binding.name.starts_with(LINKER_NAME_PREFIX) {
                    diagnostics.push(IrDiagnostic::unsupported(format!(
                        "unsupported in porffor wasm-aot: module {key}: top-level `{}` collides \
                         with a linker-synthesized name",
                        binding.name
                    )));
                }
            }
        }

        for module in self.component_namespace_modules() {
            let Some(namespace) = self
                .units
                .get(module as usize)
                .and_then(|unit| unit.namespace.as_ref())
            else {
                diagnostics.push(IrDiagnostic::unsupported(format!(
                    "unsupported in porffor wasm-aot: `import()` target module {module} has no \
                     namespace object"
                )));
                continue;
            };
            let key = &self.unit(module).record.key;
            for export in &namespace.exports {
                // The same spellability rule `modules::namespace` applies when
                // it emits the getter, asked through the same function, so the
                // two cannot disagree about which exports are expressible.
                if super::namespace::namespace_target_reference(&export.target).is_none() {
                    diagnostics.push(IrDiagnostic::unsupported(format!(
                        "unsupported in porffor wasm-aot: module {key}: `import()` cannot expose \
                         export `{}`, whose binding has no name in the merged scope",
                        export.export_name
                    )));
                }
            }
        }

        diagnostics
    }

    /// Modules whose namespace object an `import()` can reach.
    ///
    /// Transitive, because `export * as inner from "m"` makes one namespace's
    /// export *be* another module's namespace: resolving the outer one hands the
    /// inner object to the program even though no `import()` named it.
    fn component_namespace_modules(&self) -> BTreeSet<ModuleUnitId> {
        let mut observed: BTreeSet<ModuleUnitId> =
            self.components.iter().map(|entry| entry.module).collect();
        let mut pending: Vec<ModuleUnitId> = observed.iter().copied().collect();
        while let Some(module) = pending.pop() {
            let Some(namespace) = self
                .units
                .get(module as usize)
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
        observed
    }
}

/// `value` as a double-quoted JavaScript string literal.
///
/// Escapes U+2028 and U+2029 as well as the obvious cases: both are ordinary
/// characters in an ECMAScript string but line terminators to the *scanner*,
/// and an export name or a specifier is arbitrary text that may contain either.
fn js_string_literal(value: &str) -> String {
    let mut text = String::with_capacity(value.len() + 2);
    text.push('"');
    for character in value.chars() {
        match character {
            '"' => text.push_str("\\\""),
            '\\' => text.push_str("\\\\"),
            '\n' => text.push_str("\\n"),
            '\r' => text.push_str("\\r"),
            '\t' => text.push_str("\\t"),
            '\u{2028}' | '\u{2029}' => {
                text.push_str(&format!("\\u{:04X}", character as u32));
            }
            character if (character as u32) < 0x20 => {
                text.push_str(&format!("\\u{:04X}", character as u32));
            }
            character => text.push(character),
        }
    }
    text.push('"');
    text
}

/// What a `/` means at the current position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlashMeaning {
    /// The previous significant token can end an expression, so `/` divides.
    Divide,
    /// The previous significant token cannot end an expression, so `/` opens a
    /// regular-expression literal.
    Regexp,
}

/// Finds the byte range of every `import` keyword that opens an `import(` call.
///
/// A sibling of the scanner in [`source`](super::source), and deliberately not a
/// reuse of it: that one deletes declarations, which only exist at nesting depth
/// zero, while an `import()` call is an expression and can appear anywhere. The
/// shared machinery is the lexing — comments, strings, templates, regular
/// expressions — and the shared discipline is that a keyword after `.` is a
/// property name.
struct ImportCallScanner<'a> {
    source: &'a str,
    bytes: &'a [u8],
    /// Byte ranges of `import` keywords to replace, ascending, non-overlapping.
    sites: Vec<(usize, usize)>,
    /// Nesting depth of `(`, `[` and `{`, tracked only so that the `}` closing a
    /// template substitution is told apart from an ordinary `}`.
    depth: usize,
    /// One entry per open template substitution, holding the `depth` *inside*
    /// it.
    template_stack: Vec<usize>,
    slash: SlashMeaning,
    /// The previous significant token was `.`, so the next word is a property
    /// name rather than a keyword.
    previous_was_dot: bool,
    index: usize,
}

impl<'a> ImportCallScanner<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            sites: Vec::new(),
            depth: 0,
            template_stack: Vec::new(),
            slash: SlashMeaning::Regexp,
            previous_was_dot: false,
            index: 0,
        }
    }

    fn run(mut self) -> Result<Vec<(usize, usize)>, String> {
        while self.index < self.bytes.len() {
            let byte = self.bytes[self.index];
            match byte {
                b'/' if self.bytes.get(self.index + 1) == Some(&b'/') => self.skip_line_comment(),
                b'/' if self.bytes.get(self.index + 1) == Some(&b'*') => {
                    self.skip_block_comment()?;
                }
                b'/' if self.slash == SlashMeaning::Regexp => {
                    self.skip_regexp()?;
                    self.slash = SlashMeaning::Divide;
                    self.previous_was_dot = false;
                }
                b'\'' | b'"' => {
                    self.index = self.string_end(self.index, byte)?;
                    self.slash = SlashMeaning::Divide;
                    self.previous_was_dot = false;
                }
                b'`' => {
                    self.index += 1;
                    self.scan_template_body()?;
                }
                b'(' | b'[' | b'{' => {
                    self.depth += 1;
                    self.index += 1;
                    self.slash = SlashMeaning::Regexp;
                    self.previous_was_dot = false;
                }
                b')' | b']' => {
                    self.depth = self.depth.saturating_sub(1);
                    self.index += 1;
                    self.slash = SlashMeaning::Divide;
                    self.previous_was_dot = false;
                }
                b'}' => {
                    // The stack holds the depth *inside* the substitution, so
                    // the match is against the current depth before unwinding.
                    let closes_substitution = self
                        .template_stack
                        .last()
                        .is_some_and(|open_depth| *open_depth == self.depth);
                    self.depth = self.depth.saturating_sub(1);
                    self.index += 1;
                    if closes_substitution {
                        self.template_stack.pop();
                        self.scan_template_body()?;
                        continue;
                    }
                    self.slash = SlashMeaning::Divide;
                    self.previous_was_dot = false;
                }
                b'.' => {
                    self.index += 1;
                    self.slash = SlashMeaning::Regexp;
                    self.previous_was_dot = true;
                }
                byte if is_identifier_start_byte(byte) => self.scan_word(),
                byte if byte.is_ascii_digit() => {
                    self.skip_number();
                    self.slash = SlashMeaning::Divide;
                    self.previous_was_dot = false;
                }
                byte if byte.is_ascii_whitespace() => self.index += 1,
                _ => {
                    // Any other punctuator. `++`/`--` end an expression; every
                    // other operator opens one.
                    let two = self.source.get(self.index..self.index + 2);
                    self.slash = if two == Some("++") || two == Some("--") {
                        SlashMeaning::Divide
                    } else {
                        SlashMeaning::Regexp
                    };
                    self.previous_was_dot = false;
                    self.index += self.char_len_at(self.index);
                }
            }
        }
        Ok(self.sites)
    }

    /// Words after which a `/` starts a regular expression rather than a
    /// division.
    ///
    /// Purely a lexing concern: `return /re/` is a regexp, `x / y` is a divide.
    /// `let`, `static` and friends are contextual and legal as binding names, so
    /// they are deliberately absent.
    fn is_reserved_word(word: &str) -> bool {
        matches!(
            word,
            "await"
                | "break"
                | "case"
                | "catch"
                | "class"
                | "const"
                | "continue"
                | "debugger"
                | "default"
                | "delete"
                | "do"
                | "else"
                | "enum"
                | "export"
                | "extends"
                | "false"
                | "finally"
                | "for"
                | "function"
                | "if"
                | "import"
                | "in"
                | "instanceof"
                | "new"
                | "null"
                | "return"
                | "super"
                | "switch"
                | "this"
                | "throw"
                | "true"
                | "try"
                | "typeof"
                | "var"
                | "void"
                | "while"
                | "with"
                | "yield"
        )
    }

    fn scan_word(&mut self) {
        let start = self.index;
        while self
            .bytes
            .get(self.index)
            .copied()
            .is_some_and(is_identifier_part_byte)
        {
            self.index += 1;
        }
        let word = &self.source[start..self.index];
        if word == "import" && !self.previous_was_dot && self.peek_significant() == Some(b'(') {
            self.sites.push((start, self.index));
        }
        self.slash = match word {
            "this" | "super" | "true" | "false" | "null" => SlashMeaning::Divide,
            word if Self::is_reserved_word(word) => SlashMeaning::Regexp,
            _ => SlashMeaning::Divide,
        };
        self.previous_was_dot = false;
    }

    fn char_len_at(&self, index: usize) -> usize {
        self.source[index..]
            .chars()
            .next()
            .map_or(1, char::len_utf8)
    }

    /// First non-whitespace, non-comment byte at or after `self.index`.
    fn peek_significant(&self) -> Option<u8> {
        let index = self.skip_trivia_from(self.index).ok()?;
        self.bytes.get(index).copied()
    }

    /// Skips whitespace and comments starting at `index`.
    fn skip_trivia_from(&self, mut index: usize) -> Result<usize, String> {
        loop {
            match self.bytes.get(index).copied() {
                Some(byte) if byte.is_ascii_whitespace() => index += 1,
                Some(b'/') if self.bytes.get(index + 1) == Some(&b'/') => {
                    while self
                        .bytes
                        .get(index)
                        .copied()
                        .is_some_and(|byte| byte != b'\n')
                    {
                        index += 1;
                    }
                }
                Some(b'/') if self.bytes.get(index + 1) == Some(&b'*') => {
                    let mut end = index + 2;
                    loop {
                        if end + 1 >= self.bytes.len() {
                            return Err("unterminated block comment".to_string());
                        }
                        if self.bytes[end] == b'*' && self.bytes[end + 1] == b'/' {
                            end += 2;
                            break;
                        }
                        end += 1;
                    }
                    index = end;
                }
                Some(byte) if !byte.is_ascii() => {
                    let character = self.source[index..].chars().next().unwrap_or(' ');
                    if character.is_whitespace() {
                        index += character.len_utf8();
                    } else {
                        return Ok(index);
                    }
                }
                _ => return Ok(index),
            }
        }
    }

    fn string_end(&self, start: usize, quote: u8) -> Result<usize, String> {
        let mut index = start + 1;
        while index < self.bytes.len() {
            match self.bytes[index] {
                b'\\' => index += 1 + self.char_len_at((index + 1).min(self.bytes.len())),
                byte if byte == quote => return Ok(index + 1),
                _ => index += self.char_len_at(index),
            }
        }
        Err("unterminated string literal".to_string())
    }

    fn skip_line_comment(&mut self) {
        while self
            .bytes
            .get(self.index)
            .copied()
            .is_some_and(|byte| byte != b'\n')
        {
            self.index += 1;
        }
    }

    fn skip_block_comment(&mut self) -> Result<(), String> {
        let mut index = self.index + 2;
        loop {
            if index + 1 >= self.bytes.len() {
                return Err("unterminated block comment".to_string());
            }
            if self.bytes[index] == b'*' && self.bytes[index + 1] == b'/' {
                self.index = index + 2;
                return Ok(());
            }
            index += 1;
        }
    }

    fn skip_number(&mut self) {
        while self.bytes.get(self.index).copied().is_some_and(|byte| {
            byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'_' || byte == b'$'
        }) {
            self.index += 1;
        }
    }

    fn skip_regexp(&mut self) -> Result<(), String> {
        let mut index = self.index + 1;
        let mut in_class = false;
        loop {
            let Some(byte) = self.bytes.get(index).copied() else {
                return Err("unterminated regular expression literal".to_string());
            };
            match byte {
                b'\\' => index += 1 + self.char_len_at((index + 1).min(self.bytes.len())),
                b'[' => {
                    in_class = true;
                    index += 1;
                }
                b']' => {
                    in_class = false;
                    index += 1;
                }
                b'/' if !in_class => {
                    index += 1;
                    break;
                }
                b'\n' => return Err("unterminated regular expression literal".to_string()),
                _ => index += self.char_len_at(index),
            }
        }
        while self
            .bytes
            .get(index)
            .copied()
            .is_some_and(is_identifier_part_byte)
        {
            index += 1;
        }
        self.index = index;
        Ok(())
    }

    /// Consumes a template body, stopping after its closing backtick or inside a
    /// `${` substitution (which is ordinary source and must keep being scanned).
    fn scan_template_body(&mut self) -> Result<(), String> {
        while let Some(byte) = self.bytes.get(self.index).copied() {
            match byte {
                b'\\' => {
                    self.index += 1;
                    self.index += self.char_len_at(self.index.min(self.bytes.len()));
                }
                b'`' => {
                    self.index += 1;
                    self.slash = SlashMeaning::Divide;
                    self.previous_was_dot = false;
                    return Ok(());
                }
                b'$' if self.bytes.get(self.index + 1) == Some(&b'{') => {
                    self.index += 2;
                    self.depth += 1;
                    self.template_stack.push(self.depth);
                    self.slash = SlashMeaning::Regexp;
                    self.previous_was_dot = false;
                    return Ok(());
                }
                _ => self.index += self.char_len_at(self.index),
            }
        }
        Err("unterminated template literal".to_string())
    }
}

fn is_identifier_start_byte(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_' || byte == b'$' || !byte.is_ascii()
}

fn is_identifier_part_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$' || !byte.is_ascii()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sources_of(
        sources: &[(&str, &str)],
        entry: usize,
        resolutions: Vec<(ModuleUnitId, ModuleRequestIr, ModuleUnitId)>,
    ) -> ModuleGraphSources {
        ModuleGraphSources {
            entry: ModuleUnitId::try_from(entry).expect("entry index fits"),
            modules: sources
                .iter()
                .map(|(key, text)| ModuleSourceIr {
                    key: (*key).to_string(),
                    source_text: (*text).to_string(),
                    meta_url: format!("file:///{key}"),
                })
                .collect(),
            resolutions,
        }
    }

    /// Builds a graph the way the linker does: link, then the two collectors, so
    /// `components` and `namespace` are populated exactly as they are in
    /// production.
    fn graph_of(sources: &ModuleGraphSources) -> ModuleGraphIr {
        let mut graph = crate::modules::build_graph(sources).expect("graph should build");
        crate::modules::link(&mut graph);
        collect_components(&mut graph);
        crate::modules::namespace::collect_observed_namespaces(&mut graph);
        graph
    }

    fn plain(specifier: &str) -> ModuleRequestIr {
        ModuleRequestIr {
            specifier: specifier.to_string(),
            phase: ImportPhaseIr::Evaluation,
            attributes: Vec::new(),
        }
    }

    /// The `d.mjs` shape the lane exists for: a dispatcher that answers the
    /// specifier as written with the target's namespace object, and a namespace
    /// object whose getter reads the exporter's own merged-scope binding.
    #[test]
    fn prelude_serves_the_target_specifier_from_the_exporter_binding() {
        let sources = sources_of(
            &[
                ("a", "export const value = 41;"),
                ("d", "import(\"./a.mjs\").then(m => print(m.value));"),
            ],
            1,
            vec![(1, plain("./a.mjs"), 0)],
        );
        let graph = graph_of(&sources);
        let prelude = graph.dynamic_import_prelude();

        // The dispatcher resolves with the object `modules::namespace` emits,
        // never one of its own — that shared binding is the 16.2.1.10 identity.
        let resolution = format!(
            "if (key === \"./a.mjs\") {{ resolve({}); return; }}",
            module_namespace_cell_name(0)
        );
        assert!(
            prelude.contains(&resolution),
            "dispatcher matches the specifier as written, got: {prelude}"
        );
        assert!(
            prelude.contains("reject(new TypeError("),
            "an unmatched specifier rejects rather than traps, got: {prelude}"
        );
        assert!(
            prelude.contains("var key = `${specifier}`;"),
            "ToString happens inside the executor, got: {prelude}"
        );
    }

    /// One line, always: the merged script displaces every unit's line numbers by
    /// a fixed amount no matter how big the graph is.
    #[test]
    fn the_prelude_is_a_single_line() {
        let sources = sources_of(
            &[
                ("a", "export const value = 41;"),
                ("b", "export const other = 1;"),
                (
                    "d",
                    "import(\"./a.mjs\");\nimport(\"./b.mjs\");\nimport(x);",
                ),
            ],
            2,
            vec![(2, plain("./a.mjs"), 0), (2, plain("./b.mjs"), 1)],
        );
        let graph = graph_of(&sources);
        let prelude = graph.dynamic_import_prelude();
        assert!(!prelude.contains('\n'), "got: {prelude}");
        assert!(!prelude.contains('\r'), "got: {prelude}");
    }

    /// A graph with no `import()` in it contributes nothing, so the linker can
    /// skip the line entirely rather than emit an empty one.
    #[test]
    fn a_graph_without_import_calls_has_an_empty_prelude() {
        let sources = sources_of(&[("a", "export const value = 41;")], 0, Vec::new());
        let graph = graph_of(&sources);
        assert_eq!(graph.dynamic_import_prelude(), "");
    }

    /// A referrer whose only specifier is computed still gets a dispatcher; it
    /// falls through to `reject`, which is a rejected promise and not a trap.
    #[test]
    fn a_computed_specifier_gets_an_always_rejecting_dispatcher() {
        let sources = sources_of(&[("d", "import(x);")], 0, Vec::new());
        let graph = graph_of(&sources);
        let dispatchers = graph.dynamic_import_dispatchers();
        assert!(dispatchers.contains("function $porffor$module$import$0("));
        assert!(!dispatchers.contains("if (key ==="), "got: {dispatchers}");
        assert!(dispatchers.contains("reject(new TypeError("));
    }

    #[test]
    fn a_call_site_is_rewritten_to_the_referrer_dispatcher() {
        let sources = sources_of(&[("d", "import(\"./a.mjs\").then(f);")], 0, Vec::new());
        let graph = graph_of(&sources);
        assert_eq!(
            graph
                .rewrite_dynamic_import_calls(0, "import(\"./a.mjs\").then(f);")
                .expect("rewrite should succeed"),
            "$porffor$module$import$0(\"./a.mjs\").then(f);"
        );
    }

    /// The call site is an expression, so it is not confined to nesting depth
    /// zero the way a declaration is.
    #[test]
    fn a_nested_call_site_is_rewritten() {
        let source = "function load() { return import(\"m\"); }\nimport(\"m\");";
        let sources = sources_of(&[("d", source)], 0, Vec::new());
        let graph = graph_of(&sources);
        let rewritten = graph
            .rewrite_dynamic_import_calls(0, source)
            .expect("rewrite should succeed");
        assert_eq!(rewritten.matches("$porffor$module$import$0(").count(), 2);
        assert!(!rewritten.contains("import("), "got: {rewritten}");
    }

    /// `import.meta` belongs to another owner and must survive untouched, and a
    /// property access named `import` is not the keyword.
    #[test]
    fn import_meta_and_property_access_are_left_alone() {
        let source = "print(import.meta.url); obj.import(1);";
        let sources = sources_of(
            &[("d", "import(x); print(import.meta.url); obj.import(1);")],
            0,
            Vec::new(),
        );
        let graph = graph_of(&sources);
        assert_eq!(
            graph
                .rewrite_dynamic_import_calls(0, source)
                .expect("rewrite should succeed"),
            source
        );
    }

    /// The scanner is lexical, so `import(` that is not code stays put.
    #[test]
    fn import_calls_inside_literals_and_comments_are_left_alone() {
        let source = concat!(
            "const s = \"import('m')\";\n",
            "const t = `${ 1 } import('m')`;\n",
            "// import('m')\n",
            "/* import('m') */\n",
            "const r = /import\\('m'\\)/;\n"
        );
        let sources = sources_of(&[("d", "import(x);")], 0, Vec::new());
        let graph = graph_of(&sources);
        assert_eq!(
            graph
                .rewrite_dynamic_import_calls(0, source)
                .expect("rewrite should succeed"),
            source
        );
    }

    /// A `${}` substitution is ordinary source: a call site inside one is real.
    #[test]
    fn a_call_site_inside_a_template_substitution_is_rewritten() {
        let source = "const t = `${ import(\"m\") }`;";
        let sources = sources_of(&[("d", source)], 0, Vec::new());
        let graph = graph_of(&sources);
        assert_eq!(
            graph
                .rewrite_dynamic_import_calls(0, source)
                .expect("rewrite should succeed"),
            "const t = `${ $porffor$module$import$0(\"m\") }`;"
        );
    }

    /// The replacement is longer than the keyword but adds no line terminator,
    /// so a diagnostic's line number still points at the right line.
    #[test]
    fn rewriting_preserves_line_structure() {
        let source = "print(1);\nimport(\"m\");\nprint(2);\n";
        let sources = sources_of(&[("d", source)], 0, Vec::new());
        let graph = graph_of(&sources);
        let rewritten = graph
            .rewrite_dynamic_import_calls(0, source)
            .expect("rewrite should succeed");
        assert_eq!(rewritten.lines().count(), source.lines().count());
        assert_eq!(
            rewritten.lines().nth(1),
            Some("$porffor$module$import$0(\"m\");")
        );
    }

    /// The lexical scan and boa's parse can only disagree through a method named
    /// `import`. That must fail loudly, not emit a call to a function the
    /// prelude never declared.
    #[test]
    fn a_call_site_the_record_does_not_list_is_reported() {
        let sources = sources_of(&[("d", "print(1);")], 0, Vec::new());
        let graph = graph_of(&sources);
        let error = graph
            .rewrite_dynamic_import_calls(0, "const o = { import() { return 1; } };")
            .expect_err("a disagreement must be reported");
        assert!(error.contains("the module record lists 0"), "got: {error}");
    }

    #[test]
    fn an_unterminated_string_is_reported_rather_than_mangled() {
        let sources = sources_of(&[("d", "import(x);")], 0, Vec::new());
        let graph = graph_of(&sources);
        let error = graph
            .rewrite_dynamic_import_calls(0, "const s = \"import('m')")
            .expect_err("an unlexable source must be reported");
        assert!(error.contains("unterminated"), "got: {error}");
    }

    #[test]
    fn a_linkable_graph_reports_nothing() {
        let sources = sources_of(
            &[
                ("a", "export const value = 41;"),
                ("d", "import(\"./a.mjs\");"),
            ],
            1,
            vec![(1, plain("./a.mjs"), 0)],
        );
        let graph = graph_of(&sources);
        assert_eq!(graph.check_dynamic_import_linkable(), Vec::new());
    }

    /// `import.defer()` must not evaluate eagerly and `import.source()` must not
    /// produce a namespace, so neither can be served by a dispatcher that
    /// resolves to one.
    #[test]
    fn a_non_evaluation_phase_is_reported() {
        let sources = sources_of(&[("d", "import(\"m\");")], 0, Vec::new());
        let mut graph = graph_of(&sources);
        // Built rather than parsed: whether this boa vendors `import.defer(x)`
        // is not what is under test, and the check must hold for any site the
        // record carries.
        graph.units[0]
            .record
            .dynamic_import_sites
            .push(DynamicImportSiteIr {
                static_specifier: Some("m".to_string()),
                phase: ImportPhaseIr::Defer,
            });
        let diagnostics = graph.check_dynamic_import_linkable();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("import.defer()")),
            "got {diagnostics:?}"
        );
    }

    /// The synthesized names live in the same merged scope as user code, so a
    /// user binding that reaches into the reserved prefix is reported rather
    /// than allowed to shadow a dispatcher.
    #[test]
    fn a_user_binding_in_the_reserved_prefix_is_reported() {
        let source = format!("const {LINKER_NAME_PREFIX}namespace$0 = 1;\nimport(x);");
        let sources = sources_of(&[("d", source.as_str())], 0, Vec::new());
        let graph = graph_of(&sources);
        let diagnostics = graph.check_dynamic_import_linkable();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("linker-synthesized name")),
            "got {diagnostics:?}"
        );
    }

    /// An anonymous `export default` binds the unspellable `*default*`, which no
    /// getter body can name in the merged scope.
    #[test]
    fn an_unspellable_export_binding_is_reported() {
        let sources = sources_of(
            &[
                ("a", "export default function () { return 1; }"),
                ("d", "import(\"./a.mjs\");"),
            ],
            1,
            vec![(1, plain("./a.mjs"), 0)],
        );
        let graph = graph_of(&sources);
        let diagnostics = graph.check_dynamic_import_linkable();
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("no name in the merged scope")),
            "got {diagnostics:?}"
        );
    }

    /// Two specifiers naming one module resolve with one binding, so `import()`
    /// twice observes the same object (16.2.1.10).
    #[test]
    fn two_specifiers_for_one_module_share_one_namespace_binding() {
        let sources = sources_of(
            &[
                ("a", "export const value = 41;"),
                ("d", "import(\"./a.mjs\");\nimport(\"a\");"),
            ],
            1,
            vec![(1, plain("./a.mjs"), 0), (1, plain("a"), 0)],
        );
        let graph = graph_of(&sources);
        let prelude = graph.dynamic_import_prelude();
        assert_eq!(
            prelude
                .matches(&format!("resolve({})", module_namespace_cell_name(0)))
                .count(),
            2,
            "got: {prelude}"
        );
    }

    /// The whole point of routing `resolve` through
    /// [`module_namespace_cell_name`]: `import()` and `import * as ns` of one
    /// module name the same binding, so the objects are `===`. This lane must
    /// never mint a namespace binding of its own.
    #[test]
    fn a_dispatcher_resolves_with_the_static_namespace_binding() {
        let sources = sources_of(
            &[
                ("a", "export const value = 41;"),
                (
                    "d",
                    "import * as ns from \"./a.mjs\";\nimport(\"./a.mjs\");",
                ),
            ],
            1,
            vec![(1, plain("./a.mjs"), 0)],
        );
        let graph = graph_of(&sources);
        let prelude = graph.dynamic_import_prelude();
        assert!(
            prelude.contains(&format!("resolve({})", module_namespace_cell_name(0))),
            "got: {prelude}"
        );
        assert!(
            !prelude.contains("$porffor$module$namespace$"),
            "this lane must mint no namespace binding of its own: {prelude}"
        );
    }

    /// A method literally named `import` is indistinguishable from an
    /// `ImportCall` to a lexical scan, so a unit that has *both* a real
    /// `import()` and such a method must be reported rather than have the
    /// method renamed to the dispatcher.
    #[test]
    fn a_hallucinated_call_site_alongside_a_real_one_is_reported() {
        let sources = sources_of(&[("d", "import(\"a\");")], 0, Vec::new());
        let graph = graph_of(&sources);
        let error = graph
            .rewrite_dynamic_import_calls(0, "import(\"a\"); const o = { import() { return 1; } };")
            .expect_err("a count disagreement must be reported");
        assert!(error.contains("the module record lists 1"), "got: {error}");
        // The caller owns the `module {key}:` prefix, so this must not add one.
        assert!(!error.contains("module d:"), "got: {error}");
    }

    #[test]
    fn a_string_literal_escapes_quotes_and_line_separators() {
        assert_eq!(js_string_literal("a\"b\\c"), "\"a\\\"b\\\\c\"");
        assert_eq!(js_string_literal("a\u{2028}b"), "\"a\\u2028b\"");
    }
}
