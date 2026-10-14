# Runtime and program helper type ownership

Layout ABI 4 separates helper signature ownership from helper body ownership.
The one `RuntimeHelperId` registry still owns every helper's typed operands,
results, and declaration order. Body/function indices keep their existing
Runtime versus Program/Hook ranks. Type indices use a separate closed region:

- Runtime helpers and program hooks belong to the core type region. The shared
  runtime calls hooks through typed global function references, so their
  signatures remain part of the runtime contract even though P owns the bodies.
- Direct Program helpers belong to a suffix after the complete core GC graph.
  Runtime modules omit those signatures and bodies. Program and standalone
  modules append them as singleton function types referring back to core types.

The static signature prefix and the core recursion group have identical type
indices and exact type declarations in R and P. The assigned GC base depends
only on static signatures and core helper signatures. Adding or editing a
direct Program helper therefore cannot enlarge the core recursion group or
renumber its GC layouts. Adding or editing a hook signature is an actual runtime
ABI change and must invalidate the runtime.

Registration takes the actual module kind, consumes the private type section,
and finalizes the matching root schema as before. No caller supplies type
counts, helper-index lists, or a mutable section. Both signature declaration and
index use derive from the same exhaustive ownership match and registry rows.

The ABI 3 to 4 migration reindexes the existing core once. Layout version 4
invalidates old raw caches, build packages, and runtime keys coherently. Existing
package, Intl profile, pool, Wasm validation, source and native admission checks
remain authoritative. Tests decode real R/P bytes, compare core groups and
import signatures, validate every actual helper row, and retain hook globals
and `call_ref` signatures in the core.

This change does not narrow compiler source fingerprints or introduce a new
trusted native input. Source/build-package identity remains conservative over
the compiler inputs. Exact-Wasm native caches may reuse unchanged R after this
migration; no timing or cache-hit measurement is asserted without a fresh owned
verification run. Extracting a runtime emitter dependency boundary is a
separate change if fresh raw emission and package rewrites need to be avoided.
