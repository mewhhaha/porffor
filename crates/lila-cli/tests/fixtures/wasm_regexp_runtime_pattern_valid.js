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

// (c) A computed valid pattern whose string NEVER appears as a literal.
//
// `"(?<q1>x)"` is assembled at run time, so no row exists for it. The POLICY
// this lane chose, and this is the assertion that pins it:
//
//   a table miss does NOT throw.
//
// The alternative — throw SyntaxError on any miss — was rejected on measured
// grounds: a zeroed program does not mean `exec` answers `null`, it means
// `emit_regexp_exec_simple_from_locals` (a real fallback matcher) gets its turn,
// and only when that declines does exec throw
// `TypeError: RegExp.prototype.exec unsupported pattern`. Throwing at
// construction would convert every answer that fallback gets right — including
// the ordinary `new RegExp("a", computedFlags)` shape — into a spurious
// SyntaxError.
//
// Measured on the batch-7 head: construction succeeds, `source` is exact, and
// `test` throws that TypeError. What must never happen is the third option: a
// quiet `false` for a pattern that should match. That is the wrong-answer class,
// and it is what the last check below forbids.
// The halves are read out of an array rather than concatenated as two literals
// on purpose: `"(?<" + "q1>x)"` is a constant-foldable expression, and a fold
// would turn this into an ordinary literal, put it in the candidate set, and
// make the whole "unseen" case vacuous. Element reads cannot be folded, so the
// concatenated pattern genuinely never exists at compile time. (Both halves are
// themselves illegal patterns and pick up `Rejected` rows; neither is ever used
// as one.)
var unseenParts = ["(?<", "q1>x)"];
var unseenHead = unseenParts[0];
var unseenTail = unseenParts[1];
var unseen = unseenHead + unseenTail;

var unseenRegExp = null;
var unseenConstruction = "ok";
try {
  unseenRegExp = new RegExp(unseen);
} catch (error) {
  unseenConstruction = "threw:" + error.name;
}
check(unseenConstruction, "ok", "policy: an unseen but valid pattern does not throw");
check(unseenRegExp.source, unseenHead + unseenTail, "an unseen pattern keeps its source");

var unseenAnswer;
try {
  unseenAnswer = unseenRegExp.test("x") ? "match" : "no-match";
} catch (error) {
  unseenAnswer = "threw:" + error.name;
}
check(
  unseenAnswer === "no-match",
  false,
  "an unseen valid pattern must never answer a silent no-match"
);

true;
