# Generator catch environment restoration

## Demonstrated composition failure

Checkpoint8 repairs capture analysis of functions in catch-pattern defaults.
Eight of its ten catch-pattern native tests pass. The already admitted plain
async and plain generator suspension cases expose a separate runtime lifetime
problem. `catch_generator_body_suspension` throws instead of yielding `11` and
`13` from a pattern's reader closures before and after a write to its catch
parameter. The preceding async conditional batch repairs the plain async case;
this overlay repairs the plain generator case without expanding admission.

## Canonical environment owners

A plain generator's existing environment field at header offset 152 remains
its stable invocation Environment Record. A separate traced field at offset
240 saves its active lexical chain; it fits the existing 256-byte header.
Generator allocation initializes the field, and the invocation prologue seeds
it with the invocation root exactly once. Function lexical capture and the
parameter/body environment relationship remain unchanged.

Every existing plain-generator yield producer saves the active chain before
returning: direct `yield`, the two existing direct conditional-yield arms and
`yield*` delegation. Delegation retains its IteratorResult identity and its
existing completion auxiliary flag. No Promise or async-generator resume
protocol is changed.

The same lexical restoration routine serves plain async functions and plain
generators. Its closed execution-kind match selects the corresponding saved
chain and state fields together. Materialized blocks execute only while their
continuation range is active. Fresh entry allocates cells, while resumed entry
finds the existing child of the already restored enclosing record along the
saved chain. Inactive sibling scopes cannot attach a different record.

Both generator catch emitters select the catch entry state after a new throw,
allocate or restore the parameter Environment Record and then select binding
storage. Thrown payload and tag survive environment allocation in dedicated
temporaries. Parameter initialization runs only on new Throw, and body resume
reuses those cells. Return, throw and finalizer dispatch retain the existing
control targets and environment-depth unwinding.

## Verification boundary

The existing `aot_catch_pattern_owners` generator regression is the original
reproducer. Six new `aot_generator_catch_environment` native tests exercise
simple catch reader identity, sibling captured block records, separate default
parameter/body/catch cells, delegated yields, return and throw through a
suspending captured finalizer, and interleaved generator instances.

The stage is formatted and reviewed without running Cargo, the native runner
or Test262. Those checks remain pending the integrating checkpoint. Published
suite counts are unchanged.

The overlay changes no generator lowering plan or admission rule. The four
separately recorded unsupported generator and async-generator probes remain
separate work. It does not establish complete generator, labelled-block,
`with`-environment or async-generator suspended-scope support.
