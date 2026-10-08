# Native function source syntax

The builtin catalog supplies a function's initial public name and its frozen
native source representation. Debug labels describe compiler owners separately.
The shared legacy RegExp accessor bodies retain their existing function identity
and installation under all existing aliases; their native names are `get input`
and `set input`. The former debug phrases `get RegExp legacy static` and
`set RegExp legacy static` cannot appear as native property names because they
contain additional identifiers separated by spaces.

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
