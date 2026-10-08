// RE-RT (batch 7): the accepted half of the AOT runtime RegExp pattern table.
//
// Every pattern here reaches the RegExp constructor / `compile` / a
// String.prototype entry point as a *computed* value, so none of them can take
// the static-lowering path in `lila-ir` (`try_lower_static_regexp_compilation`
// only fires on a literal argument). They all go through
// `emit_runtime_regexp_program_slots`, which looks the `(source, flags)` pair up
// by string value in the table the AOT build wrote.
//
// The companion fixture `wasm_regexp_runtime_pattern_invalid.js` covers the
// rejected half.

function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

// (a) A computed VALID pattern whose string literal DOES appear in the script.
//
// The literal lives in an array literal, which is one of the shapes
// `StringPool::collect` feeds into `runtime_regexp_candidate_literals`, so the
// compile-time RegExp compiler is offered this exact pattern and produces a
// program row. The value reaching the constructor is `seenPatterns[0]` — an
// element read, not a literal — so the lookup is genuinely done at run time.
//
// This pattern is also the anti-vacuity guard for the whole lane.
// `(?<x>a)|(?<x>b)` is LEGAL: duplicate capture-group names are permitted when
// the groups are in different alternatives (lila-ir `regexp.rs`
// `duplicate_names_diverge`). A "fix" that made every duplicate-named pattern
// throw SyntaxError would turn the invalid fixture green and this one red.
var seenPatterns = ["(?<x>a)|(?<x>b)"];
var seen = seenPatterns[0];

var seenRegExp = new RegExp(seen);
check(seenRegExp.source, seenPatterns[0], "computed valid pattern keeps its source");
check(seenRegExp.test("a"), true, "first alternative matches");
check(seenRegExp.test("b"), true, "second alternative matches");
check(seenRegExp.test("z"), false, "a non-alternative does not match");

// The same pattern through RegExp.prototype.compile, which is a different one of
// the seven call sites of `emit_runtime_regexp_program_slots`.
var compileTarget = /[ab]/;
compileTarget.compile(seen);
check(compileTarget.source, seenPatterns[0], "compile installs the computed valid pattern");
check(compileTarget.test("b"), true, "the recompiled receiver matches");

// A String.prototype entry point, which builds a synthetic RegExp from a string
// and hits the table through yet another call site.
var simplePatterns = ["ab"];
check("zzab".search(simplePatterns[0]), 2, "search compiles a computed pattern");

// (b) RE-VERDICT (batch 8): a computed VALID pattern that the compile-time
// RegExp compiler used to answer `InvalidSyntax` for.
//
// This is the half of the batch-7 runtime table that turns a compiler verdict
// into a thrown SyntaxError. `RegExpCompileErrorKind::InvalidSyntax` becomes
// `CandidateOutcome::Rejected`, becomes `RuntimeRegExpEntryKind::Rejected`,
// whose `throws_syntax_error()` is true — so before the fix these two
// constructions threw `SyntaxError` for perfectly legal patterns, and after it
// they must build real programs.
//
//   `\/` under `u` — `IdentityEscape[+UnicodeMode] :: SyntaxCharacter | `/``.
//                    The SOLIDUS alternative was missing from the atom parser.
//   `\u{41}` under `u` — `RegExpUnicodeEscapeSequence[+UnicodeMode] ::
//                    `u{` CodePoint `}``. Only the class parser had it.
//
// Both reach the table as computed values (element reads), and both literals
// appear in array literals so `StringPool::collect` offers them to
// `runtime_regexp_candidate_literals`. The flags literal `"u"` is what puts the
// `u` column in the `|literals| x |flags|` table at all.
var unicodePatterns = ["\\/", "\\u{41}"];
var unicodeFlags = ["u"];

var solidus = new RegExp(unicodePatterns[0], unicodeFlags[0]);
check(solidus.source, unicodePatterns[0], "the escaped solidus keeps its source");
check(solidus.flags, unicodeFlags[0], "the escaped solidus keeps its flags");
check(solidus.test("/"), true, "an escaped solidus matches a solidus");
check(solidus.test("x"), false, "an escaped solidus matches nothing else");

var codePoint = new RegExp(unicodePatterns[1], unicodeFlags[0]);
check(codePoint.source, unicodePatterns[1], "the braced code-point escape keeps its source");
check(codePoint.test("A"), true, "a braced code-point escape matches its code point");
check(codePoint.test("B"), false, "a braced code-point escape matches nothing else");

// (c) A clean computed pattern whose complete source never appears in the
// finite candidate table still compiles through the Wasm runtime compiler.
// This prevents a blanket table-miss rejection from satisfying the separate
// unsupported-grammar control in wasm_regexp_runtime_gap_named_group.js.
var unseenClean = String.fromCharCode(0x7a) + "+";
var unseenCleanRegExp = new RegExp(unseenClean, "u");
check(unseenCleanRegExp.source, unseenClean, "uncached clean source");
check(unseenCleanRegExp.test("zz"), true, "uncached clean pattern matches");
check(unseenCleanRegExp.test("x"), false, "uncached clean pattern rejects");

true;
