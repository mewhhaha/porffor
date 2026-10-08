# Computed import calls in Script entries

This separate next-batch proposal retains a Script's one-node module graph
when its retained AST contains an import call but discovery finds no literal
target. `ModuleSourceIr::script_has_dynamic_import_sites` distinguishes that
state from an ordinary Script containing an object or class method named
`import`. The Engine still uses the lexical fast gate, the original Script
parse, the existing loader policy and the existing canonical module driver.

Operand evaluation and GetValue happen before the Promise is created; their
abrupt completion stays synchronous and stops options evaluation. Both
operands are evaluated before specifier coercion. A ToString failure rejects
the Promise with the original thrown value and stops attribute property reads.
An unknown computed name has no compiled module target and uses the existing
dispatcher rejection; reactions run after the Script body. Each call retains
its own Promise. No runtime source parser or alternate execution engine is
introduced.

The existing loader constructor produces an entry at index zero, an empty
resolution table and one retained Script source when there are no discovery
requests. This is a valid graph, not a missing graph. Literal request discovery
and target loading remain unchanged. The phase-aware source-import Script
driver boundary remains a separate T12 requirement.

Four Engine controls cover synchronous getter/unbound-reference failure,
ToString ordering and rejection identity, fresh asynchronous unknown-target
rejections under RejectAll, and ordinary object/class methods named `import`.
Each is declared for sloppy and strict Scripts (eight observations). These
controls are UNEXECUTED. Source formatting and static review do not establish
product acceptance or any of the historical 220 frozen-publisher outcomes.

The proposal must join a future complete source batch only after current Intl
MAIN admission. Required future verification is one focused compile, this
four-control Engine target, existing Script/module import-job regressions,
the unchanged exact pinned witnesses, and the prescribed broad checkpoint.
It does not close T12, T07 or T26, change pinned counts, or publish status.
