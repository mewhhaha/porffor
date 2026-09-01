# Module-entry source authority

A graph entry has exactly one source authority. `ModuleEntry` is the closed
public domain for the two legal choices:

- `HostLoad { locator }` requires `HostModuleLoader::load` to provide the
  entry; and
- `InMemory { locator, source_text }` uses the embedder's exact text without
  loading that entry through the host.

Both choices retain a locator because host canonicalization and relative
dependency resolution still need stable entry identity. Source authority is
not an `Option<String>` beside that locator: `None` and `Some` do not name the
host operation they select, and the richer record could previously be passed
to an already-parsed entry path where its override was silently ignored.

The parsed module and parsed Script handoffs accept only `entry_locator: &str`
plus their typed parse product. They cannot accept `ModuleEntry`, so no caller
can attach a second source authority after parsing. Dependencies still load
through the same host and every entry locator still passes through
`HostModuleLoader::canonical_key` exactly once.

Focused behavior requires a host entry to read its source through the
filesystem loader and an in-memory entry to succeed with the reject-all loader.
The structure regression fixes the two-variant domain, exhaustive projections,
the two load branches and the narrower parsed handoff:

```sh
cargo test -p lila-engine --test module_entry_source_authority_structure
cargo test -p lila-engine module_loader::tests::a_host_entry_loads_its_source_through_the_loader -- --exact
cargo test -p lila-engine module_loader::tests::an_in_memory_entry_does_not_ask_the_host_to_load_it -- --exact
```

This boundary does not add module types, lazy dynamic evaluation, namespace
exotic internal methods or async module lifecycle support. It makes only the
entry's existing compile-time source choice explicit and unignorable.
