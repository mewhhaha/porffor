(function () {
  function check(actual, expected, name) {
    if (actual !== expected) throw new Error(name);
  }
  var counts = [0, 1, 2, 3, 7, 8, 9, 31, 32, 33];
  for (var i = 0; i < counts.length; i++) {
    var expected = '';
    for (var j = 0; j < counts[i]; j++) expected += 'a\ud800b';
    check('a\ud800b'.repeat(counts[i]), expected, 'repeat doubling boundary');
  }
  check('ab'.repeat(2.9), 'abab', 'fractional repeat');
  check('ab'.repeat(NaN), '', 'NaN repeat');
  check('ab'.repeat(-0), '', 'negative zero repeat');
  check(''.repeat(Number.MAX_VALUE), '', 'empty finite huge repeat');
  for (var bad of [-1, Infinity, -Infinity]) {
    var caught = false;
    try { ''.repeat(bad); } catch (e) { caught = e instanceof RangeError; }
    check(caught, true, 'invalid repeat count');
  }
  check(''.padStart(1, 'ab'), 'a', 'empty receiver partial pattern');
  check(''.padEnd(2, 'ab'), 'ab', 'empty receiver exact pattern');
  check(''.padStart(3, 'ab'), 'aba', 'empty receiver doubled tail');
  check(''.repeat(0), '', 'empty zero repeat');
  check('xy'.padStart(9, 'abc'), 'abcabcaxy', 'prefix truncation');
  check('x'.padEnd(8, 'ab'), 'xabababa', 'suffix truncation');
  check('\udc00'.padStart(4, '\ud800'), '\ud800\ud800\ud800\udc00', 'start UTF16 seam');
  check('\ud800'.padEnd(4, '\udc00'), '\ud800\udc00\udc00\udc00', 'end UTF16 seam');
  check('x'.padStart(1e30, ''), 'x', 'empty filler before extent error');
  var caught = false;
  try { 'x'.padStart(1e30, 'a'); } catch (e) { caught = e instanceof RangeError; }
  check(caught, true, 'large nonempty pad');
  var log = '';
  var receiver = {toString: function () { log += 'r'; return 'ab'; }};
  var count = {valueOf: function () { log += 'n'; return 3; }};
  check(String.prototype.repeat.call(receiver, count), 'ababab', 'repeat coercion result');
  check(log, 'rn', 'repeat coercion order');
  log = '';
  var target = {valueOf: function () { log += 't'; return 7; }};
  var filler = {toString: function () { log += 'f'; return 'xy'; }};
  check(String.prototype.padStart.call(receiver, target, filler), 'xyxyxab', 'pad coercion result');
  check(log, 'rtf', 'pad coercion order');
  log = '';
  var never = {toString: function () { throw new Error('unused filler coercion'); }};
  check(String.prototype.padEnd.call(receiver, 1, never), 'ab', 'no gap');
  check(log, 'r', 'no gap skips filler coercion');
  var sentinel = {};
  var caught = false;
  try { String.prototype.repeat.call(receiver, {valueOf: function () { throw sentinel; }}); }
  catch (e) { caught = e === sentinel; }
  check(caught, true, 'repeat abrupt identity');
  var large = 'ab\ud800'.repeat(8193);
  check(large.length, 24579, 'large repeat length');
  check(large.slice(0, 6), 'ab\ud800ab\ud800', 'large repeat prefix');
  check(large.slice(-6), 'ab\ud800ab\ud800', 'large repeat suffix');
  print('string-bulk-copy:ok');
})();
