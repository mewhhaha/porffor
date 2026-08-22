function check(actual, expected, label) {
  if (actual !== expected) throw label;
}

function exec(re, input, expected, captures, index, label) {
  var result = re.exec(input);
  check(result !== null, true, label + " result");
  check(result[0], expected, label + " value");
  check(result.index, index, label + " index");
  check(result.length, captures.length + 1, label + " capture count");
  for (var i = 0; i < captures.length; i = i + 1) {
    check(result[i + 1], captures[i], label + " capture " + i);
  }
}

// RepeatMatcher must reject an optional iteration that succeeds without
// advancing, while still permitting a later iteration to consume input.
exec(/(a?b??)*/, "ab", "ab", ["b"], 0, "exact Test262 progress");

// The rejection is local to the empty optional iteration. Ordered choice must
// still backtrack through an earlier consuming iteration to satisfy a suffix.
exec(/(a?b??)*b/, "ab", "ab", ["a"], 0, "suffix backtracking");

exec(/(a?)*a/, "aa", "aa", ["a"], 0, "greedy nullable repeat");
exec(/(a?)*?a/, "aa", "a", [undefined], 0, "lazy nullable repeat");

// Required iterations may match empty. Only a subsequent optional iteration
// is subject to the zero-progress rejection.
exec(/(a?){2,}b/, "b", "b", [""], 0, "required empty minima");
exec(/(a?){2,}?b/, "aab", "aab", ["a"], 0, "lazy required minima");
exec(/^(a?){2,4}b$/, "b", "b", [""], 0, "bounded nullable control");

exec(/((a?)*)b/, "aab", "aab", ["aa", "a"], 0, "captured nullable repeat");
exec(/((a?)*)*b/, "aab", "aab", ["aa", "a"], 0, "nested nullable repeat");

// Lookbehind compiles the same repeat in reverse and must carry the same
// progress rule rather than silently relying on forward instruction order.
exec(/(?<=(a?)*)b/, "aab", "b", ["a"], 2, "reverse nullable repeat");

exec(/(a?b??)*/, "", "", [undefined], 0, "empty overall match");
check(JSON.stringify("bb".match(/(a?)*/g)), JSON.stringify(["", "", ""]), "global empty progress");
check(JSON.stringify("bb".match(/(a?)*?/g)), JSON.stringify(["", "", ""]), "lazy global empty progress");

true;
