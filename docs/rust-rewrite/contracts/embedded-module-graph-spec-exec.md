# Spec-exec embedded graph adapter

Spec-exec consumes the same immutable `Arc<EmbeddedModuleGraph>` selected by the
embedded host policy. `EmbeddedGraphModuleLoader` loads only the validated
owner's exact declared Source Text or JSON module records. It performs no filesystem,
network, working-directory, URL-base, path-normalization or Test262 harness
lookup. Complete canonical attributes and the typed referrer remain part of
resolution. A missing row is a host TypeError, including in created Realms.
Lone UTF-16 surrogates in runtime request or attribute strings reject instead of
being escaped into a different, potentially declared backslash-u string.

The actual root Script producer couples goal, exact source and optional supplied
identity to the graph entry before parsing. An omitted diagnostic filename uses
the declared identity. Both execute and observe use this producer. The actual
root Module producer performs the same admission and parses through the graph
loader's dependency producer; it installs that real Module in its Realm's parsed
cache before loading a cyclic graph. A Script and a same-locator Module remain
different referrer roles and retain different source bodies.

Each root session or agent thread has one Embedded Boa Context. Embedded-created
Realms use its `Context::create_realm`, rather than another Context builder. The
private host-Realm handle distinguishes an Embedded `Realm` from an ambient
separate `Rc<RefCell<Context>>`; its policy match cannot silently give an Embedded
Realm another interner. The created Realm's actual module cache is registered
before host globals or user source run. Host actions enter that Realm and restore
the prior Realm after success or error. Embedded eval restores it before draining
jobs, whose Boa producers retain the actual execution Realm. All Realms in that
Context therefore share the one parser/linker interner and job domain. Agents
have independent thread-local Contexts and cannot exchange Boa objects.

This ownership is required by the pinned Boa implementation. `Module::parse`
interns AST/import/export symbols in its supplied Context even when an explicit
Realm is supplied. Subsequent linking and export resolution read those symbols
through the executing Context's interner. Merely parsing under the origin Realm
with another Context can give cached symbols a different meaning. The adapter
keeps those operations within one actual interner domain; it does not copy or
reconstruct an interner or alter vendored APIs.

The Context-installed loader retains private actual Realm/cache owners. Repeated
declared aliases to one identity reuse that Realm's real Module and namespace.
Different Realms have independent parsed instances even though their immutable
source owner and interner are shared. Boa calls the loader before recording or
asserting a dynamic target in the actual referrer's loaded-module map. Module,
Script and Realm callbacks therefore select their own actual Realm's existing
cache, rather than whichever Realm invokes the callback. A first target is parsed
with that cache owner's Realm supplied to `Module::parse`. Later foreign linking,
static imports and reexports still use the same Context/interner; repeated
callbacks retain namespace identity and first target globals/prototypes belong
to the source Realm. No source registry or replacement Module records are added.

The admitted record kind selects `Module::parse` or Boa's real
`Module::parse_json` factory. JSON parsing temporarily enters the actual cache
owner's Realm, creates data and the synthetic default-export record there, and
restores the prior Realm on either outcome. Parser errors become their original
opaque error objects before restoration. A borrowed foreign import callback
therefore cannot allocate JSON data or SyntaxError prototypes in the caller's
Realm. The existing per-Realm cache retains repeated JSON namespace/default
identity; separate Realms retain separate parsed values.

Module identities first project through the actual Realm/cache record. The exact
retained source locator can then project against the declared graph after that
fast path. All Module producers reachable in this fresh session are the
graph-backed parsers and validated entry factory. A synthetic JSON module has no
source locator; its identity is retained by its actual Realm/cache entry. A
source locator is never normalized,
joined with a request, or inferred from metadata. Absent, non-UTF-8 or undeclared
Module locators deny resolution. Script referrers use their exact retained
locator; unlocated Scripts and Realm referrers use only explicit Unlocated rows.

`init_import_meta` projects the same actual Module identity and writes the
independently declared graph URL to Boa's fresh null-prototype metadata object.
This includes first metadata access inside a sibling-Realm callback. Boa's hook
returns `()`, so it cannot return a JavaScript error for an unowned native Module.
The adapter withholds `url` for that case rather than inventing a URL or panicking;
it cannot arise from this session's actual Module producers. Resolution remains
an explicit error for the same absent/undeclared locator.

The host resolution key is phase-free while Boa retains the original request
phase. The loader parses a requested Source Text Module and leaves Source,
Defer, link and evaluation behavior with the existing module operations. Dynamic
Source rejects the unavailable Source Text Module representation after loading;
it does not evaluate the target. A later Defer/Evaluation request reuses the same
parsed source. JSON records likewise have no source-phase representation. The
adapter adds no positive ModuleSource facility, text/bytes record, second module
graph or new Realm/object representation.

The host store borrow ends before running user code, so a Realm can create or
enter another Realm while retaining the graph policy. Ambient separate Contexts
retain their lifetimes behind `Rc<RefCell<Context>>`; `try_borrow_mut` reports a
host TypeError for the same active Context instead of a store-level RefCell panic.
Filesystem and RejectAll retain their meanings and separate Context route. The
existing Test262 filesystem loader and harness stay confined to Filesystem.
Existing root prelude/load/link/evaluation and structured completion positions
are preserved; denied static loads and loaded parser failures remain execution
errors, while dynamic import rejections stay in the normal Promise route.

Nine finite authored controls cover cycles and dynamic aliases, separate URLs,
computed attributes and exact strings, Script self-import, available undeclared
ambient source denied from root/nested Realms/agents, explicitly declared
Unlocated imports with fresh Realm/session instances, Source/Defer/Evaluation
reuse, foreign Module/Script namespace identity and target Realm, metadata, a
cached Source target linked by a foreign callback, a fresh foreign static import
and reexport of that cached origin dependency, and invalid loaded source/root
mismatch before evaluation. These sources have not run. Compilation, focused
execution and whole-batch checks remain pending with the Engine/IR and v4 replay
adapters.

Three additional, unrun JSON controls cover default-only linking and alias
identity, data descriptors and duplicate-key order, exact UTF-16 and numeric
values, dynamic parse/request rejection timing, unavailable source phase, and
borrowed foreign data/error Realm ownership. They use the typed graph factory
and the original observed execution APIs.
