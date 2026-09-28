# Compiler contracts

These documents specify algorithm, type, Realm, lifetime and ownership
invariants. They are durable references for source and regression tests, not
an active task queue or a source of current pass counts. Use the
[measured failure backlog](../../../tasks/README.md) for current repair work and
the [architecture index](../README.md) for an overview.

Start with the relevant boundary:

- [Spec operations](spec-operations.md) and
  [iterator protocol](iterator-protocol.md)
- [Reference records](reference-records.md) and
  [environment lifecycle](environment-record-binding-lifecycle.md)
- [Numeric conversion codomains](numeric-conversion-codomains.md)
- [Property descriptors](property-descriptor-closed-domain-proposal.md)
- [Function protocol](function-protocol.md)
- [IteratorClose obligations](iterator-close-obligation.md)
- [Module instantiation](module-instantiation.md),
  [async lifecycle](module-async-lifecycle.md), and
  [entry completion](module-entry-completion.md)
- [Typed-array buffer witnesses](typed-array-witness-use-ownership.md)
- [Heap collector policy](heap-collector-policy-authority.md)
- [Intl NumberFormat](intl-numberformat-wasm.md)
- [Test262 execution identity](test262-execution-identity.md),
  [snapshot admission](test262-checkpoint-run-identity-admission.md), and
  [aggregate evidence](test262-aggregate-evidence-requirement.md)

Other filenames name the particular operation or ownership boundary. Locate
one with `rg --files docs/rust-rewrite/contracts`, then follow its source and
regression references. A dated count or verification note inside an older
contract describes that revision; it does not establish the current compiler's
conformance. Keep live invariants here and new status/evidence in the backlog.
