# Reentrant RegExp execution

`RegExpBuiltinExec` converts `lastIndex` to a length before reading the current
internal flags or matcher. `RegExp.prototype.compile` called during that
conversion can replace the entire program, its captures, named groups and flags.
The execution uses the replacement, even when it changes the matching path.

The shared intrinsic execution wrapper first checks the receiver and converts
the input to a string, then reads and coerces `lastIndex` exactly once. It lends
the resulting local to each matching path. Those paths read the current flags
and immutable program only after conversion has completed. A thrown conversion
preserves the thrown value and any recompilation that already occurred.
Global/sticky execution uses the converted index; an ordinary expression starts
at zero after performing the same observable conversion.

The focused `aot_regexp_recompile` regressions cover replacement capture/group/
indices metadata, enabling and removing global matching, switching between
static and computed patterns, input-conversion order and thrown-value identity.
The result-mode structural guard retains its ownership and exact projection
checks with the borrowed converted-index argument. Native execution of this
follow-up is pending the next immutable compiler checkpoint.

The ordering follows [ECMA-262 RegExpBuiltinExec](https://tc39.es/ecma262/multipage/text-processing.html#sec-regexpbuiltinexec).
