# Filesystem module loading boundary

FilesystemModuleLoader now applies its configured-root check at both resolution
and load. Public HostModuleLoader::load calls and HostLoad entry modules do not
have to pass through resolve, so a previously resolved key is not a sufficient
read authority. The loader revalidates the path immediately before reading its
source and returns ModuleLoadError::Denied when it resolves outside the root.

The regression cohort covers direct outside keys, HostLoad entries, sibling
paths sharing the root's text prefix, valid inside loads, and a resolved file
replaced with an outside symlink before load. The in-memory source authority
and custom HostModuleLoader implementations are unchanged.

This is a bounded repair, not a claim of an adversarial filesystem sandbox:
canonicalization followed by an ordinary filesystem read still has a check/open
race against concurrent filesystem mutation. A future stronger boundary needs
handle-relative filesystem capabilities rather than more string checks.

Validation:

```sh
cargo test --locked -p lila-engine --lib module_loader::
```

No published Test262 status or execution denominator changes.
