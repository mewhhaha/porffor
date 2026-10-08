# Mixed async-generator Switch IR

`AsyncGeneratorSwitchIr` consumes the actual mixed source's discriminant,
CaseBlock entry, optional lazy selector regions, fallback, source-order bodies
and exit. Even eager phases have entries. Its private constructor checks every
range against actual checked mixed regions and compares the complete Await/Yield
tape, including suspension kinds and unique resume states. The original ordinary
generator and plain async validators reject this foreign carrier explicitly.

The ordinary and mixed complete Switch carriers retain one private
`CheckedSwitchStorageIr`. Its constructor requires two distinct, uniquely
allocated invocation cells, a sole final retained discriminant publication and
the original shared uninitialized CaseBlock environment with unique names and
slots. Invocation scratch names cannot enter that environment. No per-case
environment can be published by the case constructor.

The native backend consumes the complete owner through one closed protocol
adapter and the original two-cell selection, fallthrough, StatementList value
and cleanup lifetime. Discriminant work completes outside CaseBlock. Selectors
and bodies use the source certificate's original CaseBlock ancestry. The global
metadata, state/tape, early-error, throw-inference and dispatcher readers consume
the new carrier explicitly, including original lexical declarations.

Six private controls use actual parsed and lowered programs to damage cell
inventory, discriminant publication, case/default order, suspension kinds,
outward head branches and environment ownership. These controls are authored,
uncompiled and unrun. Current source review establishes no runtime acceptance;
mandatory joined verification follows the complete coherent source pass.
