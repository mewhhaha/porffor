(function () {
  function check(actual, expected, label) {
    if (actual !== expected) throw new Error(label);
  }
  var log = '';
  var later = {toString: function () { log += 'c'; return 'C'; }};
  var first = {toString: function () {
    log += 'a'; later.toString = function () { log += 'm'; return 'M'; }; return 'A';
  }};
  var second = {toString: function () { log += 'b'; return 'B'; }};
  var receiver = {toString: function () { log += 'r'; return 'R'; }};
  check(String.prototype.concat.call(receiver, first, second, later, 'D', 'E', ''),
        'RABMDE', 'coercion result');
  check(log, 'rabm', 'left to right coercions');
  var sentinel = {};
  var stop = {toString: function () { log += '!'; throw sentinel; }};
  var never = {toString: function () { throw new Error('coercion after abrupt'); }};
  var caught = false;
  try { 'x'.concat('a', stop, never, never, never); }
  catch (e) { caught = e === sentinel; }
  check(caught, true, 'abrupt identity');
  check(log, 'rabm!', 'abrupt ordering');
  caught = false;
  try { 'x'.concat('a', 'b', Symbol('s'), never, never); }
  catch (e) { caught = e instanceof TypeError; }
  check(caught, true, 'symbol throws');
  check('\ud800'.concat('\udc00', '', '\ud800', 'x', '\udc00'),
        '\ud800\udc00\ud800x\udc00', 'UTF16 seams');
  check(''.concat('', '', '', '', '', ''), '', 'empty parts');
  check('x'.concat(), 'x', 'zero arguments');
  check('x'.concat('a'), 'xa', 'one argument');
  check('x'.concat('a', 'b', 'c', 'd'), 'xabcd', 'small path');
  print('batched-string-concat:ok');
})();
