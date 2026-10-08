# Mixed async-generator ForIn IR

`AsyncGeneratorForInIr` consumes the actual source head identity and eager per-key
head proof, complete mixed head/body ranges, the advance and exit boundaries, and
the exact Await/Yield tape. Even phase-free loops have complete entries. Ordinary
generator and plain async state validators reject this foreign carrier explicitly.

One private `CheckedForInStorageIr` owns the original four distinct invocation
cells, completed head publication, original per-key initializer and optional TDZ
and iteration environments. All three complete ForIn carriers consume that proof.
Its constructor validates unique actual inventory and the sole final retained
head publication, then calls the original eager retained-key/environment validator.
The validated initializer is moved into the proof; a carrier cannot retain an
unvalidated sibling block. Protocol constructors separately check source identity,
ranges and exact suspension tapes.

Mixed lexical heads retain the original TDZ ancestry, and body suspensions retain
their actual iteration record and any nested captured Block. The complete native
carrier uses the original cursor and four cells. Advance evaluates the eager
initializer once for each selected key; resumed body work uses the same key and
StatementList value. ForIn never performs IteratorClose.

Global summary, state/tape, early-error, throw-inference, AOT planning and native
dispatcher readers consume the complete mixed carrier. The outer head-prefix
lowerer defers to the actual checked whole owner so head TDZ precedes suspension.

Seven private controls use parsed and lowered programs to damage cell inventory,
publication, initializer, source identity, body ranges, equal-extent suspension
kinds and original environment domains. They are authored, uncompiled and unrun.
Source review establishes no current runtime acceptance; mandatory joined
verification follows the complete coherent source pass.
