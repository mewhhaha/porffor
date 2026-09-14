# RegExp word-boundary assertions

`\b` succeeds when the characters immediately before and after the current
match position have different word membership. `\B` succeeds when that
membership is equal. An absent character is not a word character, so `\B`
succeeds on empty input. Neither assertion changes the match position or
captures. These rules follow
[CompileAssertion](https://tc39.es/ecma262/multipage/text-processing.html#sec-compileassertion)
and [IsWordChar](https://tc39.es/ecma262/multipage/text-processing.html#sec-iswordchar).

The compiler interns the ASCII WordCharacters set after applying its existing
case-closure operation. The closure uses the locally active `i` modifier and
Unicode mode; Unicode folding therefore includes the long-s and Kelvin-sign
ASCII equivalents while legacy matching excludes them. The instruction carries
that range slice and a closed Boundary/NonBoundary polarity. No runtime source
parsing or special handling of particular patterns is involved.

The matcher reads adjacent characters through its existing UTF-8 decoder and
canonical range search. Legacy positions between surrogate halves inspect the
adjacent UTF-16 units; Unicode positions inspect complete code points. A
separate scratch cursor keeps the match position unchanged in both forward and
reverse assertion contexts. Failure uses the existing capture-restoring
backtracking path.

Direct assertion quantifiers are syntax errors. Grouped assertions are nullable
and use the existing progress guards for repetition. Character-class `\b`
retains its consuming backspace meaning. IR and native tests cover polarity,
scoped folding, quantifier grammar, lookbehind, empty input, captures, astral and
lone-surrogate positions, and zero-width global/sticky results.

Lookbehind also accepts the existing `\s` and `\S` consuming instructions.
Forward and reverse matching share one ECMAScript WhiteSpace/LineTerminator
classifier. The reverse matcher uses its existing character decoder and cursor
movement, including one UTF-16 unit in legacy mode and a complete code point in
Unicode mode. Tests cover the complete whitespace set, excluded Unicode spaces,
complement matching, captures across surrogate positions, repetitions, and
nested forward/reverse assertions.

Forward `\S` uses the same successful-character cursor transition as dot and
negative ASCII classes. Unicode matching consumes a complete scalar and reports
its UTF-16 width; legacy matching retains the byte position between an astral
scalar's two UTF-16 units. The instruction cursor always advances by one.
The frozen fourth-checkpoint probe reproduced truncated Unicode captures and
failure to match two legacy `\S` atoms against one astral scalar. Native tests
also cover sticky/global iteration, lone surrogates and backtracking.
