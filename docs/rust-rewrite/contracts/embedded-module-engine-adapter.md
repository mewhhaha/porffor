# Embedded module graph compiler adapter

`ModuleLoadingPolicy::Embedded` supplies the same immutable graph owner to the
Engine's preparation, compilation unit, agent compile policy and spec-exec
handoff. Policy clones preserve its `Arc`; they do not restore an ambient loader.
Engine root admission checks the source, parse goal and optional filename against
the declared entry before parsing. An omitted filename becomes the declared
identity. A private parsed-entry owner is the only input accepted by catalog
assembly, so loading reuses that entry parse rather than pairing unrelated source
and graph identities.

The private prepared-module enum pairs an Embedded `Arc` with a required catalog,
while its ambient variant carries a closed ambient policy and optional discovered
graph. Cache reads borrow that catalog; exhaustive lowering consumes the same
variant and projects the compilation unit's public loading policy. An Embedded
preparation with no catalog cannot be constructed.

The compiler parses every canonical Module row once and retains each parse
outcome. It consumes the complete exact resolution catalog, including targets
not discoverable from literal import operands. Script and Module referrers at the
same locator remain distinct. Metadata URLs are observations only; this route
does not normalize paths, derive edges, read files or substitute missing rows.
The existing request constructor preserves UTF-16 attribute key order.

Complete-catalog lowering projects the entry's static closure and declared
dynamic candidates through the existing module records, canonical activations,
namespace cells and import dispatchers. Unused rows do not acquire eager bodies.
Source requests parse their record before rejecting unavailable source
representation, and open no outgoing dependencies. Evaluation and defer requests
retain their own phases. Malformed or unlinkable dynamic-only candidates become
the existing import-job rejection cells; static failures still reject compilation.
Private scalar-eligibility facts from the actual AST/interner prevent rendered
surrogates from aliasing declared literal backslash requests or attributes.
Unknown runtime operands are checked by the existing exact JS dispatcher.

Agent source uses a separate private `UnlocatedScript` entry route. It shares
the root graph but can match only declared Unlocated rows; it cannot impersonate
the root Script or infer a locator from the graph's entry. Its parsed source
keeps an absent filename. Agent options, prepared compilation and compilation
units retain the same graph owner across cache retries and worker handoffs.

The additive Embedded cache discriminator consumes the full graph fingerprint,
including unused source/edge rows and independent metadata URLs. Unlocated agent
compilation has a separate Embedded key domain. Filesystem and RejectAll retain
their original discriminators, framing and module-discovery behavior; their
loader capability excludes an Embedded graph by construction.

Finite Engine controls cover computed-only and runtime-attributed requests,
astral/BMP key order, once-only canonical evaluation, Script self-import, metadata
URL independence, unused malformed rows, exact string denial, source/defer jobs,
dynamic rejection cells, entry cycles, actual agent policy propagation and cache
changes from unused authority. They are authored source controls. Compilation,
Wasm validation, runtime, tests and full-batch verification remain pending.
