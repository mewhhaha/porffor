// RE-VERDICT (batch 8): the STATIC half of the two Unicode-mode productions
// `porffor-ir`'s RegExp parser was missing.
//
// Companion to `wasm_regexp_runtime_pattern_valid.js`, which covers the same two
// patterns through the runtime table. Every regexp here is a SOURCE LITERAL, so
// each one takes `lower_regexp_literal` -> `RegExpProgram::compile(...).ok()`.
// A rejected pattern does not throw there; it silently produces
// `program: None`, and the receiver is then left to the fallback matcher, which
// either declines with `TypeError: RegExp.prototype.exec unsupported pattern`
// or answers by luck. Either way this fixture is the literal-path witness that
// the compile-time verdict changed.
//
// The two productions, both of which the class parser already implemented and
// the atom parser did not:
//
//   IdentityEscape[+UnicodeMode] :: SyntaxCharacter | `/`
//   RegExpUnicodeEscapeSequence[+UnicodeMode] :: `u{` CodePoint `}`
//
// `test262/vendor/test262/test/built-ins/RegExp/unicode_identity_escape.js`
// asserts the first in both positions; its line 35 is
// `assert(/\//u.test("/"), "IdentityEscape in AtomEscape: /\\//")`.

function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

// --- IdentityEscape: the SOLIDUS alternative -------------------------------

check(/\//u.test("/"), true, "IdentityEscape in AtomEscape under u");
check(/\//v.test("/"), true, "IdentityEscape in AtomEscape under v");
check(/\//u.test("x"), false, "an escaped solidus matches nothing else");

// The class position, which has always been correct. Its agreement is what
// identified the atom position as the defect rather than the rule.
check(/[\/]/u.test("/"), true, "IdentityEscape in ClassEscape under u");

// The ordinary real-world shape, and the reason this is not a corner case: a
// URL pattern escapes the delimiter twice.
check(/https?:\/\//u.test("https://example.test"), true, "a url pattern matches");
check(/https?:\/\//u.test("ftp://example.test"), false, "a url pattern is not universal");

// The guard against the WRONG fix. If `/` had been added to the SyntaxCharacter
// predicate instead of to the identity-escape rule, an UNESCAPED solidus in a
// pattern would have become an unsupported metacharacter.
check(/a\/b/u.test("a/b"), true, "an escaped solidus in the middle of a pattern");
check(new RegExp("a/b").test("a/b"), true, "an unescaped solidus is still a literal");

// --- RegExpUnicodeEscapeSequence: the braced alternative -------------------

check(/\u{41}/u.test("A"), true, "a braced code-point escape under u");
check(/\u{41}/v.test("A"), true, "a braced code-point escape under v");
check(/\u{41}/u.test("B"), false, "a braced code-point escape matches nothing else");
check(/[\u{41}]/u.test("A"), true, "the class position agrees");

// Unbraced `\uHHHH` is the other alternative and must not have regressed: the
// braced branch is gated on the `{` and must not swallow this one.
check(/\u0041/u.test("A"), true, "the four-digit alternative still works");

true;
