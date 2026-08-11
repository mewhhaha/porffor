// RE-RT (batch 7): the rejected half of the AOT runtime RegExp pattern table.
//
// `(?<x>a)(?<x>b)` is NOT a legal Pattern — the two `x` groups are in the same
// alternative — and `RegExpProgram::compile` answers
// `RegExpCompileErrorKind::InvalidSyntax` for it. Before this lane that verdict
// was thrown away: `StringPool::queue_runtime_regexp_programs` did
//
//     let Ok(program) = RegExpProgram::compile(compilation_source, flags) else {
//         continue;
//     };
//
// so the pattern left no row, the emitted lookup in
// `emit_runtime_regexp_program_slots` fell out of its loop with no else arm, and
// the caller published a live RegExp whose `instruction_count` is 0. Measured on
// the pre-fix head: `new RegExp(computedBadPattern)` returned an object, set
// `source`, and threw nothing at all. That is the one Bug-outcome failure in the
// batch-7 sweep
// (`annexB/built-ins/RegExp/prototype/compile/duplicate-named-capturing-groups-syntax.js`).
//
// Every pattern below reaches its entry point as a *computed* value, so the
// static `porffor-ir` path cannot mask the runtime one. That distinction is the
// whole test: with a literal argument the pre-fix compiler already threw
// correctly, and wrapping the same call in a function made it silent.

function check(actual, expected, label) {
  if (actual !== expected) {
    throw label + ": " + actual + " !== " + expected;
  }
}

var badPatterns = ["(?<x>a)(?<x>b)"];
var bad = badPatterns[0];

// The RegExp constructor.
var constructed = "not-constructed";
var constructorErrorName = "no-throw";
try {
  constructed = new RegExp(bad);
} catch (error) {
  constructorErrorName = error.name;
}
check(constructorErrorName, "SyntaxError", "new RegExp rejects the rejected pattern");
check(constructed, "not-constructed", "no live RegExp survives a rejected pattern");

// RegExp.prototype.compile — the entry point the measured test262 case uses.
//
// Deliberately NOT asserted here: the receiver's `source` after the throw.
// `RegExpInitialize` parses (step 3) before it stores `[[OriginalSource]]`
// (step 4), but this backend stores source and flags first and only then looks
// the program up, so the receiver is left holding the pattern that was refused.
// That is a real ordering defect and it is filed in the lane note; pinning
// today's answer here would make the fixture argue for the bug.
var compileTarget = /[ab]/;
var compileErrorName = "no-throw";
try {
  compileTarget.compile(bad);
} catch (error) {
  compileErrorName = error.name;
}
check(compileErrorName, "SyntaxError", "compile rejects the rejected pattern");

// A String.prototype entry point. Fixing only `compile` would leave the six
// other call sites of `emit_runtime_regexp_program_slots` on the silent path,
// so one of them is exercised directly: `String.prototype.search` with a string
// argument builds a synthetic RegExp from it, which per spec is a RegExpCreate
// and therefore a SyntaxError for an illegal pattern.
var searchErrorName = "no-throw";
try {
  "ab".search(bad);
} catch (error) {
  searchErrorName = error.name;
}
check(searchErrorName, "SyntaxError", "search rejects the rejected pattern");

// The legal sibling, in the same script, must still compile and match. This is
// the anti-vacuity half: `(?<x>a)|(?<x>b)` puts its duplicate names in DIFFERENT
// alternatives, which the spec allows, and a fix that threw for both patterns
// would pass every check above while being wrong.
var goodPatterns = ["(?<x>a)|(?<x>b)"];
var good = goodPatterns[0];
var goodRegExp = new RegExp(good);
check(goodRegExp.source, goodPatterns[0], "the legal duplicate-name pattern still compiles");
check(goodRegExp.test("b"), true, "the legal duplicate-name pattern still matches");

true;
