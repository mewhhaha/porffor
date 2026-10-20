# Canonical globals after a Script prelude — 2026-10-20

The prelude split reconstructed the initial global object from standard globals
and explicitly referenced host functions. It omitted `parseInt` and `parseFloat`
when code inspected them only through the global object. Their descriptors and
own-property names were therefore missing in the Test262 entry Script.

Bootstrap now restores canonical every-Realm host globals independently of
identifier references. Entry function, var and lexical declarations still wait
until after the prelude. Both Product and Test262 policies allow these ECMAScript
globals; entry-only host extensions retain their existing scope.

The isolated ordinary optimized Wasm-AOT compiler passes all ten original failing
execution IDs and four retained Number alias/descriptor controls, with fresh
per-case state and the normal 60-second deadlines. All 742 IR integration tests
pass, including two new regressions for unreferenced globals and delayed entry
declarations. The product dependency graph contains no interpreter oracle.

The tested source copy uses the same crate bytes with this patch; its Cargo
patch paths refer to the same unchanged vendor sources. The actual measured
compiler retains its unversioned-archive identity and source fingerprint in the
[receipt](global-prelude-hosts-fix-20261020.json). It is not relabelled as this
Git commit. [Focused evidence archive](../../test262/snapshots/canonical-global-prelude-fix-20261020.tar.gz)
preserves exact native snapshots, commands, build output, IR results, original
sources and patch with a verified SHA-256 inventory.

The completed baseline and README chart retain the original compiler's literal
outcomes. This repair changes the source after that measurement. A new complete
Test262 campaign for the fixed compiler and the other native/release acceptance
gates remain pending. [Full baseline](test262-current.md).
