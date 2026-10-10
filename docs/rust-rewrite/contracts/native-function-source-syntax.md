# Native function source syntax

The builtin catalog supplies a function's initial public name and its frozen
native source representation. Debug labels describe compiler owners separately.
Each captured legacy RegExp slot allocates one canonical getter, and input
also allocates its setter. All aliases publish the same completed GC function
record. Their initial names use the slot's canonical property (`get input`,
`get lastMatch`, and so on), so punctuation aliases such as `$&` cannot replace
the stored source or introduce invalid NativeFunction syntax. Debug labels
remain separate from these names.

[`Function.prototype.toString`](https://tc39.es/ecma262/multipage/fundamental-objects.html#sec-function.prototype.tostring)
requires NativeFunction syntax and preserves a builtin's captured
`[[InitialName]]`. [`SetFunctionName`](https://tc39.es/ecma262/multipage/ordinary-and-exotic-objects-behaviours.html#sec-setfunctionname)
initializes that name alongside the public `name` property. Changing, replacing
with an accessor, or deleting public `name` later does not change the stored
source string or cause `toString` to read the property. Ordinary builtin names,
accessor prefixes and computed well-known Symbol names retain their existing
representations. There is no anonymous formatting fallback for named builtins.

A const check rejects malformed native names in the closed builtin catalog.
Its admitted spellings are the existing anonymous, ASCII IdentifierName and
computed well-known Symbol forms, with optional getter/setter prefixes. It is a
catalog invariant rather than a parser for arbitrary JavaScript property names.
Exact user-source representations and the existing native materializer are
unchanged.

The frozen published `built-in-function-object.js` case failed both Script modes
while visiting `%RegExp%.input`. The new Engine target embeds that unchanged
source, the complete native-function matcher, and the complete intrinsic
traversal. Separate paired controls retain initial accessor names, alias
identity, Symbol syntax, and source stability across public-name mutation,
throwing getters and deletion. This source proposal is uncompiled and has no
runtime or conformance PASS. The historical task checkpoint and published
failure remain evidence of their own runs; Root must verify the coherent batch.

The cloud checkpoint exposed distinct accessor allocations under aliases and
invalid punctuation-alias source strings. The repair moves allocation before
the alias-publication loop. The original two controls remain unchanged; an
additional paired control checks every legacy alias, distinct captured slots,
canonical getter/setter names, and complete NativeFunction grammar. Execution
keeps the original 60,000-ms deadline. See [the cloud receipt](../cloud-continuation-20261009.md) for
verification; source staging alone is not a PASS.
