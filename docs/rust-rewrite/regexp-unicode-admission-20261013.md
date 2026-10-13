# Unicode RegExp admission and immutable runtime identity — 2026-10-13

Status: coherent candidate written in an isolated checkout; native verification,
exact pinned before/after comparison and idle performance measurement pending.
The baseline checkout remains frozen at `4e01400fc` with its original optimized
worker and two complete unfiltered families still running.

## Runtime syntax rule

A `{` encountered in atom position cannot begin a valid Unicode PatternCharacter.
The runtime parser previously accepted failed braced-quantifier probes as literal
text under every character mode. Only the Annex B legacy grammar permits that
fallback. The atom parser now rejects a recognized bare quantifier or any bare
opening brace in `u`/`v` mode. A failed probe after a preceding atom leaves the
cursor at `{`, so the same atom rule rejects incomplete suffixes such as `a{`.
Valid postfix quantifiers and escaped braces retain their existing routes.

The `compile` builtin publishes source, flags and program only after compiler
success. Computed-source tests exercise constructors and recompilation in both
Unicode modes, verify legacy literal matching, and check that failed recompile
preserves source, flags, lastIndex and the old matcher. Original pinned fixtures
and their deadlines are unchanged.

## Immutable runtime digest

`RuntimeArtifact::new` accepts owned immutable bytes and the validated layout.
It computes both the existing layout-prefixed runtime key and the distinct raw
Wasm SHA-256. Emission and fully checked cache decoding use this constructor;
callers cannot supply a digest. Serialization retains its original format.

A borrowed module-source enum chooses program bytes or a runtime artifact.
Runtime memory-cache lookup reads the artifact digest; program lookup continues
to hash its actual bytes. The cache key still contains native compilation mode
and the same raw-byte SHA. Bypass paths, capacity, eviction and native-image
manifest/source/target/config/ABI admission remain on their original routes.
No memory-footprint or speedup claim follows solely from removing a repeat scan.

## Profile boundary metadata

Preparation already includes parsing, admission and Spec IR construction through
the prepared source catalog. The later span admits coded diagnostics and creates
the compilation unit. The scope string and SDK comments now describe these
actual boundaries. Timers and original workloads do not move. Historical raw
profiles remain unchanged and their old scope string cannot enter a new baseline.

## Acceptance

Build the coherent candidate once, run the affected AOT runtime-cache controls,
Engine runtime-link/cache/profile controls, full RegExp native target, and CLI
profile controls. Replay both original failing pinned IDs and the surrounding
RegExp family, retaining exact previous passing IDs and all new failures. Check
runtime digest equivalence for emitted, decoded, Minimal and Custom artifacts;
retain corruption, wrong-ABI, native-mode and retention-bypass controls.

Measure the original twenty-fixture seven-sample profiles only after other native
work drains, using actual candidate image identity and fresh owned outputs.
A complete pinned run and all task-specific acceptance remain separate gates.
