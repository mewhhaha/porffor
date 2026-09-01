# Dynamic-import dispatcher references have one projection owner

The source rewriter has two distinct dispatcher-name authorities:

- `ModuleLocal` selects the dispatcher declared in the merged module wrapper;
- `ScriptEntryExport` selects the outer binding through which a Script entry
  reaches that wrapper-owned dispatcher.

`DynamicImportDispatcherReference` is the private, non-`Clone`, non-`Copy`
domain for that decision. The module and Script public rewriters are its two
producers. They hand it by value to `rewrite_calls`, whose one exhaustive
consuming projection selects the name at each discovered call site. A source
with no call sites may drop the unused authority at the established early
return; any source that is rewritten consumes it before publishing a name.

The domain has six lexical mentions in Rust production sources: its
declaration, owned parameter, two producers and two consumer arms. Each variant
therefore has exactly one producer and one consumer. Removing incidental debug,
clone, copy and equality capabilities makes a preliminary observation followed
by the naming projection a Rust move error. Adding a third dispatcher location
requires an explicit producer and projection arm before the crate builds.

The lexical structure regression ignores Rust comments and string, byte, C,
raw and character literals. It pins the attribute-free declaration, complete
source census, both public wrappers, sole exhaustive match and the full rewrite
body fingerprint:

```sh
cargo test -p lila-ir --test dynamic_import_dispatcher_reference_structure -- --test-threads=1
```

The structure target passes `4/4`. The exact module-local and
Script-entry-export rewrite units each pass `1/1`.

This is source-equivalent ownership hardening. It changes no rewritten source,
module graph, import phase, dispatcher spelling or emitted Wasm, and it makes no
broader module-linking or Test262 conformance claim.
